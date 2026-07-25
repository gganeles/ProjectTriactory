pub mod entities;
pub mod map;
pub mod player;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            map::biome::BiomePlugin,
            map::combat::CombatPlugin,
            map::production::ProductionPlugin,
            player::economy::EconomyPlugin,
            player::tech::TechPlugin,
            player::vision::VisionPlugin,
            player::input::InputPlugin,
            entities::projectiles::ProjectilePlugin,
        ));
    }
}
