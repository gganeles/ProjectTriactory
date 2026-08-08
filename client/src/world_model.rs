//! The client's memory of the map: a `RevealedTiles` resource fed by `TilesRevealed` messages,
//! and a `BiomeTowers` resource fed by `BiomeTowersRevealed` messages.
//!
//! There's no fog of war yet, so `RevealedTiles` just accumulates whatever the server has sent —
//! currently the whole map, once, right after connecting (see `server/src/replication.rs`). Once
//! fog exists, this is where explored-but-not-currently-visible terrain stays remembered.
//! `BiomeTowers` instead replaces its contents wholesale on every message, since the server
//! always sends its complete current town list rather than an incremental delta.

use bevy::prelude::*;
use lightyear::prelude::*;
use std::collections::HashMap;
use triactory_shared::game::map::terrain::TileData;
use triactory_shared::grid::TriCoord;
use triactory_shared::protocol::{BiomeTowersRevealed, TilesRevealed};

#[derive(Resource, Debug, Default)]
pub struct RevealedTiles {
    pub tiles: HashMap<TriCoord, TileData>,
}

/// Every placed Biome Town the server has told us about, plus which ones are a player's
/// starting town — see `BiomeTowersRevealed`'s docs. Used by
/// `client/src/rendering/hex_map.rs` to render "BT"/"BT<slot>" labels.
#[derive(Resource, Debug, Default)]
pub struct BiomeTowers {
    pub towns: Vec<TriCoord>,
    /// Index `i` is `PlayerSlot(i)`'s starting town.
    pub starting_towers: Vec<TriCoord>,
}

pub struct WorldModelPlugin;

impl Plugin for WorldModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RevealedTiles>()
            .init_resource::<BiomeTowers>()
            .add_systems(Update, (receive_tiles_revealed, receive_biome_towers_revealed));
    }
}

fn receive_tiles_revealed(
    mut revealed: ResMut<RevealedTiles>,
    mut receivers: Query<&mut MessageReceiver<TilesRevealed>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            revealed.tiles.extend(message.0);
        }
    }
}

fn receive_biome_towers_revealed(
    mut towers: ResMut<BiomeTowers>,
    mut receivers: Query<&mut MessageReceiver<BiomeTowersRevealed>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            towers.towns = message.towns;
            towers.starting_towers = message.starting_towers;
        }
    }
}
