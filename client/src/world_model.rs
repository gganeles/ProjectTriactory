//! The client's memory of the map: a `RevealedTiles` resource fed by `TilesRevealed` messages.
//!
//! There's no fog of war yet, so this just accumulates whatever the server has sent — currently
//! the whole map, once, right after connecting (see `server/src/replication.rs`). Once fog
//! exists, this is where explored-but-not-currently-visible terrain stays remembered.

use bevy::prelude::*;
use lightyear::prelude::*;
use std::collections::HashMap;
use triactory_shared::game::map::terrain::TileData;
use triactory_shared::grid::TriCoord;
use triactory_shared::protocol::TilesRevealed;

#[derive(Resource, Debug, Default)]
pub struct RevealedTiles {
    pub tiles: HashMap<TriCoord, TileData>,
}

pub struct WorldModelPlugin;

impl Plugin for WorldModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RevealedTiles>()
            .add_systems(Update, receive_tiles_revealed);
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
