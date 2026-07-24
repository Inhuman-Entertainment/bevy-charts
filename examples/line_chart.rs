//! Line chart with two series, updating live.
//!
//! Also demonstrates that mutating the chart component is all it takes to
//! redraw — the build system picks the change up on its own.
//!
//! Run with `cargo run --example line_chart`.

use bevy::prelude::*;
use bevy_charts::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyChartsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, push_samples))
        .run();
}

const FOCUS: Vec3 = Vec3::new(3.0, 1.5, 1.0);

/// How many points stay on screen before the oldest scrolls off.
const WINDOW: usize = 24;

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
        .with_dataset(Dataset::new("Frame time", Vec::new()))
        .with_dataset(Dataset::new("Draw calls", Vec::new()));

    commands.spawn((
        LineChart3d {
            data,
            begin_at_zero: true,
            ..default()
        },
        ChartSize(Vec3::new(6.0, 3.0, 2.0)),
    ));
}

/// Appends a sample to each series a few times a second, dropping the oldest
/// once the window is full.
///
/// Deliberately not every frame: each change rebuilds the chart's geometry, and
/// a sampled chart is what you would actually want anyway.
fn push_samples(
    time: Res<Time>,
    mut next_sample: Local<f32>,
    mut chart: Single<&mut LineChart3d>,
) {
    let t = time.elapsed_secs();
    if t < *next_sample {
        return;
    }
    *next_sample = t + 0.2;

    let samples = [
        16.0 + (t * 1.7).sin() * 4.0 + (t * 5.3).sin() * 1.5,
        40.0 + (t * 0.9).cos() * 12.0,
    ];

    for (dataset, sample) in chart.data.datasets.iter_mut().zip(samples) {
        dataset.data.push(sample);
        if dataset.data.len() > WINDOW {
            dataset.data.remove(0);
        }
    }
}

fn orbit_camera(time: Res<Time>, mut camera: Single<&mut Transform, With<Camera3d>>) {
    let radius = 12.0;
    let angle = time.elapsed_secs() * 0.2;
    camera.translation = FOCUS + Vec3::new(angle.sin() * radius, 5.0, angle.cos() * radius);
    let look = camera.looking_at(FOCUS, Vec3::Y);
    camera.rotation = look.rotation;
}
