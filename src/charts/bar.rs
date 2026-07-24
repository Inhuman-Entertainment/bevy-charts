//! Grouped 3D bar charts.
//!
//! Categories run along x and series along z, so a multi-series chart reads as
//! rows of bars receding into the scene rather than as bars competing for the
//! same footprint.

use bevy::prelude::*;

use crate::axis::{AxisStyle, Scale, spawn_axes};
use crate::charts::{
    ChartAssets, ChartChanged, ChartPrimitives, ChartSize, MaterialCache, box_transform,
};
use crate::data::ChartData;
use crate::palette::ChartPalette;

/// A grouped bar chart.
///
/// Spawn it with a [`ChartData`]; the chart rebuilds itself whenever the data or
/// any of its style components change.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_charts::prelude::*;
/// # fn setup(mut commands: Commands) {
/// commands.spawn(BarChart3d::new(
///     ChartData::new()
///         .with_labels(["Q1", "Q2", "Q3"])
///         .with_dataset(Dataset::new("Revenue", vec![12.0, 19.0, 7.0])),
/// ));
/// # }
/// ```
#[derive(Component, Debug, Clone, PartialEq)]
#[require(Transform, Visibility, ChartSize, ChartPalette, AxisStyle)]
pub struct BarChart3d {
    /// Categories and series to draw.
    pub data: ChartData,
    /// Fraction of each category slot left empty, in `0.0..1.0`.
    pub category_gap: f32,
    /// Fraction of each series slot left empty, in `0.0..1.0`.
    pub series_gap: f32,
}

impl Default for BarChart3d {
    fn default() -> Self {
        Self {
            data: ChartData::default(),
            category_gap: 0.25,
            series_gap: 0.2,
        }
    }
}

impl BarChart3d {
    /// A bar chart of `data` with default spacing.
    pub fn new(data: ChartData) -> Self {
        Self { data, ..default() }
    }
}

/// Value scale for a bar chart: always includes zero, because a bar's length is
/// only meaningful when measured from it.
pub(crate) fn bar_value_scale(data: &ChartData, height: f32) -> Option<Scale> {
    let (min, max) = data.value_range()?;
    Some(Scale::new(min.min(0.0), max.max(0.0), height))
}

/// How a set of bars is laid out inside a chart's box.
///
/// Grouped so that bar and histogram charts, which differ only in how they
/// arrive at their categories, can share [`spawn_bars`] without passing a long
/// tail of loose floats.
#[derive(Clone, Copy)]
pub(crate) struct BarLayout<'a> {
    /// The box the bars fill.
    pub size: Vec3,
    /// The value axis.
    pub scale: &'a Scale,
    /// Fraction of each category slot left empty, in `0.0..1.0`.
    pub category_gap: f32,
    /// Fraction of each series slot left empty, in `0.0..1.0`.
    pub series_gap: f32,
}

/// Spawn the bars for `data` as children of `parent`.
///
/// Split out from the system so [`HistogramChart3d`](crate::charts::histogram::HistogramChart3d)
/// can bin its values and draw them with exactly the same geometry.
pub(crate) fn spawn_bars(
    parent: &mut ChildSpawnerCommands,
    assets: &mut ChartAssets,
    data: &ChartData,
    palette: &ChartPalette,
    layout: &BarLayout,
) {
    let categories = data.category_count();
    if categories == 0 || data.datasets.is_empty() {
        return;
    }

    let BarLayout {
        size,
        scale,
        category_gap,
        series_gap,
    } = *layout;

    let slot_x = size.x / categories as f32;
    let slot_z = size.z / data.datasets.len() as f32;
    let bar_x = slot_x * (1.0 - category_gap.clamp(0.0, 0.95));
    let bar_z = slot_z * (1.0 - series_gap.clamp(0.0, 0.95));
    // Bars grow from wherever zero sits, which is the bottom unless the data
    // goes negative.
    let baseline = scale.map(0.0f32.clamp(scale.min, scale.max));

    for (series_index, dataset) in data.datasets.iter().enumerate() {
        let color = dataset
            .color
            .unwrap_or_else(|| palette.series_color(series_index));
        let material = assets.lit_material(color);
        let z_center = (series_index as f32 + 0.5) * slot_z;

        for (category_index, value) in dataset.data.iter().copied().enumerate() {
            if !value.is_finite() {
                continue;
            }
            let x_center = (category_index as f32 + 0.5) * slot_x;
            let top = scale.map(value);
            let (y_min, y_max) = if top >= baseline {
                (baseline, top)
            } else {
                (top, baseline)
            };

            parent.spawn((
                Mesh3d(assets.primitives.unit_cube.clone()),
                MeshMaterial3d(material.clone()),
                box_transform(
                    Vec3::new(x_center - bar_x * 0.5, y_min, z_center - bar_z * 0.5),
                    Vec3::new(x_center + bar_x * 0.5, y_max, z_center + bar_z * 0.5),
                ),
                Visibility::default(),
                crate::charts::ChartElement,
            ));
        }
    }
}

fn build_bar_charts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<MaterialCache>,
    primitives: Res<ChartPrimitives>,
    charts: Query<
        (Entity, &BarChart3d, &ChartSize, &ChartPalette, &AxisStyle),
        ChartChanged<BarChart3d>,
    >,
) {
    for (entity, chart, size, palette, axis_style) in &charts {
        let size = size.0;
        let mut assets = ChartAssets {
            meshes: &mut meshes,
            materials: &mut materials,
            cache: &mut cache,
            primitives: &primitives,
        };

        let Some(scale) = bar_value_scale(&chart.data, size.y) else {
            // Nothing finite to plot; clear whatever was there and move on.
            commands.entity(entity).despawn_related::<Children>();
            continue;
        };

        commands
            .entity(entity)
            .despawn_related::<Children>()
            .with_children(|parent| {
                spawn_axes(parent, &mut assets, size, &scale, axis_style, palette);
                spawn_bars(
                    parent,
                    &mut assets,
                    &chart.data,
                    palette,
                    &BarLayout {
                        size,
                        scale: &scale,
                        category_gap: chart.category_gap,
                        series_gap: chart.series_gap,
                    },
                );
            });
    }
}

/// Adds the [`BarChart3d`] build system.
///
/// Included in [`BevyChartsPlugin`](crate::BevyChartsPlugin); add it directly
/// only if you want bar charts without the other chart types.
pub struct BarChartPlugin;

impl Plugin for BarChartPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::ChartCorePlugin)
            .add_systems(PostUpdate, build_bar_charts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Dataset;

    #[test]
    fn value_scale_always_includes_zero() {
        let data = ChartData::new().with_dataset(Dataset::new("s", vec![10.0, 20.0]));
        let scale = bar_value_scale(&data, 3.0).unwrap();
        assert_eq!(scale.min, 0.0, "bars must be measured from zero");
        assert_eq!(scale.max, 20.0);
    }

    #[test]
    fn value_scale_spans_zero_when_data_is_negative() {
        let data = ChartData::new().with_dataset(Dataset::new("s", vec![-5.0, 8.0]));
        let scale = bar_value_scale(&data, 3.0).unwrap();
        assert_eq!((scale.min, scale.max), (-5.0, 8.0));
    }

    #[test]
    fn empty_data_has_no_scale() {
        assert!(bar_value_scale(&ChartData::new(), 3.0).is_none());
    }
}
