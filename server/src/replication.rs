//! Sends replicated state to newly-connected clients.
//!
//! Currently just the map: there's no fog of war yet, so a client receives the whole `TileMap`
//! as one `TilesRevealed` message the moment its connection is confirmed. The broader per-entity
//! replication (heroes, biomes, ...) planned in this folder's `README.md` is a later milestone.

use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use triactory_shared::protocol::{MapChannel, RegenerateMap, TilesRevealed};

use crate::game::map::{MapGenConfig, TileMap, random_seed};

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
    send_tiles(&map, client_id.0, &server, &mut sender);
}

/// DEBUG, easy to remove: reroll the map on request from a client (backs the debug "Back to
/// menu" button — see `triactory_shared::protocol::RegenerateMap`'s docs for the full list of
/// places to revert to remove this feature).
pub fn regenerate_map_on_request(
    mut clients: Query<(&RemoteId, &mut MessageReceiver<RegenerateMap>), With<ClientOf>>,
    server: Single<&Server>,
    config: Res<MapGenConfig>,
    mut map: ResMut<TileMap>,
    mut sender: ServerMultiMessageSender,
) {
    for (client_id, mut receiver) in &mut clients {
        if receiver.receive().count() == 0 {
            continue;
        }
        *map = TileMap::generate_hexagon(config.edge_tiles, random_seed())
            .expect("MapGenConfig::edge_tiles must be odd and >= 3");
        info!("Regenerated map at request of client {:?}", client_id.0);
        send_tiles(&map, client_id.0, &server, &mut sender);
    }
}

fn send_tiles(
    map: &TileMap,
    client_id: PeerId,
    server: &Server,
    sender: &mut ServerMultiMessageSender,
) {
    let tile_count = map.tiles.len();
    let tiles = map
        .tiles
        .iter()
        .map(|(coord, tile)| (*coord, *tile))
        .collect();
    let message = TilesRevealed(tiles);
    match sender.send::<_, MapChannel>(&message, server, &NetworkTarget::Single(client_id)) {
        Ok(()) => info!("Sent {tile_count} tiles to client {client_id:?}"),
        Err(err) => error!("Failed to send TilesRevealed to client {client_id:?}: {err:?}"),
    }
}
