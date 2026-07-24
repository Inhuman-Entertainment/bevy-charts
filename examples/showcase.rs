//! All five chart types in one scene.
//!
//! Run with `cargo run --example showcase`.

use bevy::prelude::*;
use bevy_charts::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, BevyChartsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_camera_and_light)
        .run();
}

/// Roughly the middle of the group of charts.
const FOCUS: Vec3 = Vec3::new(9.0, 1.0, 4.0);
const GRID: usize = 40;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        AmbientLight {
            brightness: 300.0,
            ..default()
        },
        Transform::from_xyz(11.0, 16.0, 24.0).looking_at(FOCUS, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 6_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 14.0, 10.0).looking_at(FOCUS, Vec3::Y),
    ));

    // Something for the shadows to land on. Without it the charts cast only
    // onto each other and the moving light reads as nothing but a brightness
    // change. Sunk slightly so it does not z-fight the charts' base axes.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(48.0, 48.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.16, 0.17),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(FOCUS.x, -0.05, FOCUS.z),
    ));

    let bars = ChartData::new()
        .with_labels(["Q1", "Q2", "Q3", "Q4"])
        .with_dataset(Dataset::new("Desktop", vec![12.0, 19.0, 7.0, 15.0]))
        .with_dataset(Dataset::new("Mobile", vec![8.0, 14.0, 17.0, 11.0]))
        .with_dataset(Dataset::new("Tablet", vec![3.0, 5.0, 4.0, 9.0]));
    commands.spawn((
        BarChart3d::new(bars),
        ChartSize(Vec3::new(5.0, 3.0, 3.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let lines = ChartData::new()
        .with_dataset(Dataset::new(
            "A",
            (0..20)
                .map(|i| (i as f32 * 0.4).sin() * 5.0 + 8.0)
                .collect::<Vec<_>>(),
        ))
        .with_dataset(Dataset::new(
            "B",
            (0..20)
                .map(|i| (i as f32 * 0.3).cos() * 3.0 + 5.0)
                .collect::<Vec<_>>(),
        ));
    commands.spawn((
        LineChart3d {
            data: lines,
            begin_at_zero: true,
            ..default()
        },
        ChartSize(Vec3::new(5.0, 3.0, 2.0)),
        Transform::from_xyz(7.0, 0.0, 0.0),
    ));

    // A cheap deterministic hash, so the example needs no rng dependency and
    // looks the same every run.
    let noise = |n: u32| {
        let h = n.wrapping_mul(2_654_435_761) ^ (n >> 13);
        (h % 1000) as f32 / 1000.0 - 0.5
    };
    let clusters = [
        ("A", Vec3::new(-1.5, 0.5, -1.0)),
        ("B", Vec3::new(1.0, 2.0, 1.5)),
        ("C", Vec3::new(0.0, -1.0, 2.0)),
    ];
    let scatter = clusters
        .iter()
        .enumerate()
        .map(|(s, (label, center))| {
            PointDataset::new(
                *label,
                (0..50)
                    .map(|i| {
                        let seed = (s as u32 * 1000 + i) * 3;
                        *center + Vec3::new(noise(seed), noise(seed + 1), noise(seed + 2)) * 2.0
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    commands.spawn((
        ScatterChart3d::new(scatter),
        ChartSize(Vec3::new(4.0, 3.0, 3.0)),
        Transform::from_xyz(14.0, 0.0, 0.0),
    ));

    let heights = (0..GRID * GRID)
        .map(|i| {
            let x = (i % GRID) as f32 / (GRID - 1) as f32 - 0.5;
            let z = (i / GRID) as f32 / (GRID - 1) as f32 - 0.5;
            let r = (x * x + z * z).sqrt();
            (r * 20.0).sin() * (1.0 - r * 1.4).max(0.0)
        })
        .collect::<Vec<_>>();
    commands.spawn((
        SurfaceChart3d::new(GRID, GRID, heights),
        ChartSize(Vec3::new(5.0, 2.0, 5.0)),
        Transform::from_xyz(0.0, 0.0, 6.0),
    ));

    let samples: Vec<f32> = (0..400)
        .map(|i| {
            let x = i as f32 / 400.0 * 6.0 - 3.0;
            x + (i as f32 * 0.7).sin()
        })
        .collect();
    commands.spawn((
        HistogramChart3d::new(vec![Dataset::new("Samples", samples)]).with_bins(12),
        ChartSize(Vec3::new(5.0, 3.0, 1.5)),
        Transform::from_xyz(8.0, 0.0, 7.0),
    ));
}

/// How far ahead of the camera the light orbits, in radians.
///
/// Zero would put the light directly behind the viewer, which is the one angle
/// where shadows are invisible — every shadow would fall exactly behind the
/// thing casting it. A third of a turn keeps them raking across the scene.
const LIGHT_LEAD: f32 = std::f32::consts::FRAC_PI_3;

/// Circles the camera, with the light tracking it a fixed angle ahead so the
/// shadows sweep as the scene turns.
fn orbit_camera_and_light(
    time: Res<Time>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<DirectionalLight>)>,
    mut light: Single<&mut Transform, (With<DirectionalLight>, Without<Camera3d>)>,
) {
    let angle = time.elapsed_secs() * 0.15;

    let radius = 22.0;
    camera.translation = FOCUS + Vec3::new(angle.sin() * radius, 15.0, angle.cos() * radius);
    camera.rotation = camera.looking_at(FOCUS, Vec3::Y).rotation;

    // Only a directional light's rotation affects the scene, but keeping its
    // translation on a matching orbit makes the intent obvious when reading it.
    let lit = angle + LIGHT_LEAD;
    light.translation = FOCUS + Vec3::new(lit.sin() * 18.0, 14.0, lit.cos() * 18.0);
    light.rotation = light.looking_at(FOCUS, Vec3::Y).rotation;
}
