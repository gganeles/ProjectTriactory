mod mapgen;

use bevy::prelude::*;
use triactory_shared::{AppState, SharedPlugin};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(SharedPlugin)
        .init_resource::<mapgen::MapGenConfig>()
        .add_systems(OnEnter(AppState::Game), mapgen::generate_map_on_enter)
        .run();
}
