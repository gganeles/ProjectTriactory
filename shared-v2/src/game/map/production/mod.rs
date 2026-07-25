use bevy::prelude::*;

pub struct ProductionPlugin;

impl Plugin for ProductionPlugin {
    fn build(&self, _app: &mut App) {
        // stub — registers Nrp, Arp, ProductionRate, ProductionHalted,
        // Recipe, Stockpile, ResourceLink components + production systems
    }
}
