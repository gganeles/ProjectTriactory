pub mod config;
pub mod game;
pub mod grid;
pub mod protocol;
pub mod states;

pub use states::{AppState, GameMode};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

/// Configuration added by **both** the server and client apps: registers [`AppState`] /
/// [`GameMode`], the Lightyear wire contract (`protocol::ProtocolPlugin`), and the shared
/// game domain (`game::GamePlugin`). Must be added after `ClientPlugins`/`ServerPlugins` in
/// each app's `main.rs`.
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StatesPlugin>() {
            app.add_plugins(StatesPlugin);
        }
        app.init_state::<AppState>().add_sub_state::<GameMode>();
        app.add_plugins(protocol::ProtocolPlugin);
        app.add_plugins(game::GamePlugin);
    }
}
