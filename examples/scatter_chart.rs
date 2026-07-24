//! 3D scatter plot of three clusters.
//!
//! Three series is the deliberate ceiling here: scatter marks are compared
//! pairwise rather than in sequence, and the palette only guarantees pairwise
//! separation for its first three slots.
//!
//! Run with `cargo run --example scatter_chart`.

use bevy::prelude::*;
use bevy_charts::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyChartsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_camera)
        .run();
}

const FOCUS: Vec3 = Vec3::new(2.0, 1.5, 2.0);

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        AmbientLight {
            brightness: 220.0,
            ..default()
        },
        Transform::from_xyz(8.0, 6.0, 9.0).looking_at(FOCUS, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 6_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 10.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // A cheap deterministic hash, so the example needs no rng dependency and
    // looks the same every run.
    let noise = |n: u32| {
        let h = n.wrapping_mul(2_654_435_761) ^ (n >> 13);
        (h % 1000) as f32 / 1000.0 - 0.5
    };

    let clusters = [
        ("Cluster A", Vec3::new(-1.5, 0.5, -1.0)),
        ("Cluster B", Vec3::new(1.0, 2.0, 1.5)),
        ("Cluster C", Vec3::new(0.0, -1.0, 2.0)),
    ];

    let datasets = clusters
        .iter()
        .enumerate()
        .map(|(series, (label, center))| {
            let points = (0..60)
                .map(|i| {
                    let seed = (series as u32 * 1000 + i) * 3;
                    *center + Vec3::new(noise(seed), noise(seed + 1), noise(seed + 2)) * 2.0
                })
                .collect::<Vec<_>>();
            PointDataset::new(*label, points)
        })
        .collect::<Vec<_>>();

    commands.spawn((
        ScatterChart3d::new(datasets),
        ChartSize(Vec3::new(4.0, 3.0, 4.0)),
    ));
}

fn orbit_camera(time: Res<Time>, mut camera: Single<&mut Transform, With<Camera3d>>) {
    let radius = 11.0;
    let angle = time.elapsed_secs() * 0.22;
    camera.translation = FOCUS + Vec3::new(angle.sin() * radius, 5.5, angle.cos() * radius);
    let look = camera.looking_at(FOCUS, Vec3::Y);
    camera.rotation = look.rotation;
}
