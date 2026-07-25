pub mod economy;
pub mod input;
pub mod tech;
pub mod vision;

use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            economy::EconomyPlugin,
            tech::TechPlugin,
            vision::VisionPlugin,
            input::InputPlugin,
        ));
    }
}
