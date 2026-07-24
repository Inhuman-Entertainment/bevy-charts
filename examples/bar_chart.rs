//! Grouped bar chart with three series.
//!
//! Run with `cargo run --example bar_chart`.

use bevy::prelude::*;
use bevy_charts::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyChartsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_camera)
        .run();
}

/// The point the camera circles, roughly the middle of the chart.
const FOCUS: Vec3 = Vec3::new(3.0, 1.5, 1.5);

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        AmbientLight {
            brightness: 220.0,
            ..default()
        },
        Transform::from_xyz(9.0, 6.0, 10.0).looking_at(FOCUS, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 6_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 10.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let data = ChartData::new()
        .with_labels(["Q1", "Q2", "Q3", "Q4"])
        .with_dataset(Dataset::new("Desktop", vec![12.0, 19.0, 7.0, 15.0]))
        .with_dataset(Dataset::new("Mobile", vec![8.0, 14.0, 17.0, 11.0]))
        .with_dataset(Dataset::new("Tablet", vec![3.0, 5.0, 4.0, 9.0]));

    commands.spawn((
        BarChart3d::new(data),
        ChartSize(Vec3::new(6.0, 3.0, 3.0)),
    ));
}

/// Slowly circles the camera, because a 3D chart read from one fixed angle is
/// just a 2D chart with extra steps.
fn orbit_camera(time: Res<Time>, mut camera: Single<&mut Transform, With<Camera3d>>) {
    let radius = 13.0;
    let angle = time.elapsed_secs() * 0.25;
    camera.translation = FOCUS + Vec3::new(angle.sin() * radius, 6.0, angle.cos() * radius);
    let look = camera.looking_at(FOCUS, Vec3::Y);
    camera.rotation = look.rotation;
}
