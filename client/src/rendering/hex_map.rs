//! Local preview of the hexagonal triangle map, shown while [`AppState::Game`] is active.
//!
//! There's no netcode yet, so this generates the same [`hexagon_tiles`] shape the server will
//! authoritatively build, purely for a visual check that the map generation looks right. It
//! renders as a flat mosaic of triangles viewed through an angled orthographic camera — a "2.5D"
//! look — rather than the top-down 2D camera + chunked meshes planned in this folder's
//! `README.md` for the real, replicated map.

use bevy::prelude::*;
use triactory_shared::{AppState, config::DEFAULT_EDGE_TILES, grid::hexagon_tiles};

use super::camera::MainCamera;

pub struct HexMapPlugin;

impl Plugin for HexMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), spawn_hex_map)
            .add_systems(OnExit(AppState::Game), despawn_hex_map);
    }
}

/// Marks the map preview's tile/light entities, so they can be despawned on leaving
/// `AppState::Game`. The camera is not part of this scene — it's the persistent one from
/// [`super::camera`], just repositioned to frame the map.
#[derive(Component)]
struct HexMapScene;

/// The map's radial extent from the origin (distance to its farthest tile corner), in world
/// units. Read by `input::pan` and `input::zoom` to keep the camera within reach of the board
/// instead of drifting or zooming arbitrarily far from it.
#[derive(Resource)]
pub struct MapBounds {
    pub radius: f32,
    /// The camera's XZ translation when framing the map dead-center, i.e. with zero pan applied.
    /// Panning is measured relative to *this*, not the world origin — the camera's resting
    /// translation is already offset in Z to get its angled "2.5D" tilt, which isn't panning.
    pub camera_home_xz: Vec2,
}

const EDGE_LEN: f32 = 1.0;

fn spawn_hex_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera: Single<(&mut Transform, &mut Projection), With<MainCamera>>,
) {
    let tiles =
        hexagon_tiles(DEFAULT_EDGE_TILES).expect("DEFAULT_EDGE_TILES must be a valid hexagon size");

    let up_material = materials.add(StandardMaterial::from(Color::srgb(0.40, 0.62, 0.36)));
    let down_material = materials.add(StandardMaterial::from(Color::srgb(0.28, 0.46, 0.26)));

    let mut max_dist = 0.0f32;
    for tile in &tiles {
        for corner in tile.corners_world(EDGE_LEN) {
            max_dist = max_dist.max(corner.length());
        }
    }
    let view_diameter = max_dist * 2.2;

    commands
        .spawn((HexMapScene, Transform::default(), Visibility::default()))
        .with_children(|parent| {
            for tile in &tiles {
                let [a, b, c] = tile.corners_world(EDGE_LEN);
                // Reversed order: a 2D-plane CCW winding becomes CW (facing down) once lifted
                // into the Y-up 3D world, so this flips it back to face up.
                let triangle = Triangle3d::new(to_world_3d(c), to_world_3d(b), to_world_3d(a));
                let material = if tile.is_upward() {
                    up_material.clone()
                } else {
                    down_material.clone()
                };
                parent.spawn((
                    Mesh3d(meshes.add(triangle.mesh().build())),
                    MeshMaterial3d(material),
                ));
            }

            parent.spawn((
                DirectionalLight {
                    illuminance: 6_000.0,
                    shadow_maps_enabled: true,
                    ..default()
                },
                Transform::from_xyz(view_diameter * 0.4, view_diameter, view_diameter * 0.3)
                    .looking_at(Vec3::ZERO, Vec3::Y),
            ));
        });

    *camera.0 = Transform::from_xyz(0.0, view_diameter * 0.9, view_diameter * 0.9)
        .looking_at(Vec3::ZERO, Vec3::Y);
    *camera.1 = Projection::Orthographic(OrthographicProjection {
        scaling_mode: bevy::camera::ScalingMode::AutoMin {
            min_width: view_diameter,
            min_height: view_diameter,
        },
        ..OrthographicProjection::default_3d()
    });

    commands.insert_resource(MapBounds {
        radius: max_dist,
        camera_home_xz: Vec2::new(camera.0.translation.x, camera.0.translation.z),
    });
}

fn despawn_hex_map(mut commands: Commands, roots: Query<Entity, With<HexMapScene>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<MapBounds>();
}

/// Lifts a `center_world`/`corners_world` 2D point onto the XZ plane (Y up) of the 3D scene.
fn to_world_3d(point: Vec2) -> Vec3 {
    Vec3::new(point.x, 0.0, point.y)
}
