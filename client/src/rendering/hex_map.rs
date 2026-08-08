//! Renders the map replicated from the server, shown while [`AppState::Game`] is active.
//!
//! Spawns once `world_model::RevealedTiles` has data (the server sends the whole map as one
//! message right after connecting — see `server/src/replication.rs` — so there's a frame or two
//! where `AppState::Game` is active but nothing has arrived yet). Renders as a mosaic of
//! triangles whose corners are lifted per [`vertex_heights`] — shared between neighbors, so the
//! interior reads as one continuous, smoothly sloped surface with no visible tile-to-tile
//! seams — with a deep skirt wall dropped only around the map's outer boundary (see
//! [`build_tile_mesh`]), so the map as a whole still reads as a solid 3D slab. Viewed through an
//! angled orthographic camera — a "2.5D" look — rather than the top-down 2D camera + chunked
//! meshes planned in this folder's `README.md` for the eventual fog-aware, dirty-chunk-rebuilding
//! version.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use triactory_shared::{
    AppState,
    game::map::terrain::{SEA_LEVEL, TerrainType, TileData},
    grid::TriCoord,
};

use super::camera::MainCamera;
use crate::world_model::{BiomeTowers, RevealedTiles};

pub struct HexMapPlugin;

impl Plugin for HexMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_hex_map_when_ready, spawn_biome_tower_labels_when_ready)
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            draw_biome_tower_labels
                .run_if(in_state(AppState::Game))
                .run_if(resource_exists::<BiomeTowerLabels>),
        )
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

/// World-space anchor + text for every Biome Town's "BT"/"BT<slot>" label (see
/// [`spawn_biome_tower_labels_when_ready`]), drawn each frame by [`draw_biome_tower_labels`] via
/// an egui screen-space overlay — there's no world-space text rendering in this codebase (no
/// `Camera2d`/`Text2d` alongside the map's `Camera3d`), and the camera only pans/zooms (never
/// rotates) during `AppState::Game`, so re-projecting through `Camera::world_to_viewport` every
/// frame is simpler than maintaining a billboard mesh.
#[derive(Resource)]
struct BiomeTowerLabels {
    /// (world position, label text) pairs, one per placed Biome Town.
    entries: Vec<(Vec3, String)>,
}

/// Small vertical offset so labels float just above the tile surface instead of clipping into
/// it.
const LABEL_HEIGHT_OFFSET: f32 = 0.15;

const EDGE_LEN: f32 = 1.0;

/// World-space height per unit of `elevation` (`[0.0, 1.0]`, see `TileData`). Tuned so mountain
/// peaks stand well above sea level relative to `EDGE_LEN`-sized tiles without towering over the
/// map's radius.
const HEIGHT_SCALE: f32 = 0.5;

/// World height the map's outer boundary skirt drops to — deep enough that the whole map reads
/// as a solid slab floating above the void. Without it, the map's edge tiles are mostly
/// low-elevation ocean, whose height is barely above 0 — so the map just fades into the
/// background plane instead of reading as a distinct 3D object.
const SKIRT_BASE_HEIGHT: f32 = -1.0;

/// Fixed world height every corner on the map's outer boundary is flattened to (see
/// [`spawn_hex_map_when_ready`]), instead of its elevation-averaged [`vertex_heights`] value.
/// Elevation is noisy per-tile, so left alone the boundary ring's rim height zigzags all the way
/// around the map — barely noticeable from most angles, but the fixed camera/light angle used
/// here happens to expose it clearly on the far/lit sides. Flattening just the boundary corners
/// keeps the interior sloped naturally while making the silhouette a clean straight hexagon from
/// every angle. `SEA_LEVEL` (rather than 0) keeps the rim roughly level with typical coastline
/// height instead of sinking to the map's absolute floor.
const BOUNDARY_RIM_HEIGHT: f32 = SEA_LEVEL * HEIGHT_SCALE;

/// Quantizes a corner position into a hashable key so triangles that share a corner (in exact
/// world space, since `corners_world` is deterministic) agree on which [`vertex_heights`] entry
/// to read. `f32` isn't `Hash`/`Eq`, hence the scaled-and-rounded integer pair.
fn corner_key(point: Vec2) -> (i64, i64) {
    ((point.x * 1024.0).round() as i64, (point.y * 1024.0).round() as i64)
}

/// Averages elevation across every tile touching each corner point, so adjacent triangles meet
/// at the same height instead of leaving cracks or visible seams between independently-elevated
/// tiles — this is what turns per-tile elevation into a continuous, smoothly sloped interior
/// surface (the map's outer boundary gets its own distinct treatment — see [`build_tile_mesh`]).
fn vertex_heights(tiles: &[TriCoord], revealed: &RevealedTiles) -> HashMap<(i64, i64), f32> {
    let mut sums: HashMap<(i64, i64), (f32, u32)> = HashMap::new();
    for tile in tiles {
        let elevation = revealed.tiles[tile].elevation;
        for corner in tile.corners_world(EDGE_LEN) {
            let entry = sums.entry(corner_key(corner)).or_insert((0.0, 0));
            entry.0 += elevation;
            entry.1 += 1;
        }
    }
    sums.into_iter()
        .map(|(key, (sum, count))| (key, sum / count as f32 * HEIGHT_SCALE))
        .collect()
}

/// Builds one tile's mesh: a top face through `top_heights` (per-corner, see [`vertex_heights`])
/// plus — only for edges on the map's outer boundary (`edge_is_boundary[i]`, edge between
/// `corners[i]` and `corners[(i + 1) % 3]`, see [`tile_edge_is_boundary`]) — a rectangular wall
/// dropping to [`SKIRT_BASE_HEIGHT`]. Interior edges get no wall at all: since neighboring tiles
/// share the exact same corner heights, their top faces already meet seamlessly, so a wall there
/// would just be a stray zero-height sliver. Only the map's perimeter is meant to read as a
/// cliff.
fn build_tile_mesh(corners: [Vec2; 3], top_heights: [f32; 3], edge_is_boundary: [bool; 3]) -> Mesh {
    let top: [Vec3; 3] = std::array::from_fn(|i| to_world_3d(corners[i], top_heights[i]));
    let centroid = (top[0] + top[1] + top[2]) / 3.0;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(15);

    // Top face: reversed order turns the 2D-plane's CCW winding (CW once lifted into the Y-up
    // 3D world) back into an upward-facing normal — see `to_world_3d`'s docs.
    push_triangle(&mut positions, top[2], top[1], top[0]);

    // Boundary walls only: one quad (as two triangles) per edge, connecting the top edge down to
    // `SKIRT_BASE_HEIGHT`. Winding is picked (and flipped if needed) so the wall's flat normal
    // points away from the tile's centroid — i.e. outward — which works for both the grid's
    // upward- and downward-pointing triangles without special-casing either.
    for i in 0..3 {
        if !edge_is_boundary[i] {
            continue;
        }
        let j = (i + 1) % 3;
        let bottom_i = to_world_3d(corners[i], SKIRT_BASE_HEIGHT);
        let bottom_j = to_world_3d(corners[j], SKIRT_BASE_HEIGHT);
        let mut quad = [top[i], top[j], bottom_j, bottom_i];
        let normal = (quad[1] - quad[0]).cross(quad[2] - quad[0]);
        let outward = (top[i] + top[j]) / 2.0 - centroid;
        if normal.dot(outward) < 0.0 {
            quad.reverse();
        }
        push_triangle(&mut positions, quad[0], quad[1], quad[2]);
        push_triangle(&mut positions, quad[0], quad[2], quad[3]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_computed_flat_normals()
}

/// Whether `tile`'s edge between `corners[i]` and `corners[(i + 1) % 3]` lies on the map's outer
/// boundary — i.e. no other revealed tile shares it — for each of its 3 edges. Matches
/// `edge_neighbors()` candidates back to a specific edge by checking which 2 of the *neighbor's*
/// corners coincide with this tile's `corners[i]`/`corners[j]`, rather than hand-deriving the
/// index correspondence between `corners_world`'s and `edge_neighbors`' orderings.
fn tile_edge_is_boundary(
    tile: TriCoord,
    corners: [Vec2; 3],
    tiles: &HashMap<TriCoord, TileData>,
) -> [bool; 3] {
    let neighbors = tile.edge_neighbors();
    core::array::from_fn(|i| {
        let j = (i + 1) % 3;
        !neighbors.iter().any(|neighbor| {
            tiles.contains_key(neighbor) && {
                let neighbor_corners = neighbor.corners_world(EDGE_LEN);
                corners_contain(&neighbor_corners, corners[i])
                    && corners_contain(&neighbor_corners, corners[j])
            }
        })
    })
}

/// Whether `point` matches (within float noise) one of `corners` — all derived from the same
/// deterministic `corners_world` formula, so shared corners agree exactly but for rounding.
fn corners_contain(corners: &[Vec2; 3], point: Vec2) -> bool {
    const EPSILON: f32 = 1e-4;
    corners.iter().any(|c| c.distance_squared(point) < EPSILON * EPSILON)
}

fn push_triangle(positions: &mut Vec<[f32; 3]>, a: Vec3, b: Vec3, c: Vec3) {
    positions.push(a.into());
    positions.push(b.into());
    positions.push(c.into());
}

fn spawn_hex_map_when_ready(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    revealed: Res<RevealedTiles>,
    existing: Query<(), With<HexMapScene>>,
    mut camera: Single<(&mut Transform, &mut Projection), With<MainCamera>>,
) {
    if !existing.is_empty() || revealed.tiles.is_empty() {
        return;
    }
    let tiles: Vec<TriCoord> = revealed.tiles.keys().copied().collect();
    let mut heights = vertex_heights(&tiles, &revealed);

    // Every tile's per-edge boundary flags, computed once so the corner-flattening pass below
    // and the mesh-building loop further down (which needs the same flags for its skirt wall)
    // agree and don't redo the neighbor lookup twice.
    let edge_is_boundary: HashMap<TriCoord, [bool; 3]> = tiles
        .iter()
        .map(|&tile| {
            let corners = tile.corners_world(EDGE_LEN);
            (tile, tile_edge_is_boundary(tile, corners, &revealed.tiles))
        })
        .collect();

    // Flatten every corner on the map's outer boundary to one fixed height (see
    // `BOUNDARY_RIM_HEIGHT`'s docs) — interior corners keep their elevation-averaged height, so
    // only the map's silhouette is affected, not its interior slope.
    for &tile in &tiles {
        let corners = tile.corners_world(EDGE_LEN);
        for (i, &is_boundary) in edge_is_boundary[&tile].iter().enumerate() {
            if !is_boundary {
                continue;
            }
            let j = (i + 1) % 3;
            heights.insert(corner_key(corners[i]), BOUNDARY_RIM_HEIGHT);
            heights.insert(corner_key(corners[j]), BOUNDARY_RIM_HEIGHT);
        }
    }

    // One material per biome, built lazily so unused `TerrainType`s never allocate a handle.
    let mut biome_materials: HashMap<TerrainType, Handle<StandardMaterial>> = HashMap::new();

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
                let corners = tile.corners_world(EDGE_LEN);
                let top_heights = corners.map(|c| heights[&corner_key(c)]);
                let terrain_type = revealed.tiles[tile].terrain_type;
                let material = biome_materials
                    .entry(terrain_type)
                    .or_insert_with(|| {
                        materials.add(StandardMaterial {
                            // Tiles are lifted per-vertex for the 2.5D relief look, so PBR
                            // lighting would otherwise shade each triangle by its own slope —
                            // same terrain type, visibly different shades. Unlit keeps the
                            // flat, solid per-type color the art style requires regardless of
                            // slope or light angle.
                            unlit: true,
                            ..StandardMaterial::from(terrain_type.color())
                        })
                    })
                    .clone();
                parent.spawn((
                    Mesh3d(meshes.add(build_tile_mesh(corners, top_heights, edge_is_boundary[tile]))),
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

/// Computes every Biome Town's label once `RevealedTiles` and `BiomeTowers` have both arrived
/// (they're sent as two separate messages — see `send_map_state` — so they can land a frame or
/// two apart). Independent of [`spawn_hex_map_when_ready`]'s own readiness check so neither
/// system's data blocks the other's.
fn spawn_biome_tower_labels_when_ready(
    mut commands: Commands,
    revealed: Res<RevealedTiles>,
    towers: Res<BiomeTowers>,
    existing: Option<Res<BiomeTowerLabels>>,
) {
    if existing.is_some() || revealed.tiles.is_empty() || towers.towns.is_empty() {
        return;
    }

    let starting_slot: HashMap<TriCoord, usize> = towers
        .starting_towers
        .iter()
        .enumerate()
        .map(|(slot, &tile)| (tile, slot))
        .collect();

    let entries = towers
        .towns
        .iter()
        .filter_map(|town| {
            let elevation = revealed.tiles.get(town)?.elevation;
            let height = elevation * HEIGHT_SCALE + LABEL_HEIGHT_OFFSET;
            let pos = to_world_3d(town.center_world(EDGE_LEN), height);
            let text = match starting_slot.get(town) {
                Some(slot) => format!("BT{slot}"),
                None => "BT".to_string(),
            };
            Some((pos, text))
        })
        .collect();

    commands.insert_resource(BiomeTowerLabels { entries });
}

/// Draws each label from [`BiomeTowerLabels`] at its Biome Town's current screen position, via
/// `Camera::world_to_viewport` — see [`BiomeTowerLabels`]'s docs for why this is a per-frame
/// egui overlay rather than world-space geometry.
fn draw_biome_tower_labels(
    mut contexts: EguiContexts,
    labels: Res<BiomeTowerLabels>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let (camera, camera_transform) = *camera;
    let painter = ctx.debug_painter();
    for (pos, text) in &labels.entries {
        if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, *pos) {
            painter.text(
                egui::pos2(screen_pos.x, screen_pos.y),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(16.0),
                egui::Color32::WHITE,
            );
        }
    }
    Ok(())
}

fn despawn_hex_map(mut commands: Commands, roots: Query<Entity, With<HexMapScene>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<MapBounds>();
    commands.remove_resource::<BiomeTowerLabels>();
}

/// Places a `center_world`/`corners_world` 2D point into the Y-up 3D scene, at world height
/// `height` rather than pinned to the XZ plane.
fn to_world_3d(point: Vec2, height: f32) -> Vec3 {
    Vec3::new(point.x, height, point.y)
}
