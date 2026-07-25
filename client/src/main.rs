mod input;
mod netcode;
mod rendering;
mod ui;
mod world_model;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use core::time::Duration;
use lightyear::prelude::client::ClientPlugins;
use triactory_shared::SharedPlugin;
use triactory_shared::config::TICK_RATE_HZ;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(ClientPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / TICK_RATE_HZ),
        })
        // The protocol must be added after ClientPlugins but before any Client entity is
        // spawned (see shared/src/protocol/mod.rs) — SharedPlugin does both here.
        .add_plugins(SharedPlugin)
        .add_systems(
            Startup,
            (rendering::camera::spawn_main_camera, netcode::connect_to_server),
        )
        .add_plugins(world_model::WorldModelPlugin)
        .add_plugins(ui::main_menu::MainMenuPlugin)
        .add_plugins(ui::mode_toggle::ModeTogglePlugin)
        .add_plugins(rendering::hex_map::HexMapPlugin)
        .add_plugins(input::pan::PanPlugin)
        .add_plugins(input::zoom::ZoomPlugin)
        .run();
}
