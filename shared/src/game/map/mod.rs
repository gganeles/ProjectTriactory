pub mod biome;
pub mod combat;
pub mod generation;
pub mod production;
pub mod terrain;

use bevy::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            biome::BiomePlugin,
            combat::CombatPlugin,
            production::ProductionPlugin,
        ));
    }
}
