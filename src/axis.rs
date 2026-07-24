//! Value scales, tick placement, and the axis/grid geometry every chart shares.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

use crate::palette::ChartPalette;

/// Below this the data range counts as degenerate and gets widened, so that
/// mapping a value never divides by zero.
const MIN_SPAN: f32 = 1e-6;

/// Maps a data range onto one axis of a chart's bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scale {
    /// Lowest data value the axis shows.
    pub min: f32,
    /// Highest data value the axis shows.
    pub max: f32,
    /// Length of the axis in world units.
    pub extent: f32,
}

impl Scale {
    /// A scale over `min..max` laid out along `extent` world units.
    ///
    /// A degenerate range (all values equal, or reversed) is widened around its
    /// midpoint so the chart still draws something sensible instead of collapsing.
    pub fn new(min: f32, max: f32, extent: f32) -> Self {
        let (min, max) = if max - min < MIN_SPAN {
            let mid = (min + max) * 0.5;
            (mid - 0.5, mid + 0.5)
        } else {
            (min, max)
        };
        Self { min, max, extent }
    }

    /// Data span the axis covers.
    pub fn span(&self) -> f32 {
        self.max - self.min
    }

    /// Position of `value` along the axis, in world units from its origin.
    pub fn map(&self, value: f32) -> f32 {
        (value - self.min) / self.span() * self.extent
    }

    /// Roughly `count` round-numbered tick values inside the range.
    ///
    /// Placement is delegated to `plotters`, whose `key_points` implements the
    /// usual 1/2/5-times-a-power-of-ten search. The count is a hint: the whole
    /// point is to land on round numbers, so the result may be a tick or two off.
    pub fn ticks(&self, count: usize) -> Vec<f32> {
        use plotters::coord::ranged1d::Ranged;
        use plotters::coord::types::RangedCoordf64;

        if count == 0 {
            return Vec::new();
        }
        let range: RangedCoordf64 = (self.min as f64..self.max as f64).into();
        range
            .key_points(count)
            .into_iter()
            .map(|v| v as f32)
            .filter(|v| *v >= self.min && *v <= self.max)
            .collect()
    }
}

/// How a chart draws its axes and grid.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct AxisStyle {
    /// Draw the three axis lines meeting at the chart origin.
    pub show_axes: bool,
    /// Draw grid lines across the back and side walls at each value tick.
    pub show_grid: bool,
    /// Hint for how many value ticks to place. See [`Scale::ticks`].
    pub tick_count: usize,
}

impl Default for AxisStyle {
    fn default() -> Self {
        Self {
            show_axes: true,
            show_grid: true,
            tick_count: 5,
        }
    }
}

/// Build a mesh of independent line segments.
///
/// Lines render a fixed pixel width regardless of distance, which is what a
/// recessive grid wants — thickening it with geometry would fight the data.
pub fn line_list_mesh(segments: &[[Vec3; 2]]) -> Mesh {
    let positions: Vec<[f32; 3]> = segments
        .iter()
        .flatten()
        .map(|p| [p.x, p.y, p.z])
        .collect();
    Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
}

/// Segments for the axis lines and grid of a chart of size `size`.
///
/// `value_scale` describes the vertical axis, so grid lines land on the same
/// round numbers the value ticks do. Returns axis segments and grid segments
/// separately so they can take different colors.
pub fn axis_segments(
    size: Vec3,
    value_scale: &Scale,
    style: &AxisStyle,
) -> (Vec<[Vec3; 2]>, Vec<[Vec3; 2]>) {
    let mut axes = Vec::new();
    let mut grid = Vec::new();

    // The vertical axis starts at whichever end of the box the value zero (or the
    // nearest edge of the range) sits at, so bars hanging below zero still meet it.
    let base = value_scale.map(value_scale.min.max(0.0).min(value_scale.max));

    if style.show_axes {
        axes.push([Vec3::ZERO, Vec3::new(size.x, 0.0, 0.0)]);
        axes.push([Vec3::ZERO, Vec3::new(0.0, size.y, 0.0)]);
        axes.push([Vec3::ZERO, Vec3::new(0.0, 0.0, size.z)]);
        // The zero line, when zero is somewhere in the middle of the range.
        if base > MIN_SPAN && base < size.y - MIN_SPAN {
            axes.push([Vec3::new(0.0, base, 0.0), Vec3::new(size.x, base, 0.0)]);
        }
    }

    if style.show_grid {
        for tick in value_scale.ticks(style.tick_count) {
            let y = value_scale.map(tick);
            // Back wall (z = 0) and left wall (x = 0), so the grid reads as the
            // inside of a box rather than floating planes.
            grid.push([Vec3::new(0.0, y, 0.0), Vec3::new(size.x, y, 0.0)]);
            grid.push([Vec3::new(0.0, y, 0.0), Vec3::new(0.0, y, size.z)]);
        }
    }

    (axes, grid)
}

/// Spawn the axis and grid entities for a chart as children of `parent`.
pub(crate) fn spawn_axes(
    parent: &mut ChildSpawnerCommands,
    ctx: &mut crate::charts::ChartAssets,
    size: Vec3,
    value_scale: &Scale,
    style: &AxisStyle,
    palette: &ChartPalette,
) {
    let (axes, grid) = axis_segments(size, value_scale, style);

    for (segments, color) in [(axes, palette.axis), (grid, palette.grid)] {
        if segments.is_empty() {
            continue;
        }
        let mesh = ctx.meshes.add(line_list_mesh(&segments));
        let material = ctx.unlit_material(color);
        parent.spawn((Mesh3d(mesh), MeshMaterial3d(material), Visibility::default()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_endpoints_to_the_extent() {
        let scale = Scale::new(0.0, 10.0, 4.0);
        assert_eq!(scale.map(0.0), 0.0);
        assert_eq!(scale.map(10.0), 4.0);
        assert_eq!(scale.map(5.0), 2.0);
    }

    #[test]
    fn widens_a_degenerate_range_instead_of_dividing_by_zero() {
        let scale = Scale::new(3.0, 3.0, 2.0);
        assert!(scale.span() >= MIN_SPAN);
        assert!(scale.map(3.0).is_finite());
    }

    #[test]
    fn ticks_are_round_numbers_inside_the_range() {
        let scale = Scale::new(0.0, 100.0, 1.0);
        let ticks = scale.ticks(5);
        assert!(!ticks.is_empty());
        assert!(ticks.iter().all(|t| *t >= 0.0 && *t <= 100.0));
        // Round numbers are the whole reason we defer to plotters here.
        assert!(ticks.contains(&50.0), "expected a tick at 50, got {ticks:?}");
    }

    #[test]
    fn zero_ticks_requested_yields_none() {
        assert!(Scale::new(0.0, 10.0, 1.0).ticks(0).is_empty());
    }
}
