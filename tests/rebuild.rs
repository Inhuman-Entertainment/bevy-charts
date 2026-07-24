//! Integration tests driving a real (headless) Bevy `App`.
//!
//! The unit tests cover the pure functions — scales, binning, palettes. These
//! cover the part that only exists once an `App` is running: that spawning a
//! chart component actually produces entities, that changing data rebuilds
//! them, that leaving it alone does not, and that the produced entities carry
//! everything the renderer needs.
//!
//! That last one is not hypothetical. `Mesh3d` in Bevy 0.19 requires only
//! `Transform`, not `Visibility`, so chart children were once spawned without
//! it — they compiled, passed every unit test, and rendered nothing at all.

use bevy::prelude::*;
use bevy_charts::charts::ChartElement;
use bevy_charts::prelude::*;

/// An app with everything the chart systems touch and nothing else: no window,
/// no renderer, no GPU.
fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .add_plugins(BevyChartsPlugin);
    app
}

fn sample_data() -> ChartData {
    ChartData::new()
        .with_labels(["a", "b", "c"])
        .with_dataset(Dataset::new("one", vec![1.0, 2.0, 3.0]))
        .with_dataset(Dataset::new("two", vec![3.0, 2.0, 1.0]))
}

/// Entities the chart generated as data marks, excluding axis and grid lines.
fn marks(app: &mut App) -> Vec<Entity> {
    app.world_mut()
        .query_filtered::<Entity, With<ChartElement>>()
        .iter(app.world())
        .collect()
}

fn children_of(app: &mut App, chart: Entity) -> Vec<Entity> {
    app.world()
        .get::<Children>(chart)
        .map(|c| c.iter().collect())
        .unwrap_or_default()
}

#[test]
fn spawning_a_chart_produces_geometry() {
    let mut app = headless_app();
    let chart = app.world_mut().spawn(BarChart3d::new(sample_data())).id();
    app.update();

    let children = children_of(&mut app, chart);
    assert!(!children.is_empty(), "chart spawned no children");

    // 3 categories x 2 series.
    assert_eq!(marks(&mut app).len(), 6);
}

#[test]
fn every_generated_entity_carries_what_the_renderer_needs() {
    // The regression test for the black-screen bug: a mesh entity without
    // `Visibility` never gets `ViewVisibility`, so render extraction skips it.
    let mut app = headless_app();
    let chart = app.world_mut().spawn(BarChart3d::new(sample_data())).id();
    app.update();

    for child in children_of(&mut app, chart) {
        let entity = app.world().entity(child);
        assert!(entity.contains::<Mesh3d>(), "child is missing Mesh3d");
        assert!(
            entity.contains::<MeshMaterial3d<StandardMaterial>>(),
            "child is missing a material"
        );
        assert!(entity.contains::<Transform>(), "child is missing Transform");
        assert!(
            entity.contains::<Visibility>(),
            "child is missing Visibility, so it would never be rendered"
        );
    }
}

#[test]
fn the_referenced_mesh_and_material_assets_actually_exist() {
    let mut app = headless_app();
    let chart = app.world_mut().spawn(BarChart3d::new(sample_data())).id();
    app.update();

    let children = children_of(&mut app, chart);
    let meshes = app.world().resource::<Assets<Mesh>>();
    let materials = app.world().resource::<Assets<StandardMaterial>>();
    for child in children {
        let entity = app.world().entity(child);
        let mesh = entity.get::<Mesh3d>().unwrap();
        let material = entity.get::<MeshMaterial3d<StandardMaterial>>().unwrap();
        assert!(meshes.get(&mesh.0).is_some(), "dangling mesh handle");
        assert!(materials.get(&material.0).is_some(), "dangling material");
    }
}

#[test]
fn mutating_the_data_rebuilds_the_chart() {
    let mut app = headless_app();
    let chart = app.world_mut().spawn(BarChart3d::new(sample_data())).id();
    app.update();
    assert_eq!(marks(&mut app).len(), 6);

    app.world_mut()
        .get_mut::<BarChart3d>(chart)
        .unwrap()
        .data
        .datasets
        .push(Dataset::new("three", vec![5.0, 5.0, 5.0]));
    app.update();

    assert_eq!(marks(&mut app).len(), 9, "the new series was not drawn");
}

#[test]
fn an_unchanged_chart_is_left_alone() {
    let mut app = headless_app();
    app.world_mut().spawn(BarChart3d::new(sample_data()));
    app.update();
    let first = marks(&mut app);

    // Several frames with no change at all.
    for _ in 0..3 {
        app.update();
    }

    // Identity, not just count: a rebuild would despawn these and spawn new
    // entities with different ids, which a count alone would not notice.
    assert_eq!(marks(&mut app), first, "chart was rebuilt without a change");
}

#[test]
fn changing_only_the_style_still_rebuilds() {
    let mut app = headless_app();
    let chart = app.world_mut().spawn(BarChart3d::new(sample_data())).id();
    app.update();
    let first = marks(&mut app);

    *app.world_mut().get_mut::<ChartPalette>(chart).unwrap() = ChartPalette::light();
    app.update();

    let second = marks(&mut app);
    assert_eq!(second.len(), first.len());
    assert_ne!(second, first, "palette change did not rebuild the chart");
}

#[test]
fn repeated_rebuilds_do_not_leak_materials() {
    let mut app = headless_app();
    let chart = app.world_mut().spawn(BarChart3d::new(sample_data())).id();
    app.update();
    let after_first = app.world().resource::<Assets<StandardMaterial>>().len();

    // Twenty rebuilds using the same two series colors.
    for i in 0..20 {
        app.world_mut()
            .get_mut::<BarChart3d>(chart)
            .unwrap()
            .data
            .datasets[0]
            .data[0] = i as f32;
        app.update();
    }

    let after_many = app.world().resource::<Assets<StandardMaterial>>().len();
    assert_eq!(
        after_many, after_first,
        "each rebuild minted new materials instead of reusing the cache"
    );
}

#[test]
fn a_chart_with_nothing_plottable_draws_nothing() {
    let mut app = headless_app();
    app.world_mut().spawn(BarChart3d::new(ChartData::new()));
    app.update();
    assert!(marks(&mut app).is_empty());

    let all_nan = ChartData::new().with_dataset(Dataset::new("s", vec![f32::NAN, f32::INFINITY]));
    app.world_mut().spawn(BarChart3d::new(all_nan));
    app.update();
    assert!(marks(&mut app).is_empty());
}

#[test]
fn emptying_a_chart_clears_the_geometry_it_had() {
    let mut app = headless_app();
    let chart = app.world_mut().spawn(BarChart3d::new(sample_data())).id();
    app.update();
    assert!(!marks(&mut app).is_empty());

    app.world_mut().get_mut::<BarChart3d>(chart).unwrap().data = ChartData::new();
    app.update();

    assert!(
        marks(&mut app).is_empty(),
        "stale geometry survived the data being emptied"
    );
}

#[test]
fn non_finite_values_are_skipped_rather_than_drawn() {
    let mut app = headless_app();
    let data =
        ChartData::new().with_dataset(Dataset::new("s", vec![1.0, f32::NAN, 3.0, f32::INFINITY]));
    app.world_mut().spawn(BarChart3d::new(data));
    app.update();

    assert_eq!(
        marks(&mut app).len(),
        2,
        "a non-finite value produced a bar"
    );
}

#[test]
fn every_chart_type_builds() {
    let mut app = headless_app();

    app.world_mut().spawn(BarChart3d::new(sample_data()));
    app.world_mut().spawn(LineChart3d::new(sample_data()));
    app.world_mut()
        .spawn(ScatterChart3d::new(vec![PointDataset::new(
            "pts",
            vec![Vec3::ZERO, Vec3::ONE, Vec3::splat(2.0)],
        )]));
    app.world_mut()
        .spawn(SurfaceChart3d::new(4, 4, vec![1.0; 16]));
    app.world_mut()
        .spawn(HistogramChart3d::new(vec![Dataset::new(
            "obs",
            vec![1.0, 2.0, 2.0, 3.0],
        )]));
    app.update();

    let bar = 3 * 2; // one box per category per series
    let line = 2 * 2 + 3 * 2; // a segment between each pair, plus a marker per point
    let scatter = 3; // one marker per point
    let surface = 1; // a single generated mesh
    let histogram = 10; // the default bin count, including the empty bins
    assert_eq!(
        marks(&mut app).len(),
        bar + line + scatter + surface + histogram
    );
}

#[test]
fn a_single_chart_plugin_is_enough_on_its_own() {
    // Each chart plugin registers the shared resources itself, so a game that
    // only wants bar charts should not have to add the others.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .add_plugins(bevy_charts::BarChartPlugin);

    app.world_mut().spawn(BarChart3d::new(sample_data()));
    app.update();

    assert_eq!(marks(&mut app).len(), 6);
}

#[test]
fn a_chart_stays_inside_the_box_it_was_given() {
    let mut app = headless_app();
    let size = Vec3::new(6.0, 4.0, 2.0);
    let chart = app
        .world_mut()
        .spawn((BarChart3d::new(sample_data()), ChartSize(size)))
        .id();
    app.update();

    for child in children_of(&mut app, chart) {
        let Some(transform) = app.world().get::<Transform>(child) else {
            continue;
        };
        let p = transform.translation;
        // Half a bar may sit outside on the value axis at most; the category
        // and series axes should be strictly inside.
        assert!(
            p.x >= -0.01 && p.x <= size.x + 0.01,
            "mark escaped the box on x: {p:?}"
        );
        assert!(
            p.z >= -0.01 && p.z <= size.z + 0.01,
            "mark escaped the box on z: {p:?}"
        );
    }
}
