mod map;
mod netcode;
mod replication;

use bevy::prelude::*;
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
        .add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / TICK_RATE_HZ),
        })
        // The protocol must be added after ServerPlugins but before any Server entity is
        // spawned (see shared/src/protocol/mod.rs) — SharedPlugin does both here.
        .add_plugins(SharedPlugin)
        .init_resource::<map::MapGenConfig>()
        .add_systems(Startup, (start_match, netcode::start_server))
        .add_systems(OnEnter(AppState::Game), map::generate_map_on_enter)
        .add_observer(replication::send_map_on_connect)
        .run();
}
