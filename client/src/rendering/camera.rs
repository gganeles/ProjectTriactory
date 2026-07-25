//! The single persistent camera. Spawned once at startup (`bevy_egui` binds its UI context to
//! the first camera the app creates) and repositioned by whichever scene is currently active —
//! see [`super::hex_map`] for the `AppState::Game` framing.

use bevy::prelude::*;

/// Marks the app's one camera, so scenes can find and reposition it instead of spawning their
/// own (spawning a second camera would need explicit ordering/clear-color setup to layer
/// correctly with the egui UI camera).
#[derive(Component)]
pub struct MainCamera;

pub fn spawn_main_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
