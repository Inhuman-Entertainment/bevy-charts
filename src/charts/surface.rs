//! 3D surface charts.
//!
//! This is the one chart that generates real geometry rather than placing shared
//! primitives: a height field over a `cols` × `rows` grid, triangulated and
//! shaded with per-vertex colors from the sequential ramp so height is encoded
//! twice — by elevation and by color.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::axis::{AxisStyle, Scale, spawn_axes};
use crate::charts::{ChartAssets, ChartChanged, ChartPrimitives, ChartSize, MaterialCache};
use crate::palette::ChartPalette;

/// A height-field surface.
///
/// `heights` is row-major: `heights[row * cols + col]`, with `col` running along
/// x and `row` along z. A chart whose length does not match `cols * rows` draws
/// nothing rather than guessing at the intended shape.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_charts::prelude::*;
/// # fn setup(mut commands: Commands) {
/// let (cols, rows) = (16, 16);
/// let heights = (0..rows * cols)
///     .map(|i| {
///         let (x, z) = ((i % cols) as f32 * 0.4, (i / cols) as f32 * 0.4);
///         (x.sin() + z.cos()) * 0.5
///     })
///     .collect::<Vec<_>>();
/// commands.spawn(SurfaceChart3d::new(cols, rows, heights));
/// # }
/// ```
#[derive(Component, Debug, Clone, PartialEq, Default)]
#[require(Transform, Visibility, ChartSize, ChartPalette, AxisStyle)]
pub struct SurfaceChart3d {
    /// Number of samples along x.
    pub cols: usize,
    /// Number of samples along z.
    pub rows: usize,
    /// Row-major height samples, `rows * cols` of them.
    pub heights: Vec<f32>,
}

impl SurfaceChart3d {
    /// A surface over a `cols` × `rows` grid of row-major `heights`.
    pub fn new(cols: usize, rows: usize, heights: impl Into<Vec<f32>>) -> Self {
        Self {
            cols,
            rows,
            heights: heights.into(),
        }
    }

    /// Whether the grid dimensions and sample count agree, and there is at least
    /// one quad to draw.
    pub fn is_valid(&self) -> bool {
        self.cols >= 2 && self.rows >= 2 && self.heights.len() == self.cols * self.rows
    }

    /// Smallest and largest finite sample.
    fn height_range(&self) -> Option<(f32, f32)> {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for h in self.heights.iter().copied().filter(|h| h.is_finite()) {
            min = min.min(h);
            max = max.max(h);
        }
        (min <= max).then_some((min, max))
    }
}

/// Triangulate the height field into a mesh spanning `size` in x and z.
fn surface_mesh(
    chart: &SurfaceChart3d,
    size: Vec3,
    scale: &Scale,
    palette: &ChartPalette,
) -> Mesh {
    let (cols, rows) = (chart.cols, chart.rows);
    let mut positions = Vec::with_capacity(cols * rows);
    let mut colors = Vec::with_capacity(cols * rows);
    let mut uvs = Vec::with_capacity(cols * rows);

    for row in 0..rows {
        let v = row as f32 / (rows - 1) as f32;
        for col in 0..cols {
            let u = col as f32 / (cols - 1) as f32;
            let height = chart.heights[row * cols + col];
            let height = if height.is_finite() { height } else { scale.min };

            positions.push([u * size.x, scale.map(height), v * size.z]);
            uvs.push([u, v]);

            let t = (height - scale.min) / scale.span();
            colors.push(palette.sequential(t).to_linear().to_f32_array());
        }
    }

    // Two triangles per cell, wound counter-clockwise seen from +y so the
    // computed normals point up out of the surface.
    let mut indices = Vec::with_capacity((cols - 1) * (rows - 1) * 6);
    for row in 0..rows - 1 {
        for col in 0..cols - 1 {
            let i00 = (row * cols + col) as u32;
            let i10 = i00 + 1;
            let i01 = i00 + cols as u32;
            let i11 = i01 + 1;
            indices.extend_from_slice(&[i00, i01, i10, i10, i01, i11]);
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices))
        .with_computed_normals()
}

fn build_surface_charts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<MaterialCache>,
    primitives: Res<ChartPrimitives>,
    charts: Query<
        (
            Entity,
            &SurfaceChart3d,
            &ChartSize,
            &ChartPalette,
            &AxisStyle,
        ),
        ChartChanged<SurfaceChart3d>,
    >,
) {
    for (entity, chart, size, palette, axis_style) in &charts {
        let size = size.0;

        let scale = chart
            .is_valid()
            .then(|| chart.height_range())
            .flatten()
            .map(|(min, max)| Scale::new(min, max, size.y));
        let Some(scale) = scale else {
            commands.entity(entity).despawn_related::<Children>();
            continue;
        };

        let mesh = meshes.add(surface_mesh(chart, size, &scale, palette));
        // Vertex colors carry the encoding, so the base color must not tint them.
        // The surface is viewable from below, so it is not back-face culled.
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.85,
            double_sided: true,
            cull_mode: None,
            ..default()
        });

        let mut assets = ChartAssets {
            meshes: &mut meshes,
            materials: &mut materials,
            cache: &mut cache,
            primitives: &primitives,
        };

        commands
            .entity(entity)
            .despawn_related::<Children>()
            .with_children(|parent| {
                spawn_axes(parent, &mut assets, size, &scale, axis_style, palette);
                parent.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Visibility::default(),
                    crate::charts::ChartElement,
                ));
            });
    }
}

/// Adds the [`SurfaceChart3d`] build system.
pub struct SurfaceChartPlugin;

impl Plugin for SurfaceChartPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::ChartCorePlugin)
            .add_systems(PostUpdate, build_surface_charts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_a_grid_whose_sample_count_disagrees() {
        assert!(!SurfaceChart3d::new(3, 3, vec![0.0; 8]).is_valid());
        assert!(SurfaceChart3d::new(3, 3, vec![0.0; 9]).is_valid());
    }

    #[test]
    fn rejects_a_grid_too_small_to_have_a_quad() {
        assert!(!SurfaceChart3d::new(1, 4, vec![0.0; 4]).is_valid());
    }

    #[test]
    fn triangulates_two_triangles_per_cell() {
        let chart = SurfaceChart3d::new(3, 4, vec![1.0; 12]);
        let scale = Scale::new(0.0, 1.0, 1.0);
        let mesh = surface_mesh(&chart, Vec3::splat(1.0), &scale, &ChartPalette::dark());
        let indices = mesh.indices().expect("surface mesh is indexed");
        // (3-1) * (4-1) cells, 6 indices each.
        assert_eq!(indices.len(), 2 * 3 * 6);
        assert_eq!(mesh.count_vertices(), 12);
    }
}
