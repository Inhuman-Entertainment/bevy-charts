//! 3D scatter plots.
//!
//! Unlike the categorical charts, scatter points carry their own x/y/z data
//! coordinates, which the chart maps into its bounding box using the combined
//! extent of every series — so series stay comparable against one axis.

use bevy::prelude::*;

use crate::axis::{AxisStyle, Scale, spawn_axes};
use crate::charts::{ChartAssets, ChartChanged, ChartPrimitives, ChartSize, MaterialCache};
use crate::data::{PointDataset, point_bounds};
use crate::palette::ChartPalette;

/// A 3D scatter plot.
///
/// # Series count
///
/// A scatter plot asks the viewer to tell apart marks that are never adjacent,
/// so its colors must separate pairwise, not just in sequence. The palette
/// guarantees that for its first three slots. Past three series, facet into
/// several charts rather than trusting color alone.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_charts::prelude::*;
/// # fn setup(mut commands: Commands) {
/// commands.spawn(ScatterChart3d::new(vec![PointDataset::new(
///     "Samples",
///     vec![Vec3::new(0.0, 1.0, 2.0), Vec3::new(1.0, 3.0, 0.5)],
/// )]));
/// # }
/// ```
#[derive(Component, Debug, Clone, PartialEq)]
#[require(Transform, Visibility, ChartSize, ChartPalette, AxisStyle)]
pub struct ScatterChart3d {
    /// The point series to draw.
    pub datasets: Vec<PointDataset>,
    /// Diameter of each marker in world units.
    pub point_size: f32,
}

impl Default for ScatterChart3d {
    fn default() -> Self {
        Self {
            datasets: Vec::new(),
            point_size: 0.12,
        }
    }
}

impl ScatterChart3d {
    /// A scatter plot of `datasets` with default marker size.
    pub fn new(datasets: impl Into<Vec<PointDataset>>) -> Self {
        Self {
            datasets: datasets.into(),
            ..default()
        }
    }
}

fn build_scatter_charts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<MaterialCache>,
    primitives: Res<ChartPrimitives>,
    charts: Query<
        (
            Entity,
            &ScatterChart3d,
            &ChartSize,
            &ChartPalette,
            &AxisStyle,
        ),
        ChartChanged<ScatterChart3d>,
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

        let Some((min, max)) = point_bounds(&chart.datasets) else {
            commands.entity(entity).despawn_related::<Children>();
            continue;
        };

        let x_scale = Scale::new(min.x, max.x, size.x);
        let y_scale = Scale::new(min.y, max.y, size.y);
        let z_scale = Scale::new(min.z, max.z, size.z);

        commands
            .entity(entity)
            .despawn_related::<Children>()
            .with_children(|parent| {
                spawn_axes(parent, &mut assets, size, &y_scale, axis_style, palette);

                for (series_index, dataset) in chart.datasets.iter().enumerate() {
                    let color = dataset
                        .color
                        .unwrap_or_else(|| palette.series_color(series_index));
                    let material = assets.lit_material(color);

                    for point in dataset.points.iter().filter(|p| p.is_finite()) {
                        parent.spawn((
                            Mesh3d(assets.primitives.unit_sphere.clone()),
                            MeshMaterial3d(material.clone()),
                            Transform::from_xyz(
                                x_scale.map(point.x),
                                y_scale.map(point.y),
                                z_scale.map(point.z),
                            )
                            .with_scale(Vec3::splat(chart.point_size)),
                            Visibility::default(),
                            crate::charts::ChartElement,
                        ));
                    }
                }
            });
    }
}

/// Adds the [`ScatterChart3d`] build system.
pub struct ScatterChartPlugin;

impl Plugin for ScatterChartPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::ChartCorePlugin)
            .add_systems(PostUpdate, build_scatter_charts);
    }
}
