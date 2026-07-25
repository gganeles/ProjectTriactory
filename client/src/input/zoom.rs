//! Pinch-to-zoom (mobile touchscreen + macOS/iOS trackpad) and scroll-wheel zoom (desktop mouse)
//! for the map camera.
//!
//! Active any time `AppState::Game` is active — zooming isn't specific to `BuildMode`/`PlayMode`
//! the way the drag gesture in `pan.rs` is, so this doesn't gate on `GameMode` at all.

use bevy::input::gestures::PinchGesture;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::input::touch::Touches;
use bevy::prelude::*;
use triactory_shared::AppState;

use crate::rendering::camera::MainCamera;

pub struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PinchState>()
            .add_systems(Update, zoom_camera.run_if(in_state(AppState::Game)));
    }
}

/// Zoom limits, as `OrthographicProjection::scale` values relative to the default of 1.0 set
/// when the map is framed (see `rendering::hex_map`). Scale and apparent size are inversely
/// related, so `MIN` is the most zoomed-in and `MAX` the most zoomed-out the camera can go.
const MIN_SCALE: f32 = 0.5;
const MAX_SCALE: f32 = 2.0;
const SCROLL_ZOOM_SPEED: f32 = 0.1;
const PINCH_ZOOM_SPEED: f32 = 1.5;
const TRACKPAD_PINCH_ZOOM_SPEED: f32 = 2.0;

/// The distance between the first two active touches last frame, so pinch can compute how much
/// it changed this frame rather than the raw distance.
#[derive(Resource, Default)]
struct PinchState {
    last_distance: Option<f32>,
}

fn zoom_camera(
    mut pinch: ResMut<PinchState>,
    touches: Res<Touches>,
    scroll: Res<AccumulatedMouseScroll>,
    mut trackpad_pinch: MessageReader<PinchGesture>,
    mut projection: Single<&mut Projection, With<MainCamera>>,
) {
    let Projection::Orthographic(ortho) = &mut **projection else {
        return;
    };

    // Scrolling "up"/away zooms in, so this shrinks scale.
    let mut zoom_delta = -scroll.delta.y * SCROLL_ZOOM_SPEED;

    // macOS/iOS trackpad pinch: a distinct native gesture from two-finger scrolling, not
    // reported through `MouseWheel` at all. Positive `PinchGesture` values are fingers
    // spreading apart (zooming in), so this shrinks scale too.
    for event in trackpad_pinch.read() {
        zoom_delta -= event.0 * TRACKPAD_PINCH_ZOOM_SPEED;
    }

    let active: Vec<Vec2> = touches.iter().map(|t| t.position()).take(2).collect();
    if active.len() == 2 {
        let distance = active[0].distance(active[1]);
        if let Some(last_distance) = pinch.last_distance
            && last_distance > 1.0
        {
            // Fingers moving apart (distance growing) zooms in, so this shrinks scale too.
            zoom_delta -= (distance - last_distance) / last_distance * PINCH_ZOOM_SPEED;
        }
        pinch.last_distance = Some(distance);
    } else {
        pinch.last_distance = None;
    }

    ortho.scale = (ortho.scale + zoom_delta).clamp(MIN_SCALE, MAX_SCALE);
}
