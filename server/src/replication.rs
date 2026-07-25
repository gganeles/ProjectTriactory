//! Sends replicated state to newly-connected clients.
//!
//! Currently just the map: there's no fog of war yet, so a client receives the whole `TileMap`
//! as one `TilesRevealed` message the moment its connection is confirmed. The broader per-entity
//! replication (heroes, biomes, ...) planned in this folder's `README.md` is a later milestone.

use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use triactory_shared::protocol::{MapChannel, TilesRevealed};

use crate::map::TileMap;

pub fn send_map_on_connect(
    trigger: On<Add, Connected>,
    clients: Query<&RemoteId, With<ClientOf>>,
    server: Single<&Server>,
    map: Res<TileMap>,
    mut sender: ServerMultiMessageSender,
) {
    let Ok(client_id) = clients.get(trigger.entity) else {
        return;
    };

    let tile_count = map.tiles.len();
    let tiles = map.tiles.iter().map(|(coord, terrain)| (*coord, *terrain)).collect();
    let message = TilesRevealed(tiles);
    match sender.send::<_, MapChannel>(&message, &server, &NetworkTarget::Single(client_id.0)) {
        Ok(()) => info!("Sent {tile_count} tiles to newly-connected client {:?}", client_id.0),
        Err(err) => error!("Failed to send TilesRevealed to newly-connected client: {err:?}"),
    }
}
