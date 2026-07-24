//! Animated height-field surface.
//!
//! Run with `cargo run --example surface_chart`.

use bevy::prelude::*;
use bevy_charts::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyChartsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, ripple))
        .run();
}

const FOCUS: Vec3 = Vec3::new(3.0, 1.0, 3.0);
const GRID: usize = 48;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        AmbientLight {
            brightness: 220.0,
            ..default()
        },
        Transform::from_xyz(9.0, 7.0, 9.0).looking_at(FOCUS, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 6_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 10.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        SurfaceChart3d::new(GRID, GRID, height_field(0.0)),
        ChartSize(Vec3::new(6.0, 2.0, 6.0)),
    ));
}

/// A radial ripple, sampled over the grid.
fn height_field(t: f32) -> Vec<f32> {
    (0..GRID * GRID)
        .map(|i| {
            let x = (i % GRID) as f32 / (GRID - 1) as f32 - 0.5;
            let z = (i / GRID) as f32 / (GRID - 1) as f32 - 0.5;
            let r = (x * x + z * z).sqrt();
            (r * 22.0 - t * 3.0).sin() * (1.0 - r * 1.4).max(0.0)
        })
        .collect()
}

/// Regenerates the height field a few times a second.
///
/// Each change rebuilds the surface mesh, so this is throttled rather than run
/// every frame.
fn ripple(time: Res<Time>, mut next: Local<f32>, mut chart: Single<&mut SurfaceChart3d>) {
    let t = time.elapsed_secs();
    if t < *next {
        return;
    }
    *next = t + 1.0 / 20.0;
    chart.heights = height_field(t);
}

fn orbit_camera(time: Res<Time>, mut camera: Single<&mut Transform, With<Camera3d>>) {
    let radius = 12.0;
    let angle = time.elapsed_secs() * 0.18;
    camera.translation = FOCUS + Vec3::new(angle.sin() * radius, 6.0, angle.cos() * radius);
    let look = camera.looking_at(FOCUS, Vec3::Y);
    camera.rotation = look.rotation;
}
