mod game;
mod netcode;
mod replication;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use core::time::Duration;
use lightyear::prelude::server::ServerPlugins;
use triactory_shared::config::TICK_RATE_HZ;
use triactory_shared::{AppState, SharedPlugin};

/// The server is headless and has no menu — it's always "in the match", unlike the client which
/// waits in `AppState::MainMenu` until the player taps Start.
fn start_match(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Game);
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        // MinimalPlugins doesn't include logging (unlike the client's DefaultPlugins) — without
        // this, a headless server would run with no visibility at all, `RUST_LOG` included.
        .add_plugins(bevy::log::LogPlugin::default())
        // Lightyear's ServerPlugins registers its own internal Bevy `States` type and expects
        // `StatesPlugin` to already exist — `MinimalPlugins` doesn't include it (unlike the
        // client's `DefaultPlugins`), so it has to go first here.
        .add_plugins(StatesPlugin)
        .add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / TICK_RATE_HZ),
        })
        // The protocol must be added after ServerPlugins but before any Server entity is
        // spawned (see shared/src/protocol/mod.rs) — SharedPlugin does both here.
        .add_plugins(SharedPlugin)
        .init_resource::<game::map::MapGenConfig>()
        .add_systems(Startup, (start_match, netcode::start_server))
        .add_systems(OnEnter(AppState::Game), game::map::generate_map_on_enter)
        .add_observer(replication::send_map_on_connect)
        .run();
}
