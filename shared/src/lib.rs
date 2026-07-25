pub mod components;
pub mod config;
pub mod grid;
pub mod states;

pub use states::{AppState, GameMode};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

/// Configuration added by **both** the server and client apps: currently just registers
/// [`AppState`] and [`GameMode`]. Will grow to include the 30 Hz fixed timestep, `ProtocolPlugin`,
/// and shared simulation systems (see `shared/src/README.md`).
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StatesPlugin>() {
            app.add_plugins(StatesPlugin);
        }
        app.init_state::<AppState>().add_sub_state::<GameMode>();
    }
}
