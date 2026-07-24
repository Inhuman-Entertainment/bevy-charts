//! 3D line charts.
//!
//! Each series is a polyline at its own depth along z. Segments are drawn as
//! thin boxes rather than GPU lines so they take the scene's lighting and keep a
//! constant world-space thickness as the camera moves — a one-pixel line looks
//! wrong next to lit geometry.

use bevy::prelude::*;

use crate::axis::{AxisStyle, Scale, spawn_axes};
use crate::charts::{
    ChartAssets, ChartChanged, ChartPrimitives, ChartSize, MaterialCache, segment_transform,
};
use crate::data::ChartData;
use crate::palette::ChartPalette;

/// A multi-series line chart.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_charts::prelude::*;
/// # fn setup(mut commands: Commands) {
/// commands.spawn(LineChart3d::new(
///     ChartData::new()
///         .with_labels(["Mon", "Tue", "Wed"])
///         .with_dataset(Dataset::new("Load", vec![3.0, 5.0, 4.0])),
/// ));
/// # }
/// ```
#[derive(Component, Debug, Clone, PartialEq)]
#[require(Transform, Visibility, ChartSize, ChartPalette, AxisStyle)]
pub struct LineChart3d {
    /// Categories and series to draw.
    pub data: ChartData,
    /// World-space thickness of each line.
    pub thickness: f32,
    /// Draw a marker at each data point. Markers also hide the mitre gap where
    /// two segments meet at an angle.
    pub show_points: bool,
    /// Diameter of the point markers.
    pub point_size: f32,
    /// Start the value axis at zero even when the data does not reach it.
    pub begin_at_zero: bool,
}

impl Default for LineChart3d {
    fn default() -> Self {
        Self {
            data: ChartData::default(),
            thickness: 0.04,
            show_points: true,
            point_size: 0.1,
            begin_at_zero: false,
        }
    }
}

impl LineChart3d {
    /// A line chart of `data` with default styling.
    pub fn new(data: ChartData) -> Self {
        Self {
            data,
            ..default()
        }
    }
}

/// Value scale for a line chart.
///
/// Unlike bars, lines encode position rather than length, so the axis does not
/// have to include zero — forcing it would flatten a series that varies in a
/// narrow band far from the origin.
fn line_value_scale(chart: &LineChart3d, height: f32) -> Option<Scale> {
    let (min, max) = chart.data.value_range()?;
    if chart.begin_at_zero {
        Some(Scale::new(min.min(0.0), max.max(0.0), height))
    } else {
        Some(Scale::new(min, max, height))
    }
}

fn build_line_charts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<MaterialCache>,
    primitives: Res<ChartPrimitives>,
    charts: Query<
        (Entity, &LineChart3d, &ChartSize, &ChartPalette, &AxisStyle),
        ChartChanged<LineChart3d>,
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

        let Some(scale) = line_value_scale(chart, size.y) else {
            commands.entity(entity).despawn_related::<Children>();
            continue;
        };

        let data: &ChartData = &chart.data;
        let categories = data.category_count();
        // A single category has no span to spread across; keep it centered
        // instead of dividing by zero.
        let step_x = if categories > 1 {
            size.x / (categories - 1) as f32
        } else {
            0.0
        };
        let slot_z = if data.datasets.is_empty() {
            0.0
        } else {
            size.z / data.datasets.len() as f32
        };

        commands
            .entity(entity)
            .despawn_related::<Children>()
            .with_children(|parent| {
                spawn_axes(parent, &mut assets, size, &scale, axis_style, palette);

                for (series_index, dataset) in data.datasets.iter().enumerate() {
                    let color = dataset
                        .color
                        .unwrap_or_else(|| palette.series_color(series_index));
                    let material = assets.lit_material(color);
                    let z = (series_index as f32 + 0.5) * slot_z;

                    let points: Vec<Vec3> = dataset
                        .data
                        .iter()
                        .copied()
                        .enumerate()
                        .filter(|(_, v)| v.is_finite())
                        .map(|(i, v)| {
                            let x = if categories > 1 {
                                i as f32 * step_x
                            } else {
                                size.x * 0.5
                            };
                            Vec3::new(x, scale.map(v), z)
                        })
                        .collect();

                    for pair in points.windows(2) {
                        parent.spawn((
                            Mesh3d(assets.primitives.unit_cube.clone()),
                            MeshMaterial3d(material.clone()),
                            segment_transform(pair[0], pair[1], chart.thickness),
                            Visibility::default(),
                            crate::charts::ChartElement,
                        ));
                    }

                    if chart.show_points {
                        for point in &points {
                            parent.spawn((
                                Mesh3d(assets.primitives.unit_sphere.clone()),
                                MeshMaterial3d(material.clone()),
                                Transform::from_translation(*point)
                                    .with_scale(Vec3::splat(chart.point_size)),
                                Visibility::default(),
                                crate::charts::ChartElement,
                            ));
                        }
                    }
                }
            });
    }
}

/// Adds the [`LineChart3d`] build system.
pub struct LineChartPlugin;

impl Plugin for LineChartPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::ChartCorePlugin)
            .add_systems(PostUpdate, build_line_charts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Dataset;

    fn chart(values: Vec<f32>, begin_at_zero: bool) -> LineChart3d {
        LineChart3d {
            data: ChartData::new().with_dataset(Dataset::new("s", values)),
            begin_at_zero,
            ..default()
        }
    }

    #[test]
    fn does_not_force_zero_by_default() {
        let scale = line_value_scale(&chart(vec![100.0, 104.0], false), 3.0).unwrap();
        assert_eq!((scale.min, scale.max), (100.0, 104.0));
    }

    #[test]
    fn begin_at_zero_extends_the_axis_down() {
        let scale = line_value_scale(&chart(vec![100.0, 104.0], true), 3.0).unwrap();
        assert_eq!(scale.min, 0.0);
    }
}
