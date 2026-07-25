//! Drag-to-pan camera control, active only during `AppState::Game` while `GameMode::BuildMode`
//! is active.
//!
//! There's only one gesture so far (no tap-to-move, no connection dragging yet), so this reads
//! touches/mouse directly rather than going through a full gesture classifier — see this
//! folder's `README.md`. Worth introducing `touch.rs`'s classifier once a second gesture needs
//! disambiguating against this one.

use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use triactory_shared::{AppState, GameMode};

use crate::rendering::camera::MainCamera;
use crate::rendering::hex_map::MapBounds;

/// How far past the map's own radius the camera is allowed to pan, as a multiple of it. Scales
/// with the current zoom (`OrthographicProjection::scale`) so more of the world stays reachable
/// when zoomed out, and the limit tightens up when zoomed in.
const PAN_LIMIT_FACTOR: f32 = 1.0;

pub struct PanPlugin;

impl Plugin for PanPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>().add_systems(
            Update,
            pan_camera_on_drag
                .run_if(in_state(AppState::Game))
                .run_if(in_state(GameMode::BuildMode))
                // The map now spawns once the server's TilesRevealed message arrives rather than
                // synchronously on entering `AppState::Game`, so there's a window where the state
                // is right but `MapBounds` (inserted alongside the map) doesn't exist yet.
                .run_if(resource_exists::<MapBounds>),
        );
    }
}

/// The screen-space position of the drag last frame, so this frame can compute how far it moved.
#[derive(Resource, Default)]
struct DragState {
    last_screen_pos: Option<Vec2>,
}

fn pan_camera_on_drag(
    mut drag: ResMut<DragState>,
    touches: Res<Touches>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    bounds: Res<MapBounds>,
    mut camera: Single<(&Camera, &GlobalTransform, &mut Transform, &Projection), With<MainCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let current_pos = touches.first_pressed_position().or_else(|| {
        if mouse_buttons.pressed(MouseButton::Left) {
            window.cursor_position()
        } else {
            None
        }
    });

    let Some(current_pos) = current_pos else {
        drag.last_screen_pos = None;
        return;
    };

    if let Some(last_pos) = drag.last_screen_pos {
        let world_last = ground_point(camera.0, camera.1, last_pos);
        let world_current = ground_point(camera.0, camera.1, current_pos);
        if let (Some(world_last), Some(world_current)) = (world_last, world_current) {
            // Moves the camera so the world point that was under the finger last frame is under
            // it again this frame — the map follows the finger instead of panning at a fixed rate.
            camera.2.translation += world_last - world_current;
        }
    }

    drag.last_screen_pos = Some(current_pos);

    let zoom_scale = match camera.3 {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0,
    };
    let max_pan = bounds.radius * PAN_LIMIT_FACTOR * zoom_scale;
    let current_xz = Vec2::new(camera.2.translation.x, camera.2.translation.z);
    let clamped_offset = (current_xz - bounds.camera_home_xz).clamp_length_max(max_pan);
    let clamped_xz = bounds.camera_home_xz + clamped_offset;
    camera.2.translation.x = clamped_xz.x;
    camera.2.translation.z = clamped_xz.y;
}

/// Where a screen-space point lands on the map's Y=0 ground plane, or `None` if the camera ray
/// is (near-)parallel to it.
fn ground_point(camera: &Camera, camera_transform: &GlobalTransform, screen_pos: Vec2) -> Option<Vec3> {
    let ray = camera.viewport_to_world(camera_transform, screen_pos).ok()?;
    if ray.direction.y.abs() < 1e-5 {
        return None;
    }
    let t = -ray.origin.y / ray.direction.y;
    (t > 0.0).then(|| ray.origin + ray.direction * t)
}
