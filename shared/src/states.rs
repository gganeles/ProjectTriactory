//! Top-level application state, shared by both apps so state transitions stay in lockstep.
//!
//! The client shows menu UI while in [`AppState::MainMenu`] and the match UI in
//! [`AppState::Game`]. On the server, entering [`AppState::Game`] is what triggers map
//! generation (see `server/src/map`). More granular in-match phases (lobby, starting,
//! playing, ended) are a separate, later concern — see `server/src/README.md`'s planned
//! `match_state.rs`.

use bevy::prelude::{States, SubStates};

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    Game,
}

/// The current input mode while in [`AppState::Game`]. Only exists while `AppState::Game` is
/// active (a [`SubStates`]) — it's meaningless during `MainMenu`.
///
/// - `BuildMode`: press-and-drag pans the camera over the map.
/// - `PlayMode`: a joystick drives the player's hero. Not implemented yet — the mode exists so
///   the drag-to-pan system (and, later, the joystick) can be gated on it.
#[derive(SubStates, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[source(AppState = AppState::Game)]
pub enum GameMode {
    #[default]
    BuildMode,
    PlayMode,
}
