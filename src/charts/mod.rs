//! The chart components and the systems that turn them into meshes.
//!
//! Every chart type follows the same shape: a component holding data, a set of
//! shared style components, and a system that rebuilds the chart's children
//! whenever any of them change. Charts draw into the box `0..size` in their own
//! local space, with the origin at the bottom-left-front corner, so placing a
//! chart is just setting its [`Transform`].

pub mod bar;
pub mod histogram;
pub mod line;
pub mod scatter;
pub mod surface;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::axis::AxisStyle;
use crate::palette::ChartPalette;

/// The box a chart draws inside, in local space.
///
/// x is the category axis, y the value axis, z the series/depth axis.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ChartSize(pub Vec3);

impl Default for ChartSize {
    fn default() -> Self {
        Self(Vec3::new(4.0, 3.0, 2.0))
    }
}

/// Meshes every chart reuses instead of generating its own.
///
/// Bars, line segments, and scatter markers are all a unit primitive with a
/// [`Transform`], so a chart of a thousand points still holds two mesh assets.
#[derive(Resource, Debug)]
pub struct ChartPrimitives {
    /// A 1×1×1 cube centered on its origin.
    pub unit_cube: Handle<Mesh>,
    /// A unit-diameter sphere centered on its origin.
    pub unit_sphere: Handle<Mesh>,
}

impl FromWorld for ChartPrimitives {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        Self {
            unit_cube: meshes.add(Cuboid::from_length(1.0)),
            unit_sphere: meshes.add(Sphere::new(0.5).mesh().uv(16, 8)),
        }
    }
}

/// Materials keyed by color, so rebuilding a chart does not leak an asset per bar.
///
/// Charts rebuild on every data change; without this, each rebuild would mint a
/// fresh `StandardMaterial` for colors that already have one and the asset table
/// would grow without bound.
#[derive(Resource, Debug, Default)]
pub struct MaterialCache {
    lit: HashMap<[u32; 4], Handle<StandardMaterial>>,
    unlit: HashMap<[u32; 4], Handle<StandardMaterial>>,
}

fn color_key(color: Color) -> [u32; 4] {
    let c = color.to_linear();
    [
        c.red.to_bits(),
        c.green.to_bits(),
        c.blue.to_bits(),
        c.alpha.to_bits(),
    ]
}

/// The asset handles a chart's build system needs, bundled so the per-chart
/// builders can stay short.
pub(crate) struct ChartAssets<'a> {
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
    pub cache: &'a mut MaterialCache,
    pub primitives: &'a ChartPrimitives,
}

impl ChartAssets<'_> {
    /// A lit PBR material of this color, shared with any chart already using it.
    pub fn lit_material(&mut self, color: Color) -> Handle<StandardMaterial> {
        let key = color_key(color);
        if let Some(handle) = self.cache.lit.get(&key) {
            return handle.clone();
        }
        let handle = self.materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.8,
            ..default()
        });
        self.cache.lit.insert(key, handle.clone());
        handle
    }

    /// An unlit material of this color, for axes and grid lines.
    ///
    /// Grid lines are reference marks, not objects in the scene — shading them
    /// would make their weight depend on where the lights happen to be.
    pub fn unlit_material(&mut self, color: Color) -> Handle<StandardMaterial> {
        let key = color_key(color);
        if let Some(handle) = self.cache.unlit.get(&key) {
            return handle.clone();
        }
        let handle = self.materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        });
        self.cache.unlit.insert(key, handle.clone());
        handle
    }
}

/// Marks the child entities a chart generated, so a rebuild can clear them.
#[derive(Component, Debug, Clone, Copy)]
pub struct ChartElement;

/// Query filter matching charts of type `C` whose data or shared style changed.
pub(crate) type ChartChanged<C> = Or<(
    Changed<C>,
    Changed<ChartSize>,
    Changed<ChartPalette>,
    Changed<AxisStyle>,
)>;

/// Transform placing a unit cube as a box spanning `min..max`.
pub(crate) fn box_transform(min: Vec3, max: Vec3) -> Transform {
    Transform::from_translation((min + max) * 0.5)
        .with_scale((max - min).abs().max(Vec3::splat(1e-5)))
}

/// Transform placing a unit cube as a square-section bar from `start` to `end`.
///
/// Used for line segments: the cube's local z is stretched to the segment length
/// and rotated onto the segment direction.
pub(crate) fn segment_transform(start: Vec3, end: Vec3, thickness: f32) -> Transform {
    let delta = end - start;
    let length = delta.length();
    let mut transform = Transform::from_translation((start + end) * 0.5).with_scale(Vec3::new(
        thickness,
        thickness,
        length.max(1e-5),
    ));
    if let Ok(direction) = Dir3::new(delta) {
        // `looking_to` aims local -z; the sign does not matter for a symmetric box.
        transform.rotation = Transform::default().looking_to(direction, Vec3::Y).rotation;
    }
    transform
}
