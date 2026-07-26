use bevy::prelude::*;

pub struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, _app: &mut App) {
        // stub — registers Biome, BiomeTower, BiomeOwner, BiomeLevel,
        // BiomeCapacity, OccupationTimer, UnderAttack components + systems
    }
}
