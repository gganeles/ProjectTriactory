mod input;
mod rendering;
mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use triactory_shared::SharedPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(SharedPlugin)
        .add_systems(Startup, rendering::camera::spawn_main_camera)
        .add_plugins(ui::main_menu::MainMenuPlugin)
        .add_plugins(ui::mode_toggle::ModeTogglePlugin)
        .add_plugins(rendering::hex_map::HexMapPlugin)
        .add_plugins(input::pan::PanPlugin)
        .add_plugins(input::zoom::ZoomPlugin)
        .run();
}
