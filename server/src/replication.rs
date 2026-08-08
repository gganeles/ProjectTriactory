//! Sends replicated state to newly-connected clients.
//!
//! Currently just the map: there's no fog of war yet, so a client receives the whole `TileMap`
//! as a `TilesRevealed` + `BiomeTowersRevealed` pair the moment its connection is confirmed. The
//! broader per-entity replication (heroes, biomes, ...) planned in this folder's `README.md` is
//! a later milestone.

use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use triactory_shared::protocol::{
    BiomeTowersRevealed, MapChannel, RegenerateMap, SetMapConfig, TilesRevealed,
};

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
    send_map_state(&map, client_id.0, &server, &mut sender);
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
        *map = TileMap::generate(config.map_type, config.num_players, random_seed())
            .expect("MapGenConfig should hold a valid map_type/num_players combination");
        info!("Regenerated map at request of client {:?}", client_id.0);
        send_map_state(&map, client_id.0, &server, &mut sender);
    }
}

/// DEBUG, easy to remove: apply a client-requested `map_type`/`num_players` to `MapGenConfig`,
/// regenerate the map accordingly (with a fresh seed, like `regenerate_map_on_request`), and
/// send the result back. Backs the debug map-settings picker on the client's main menu — see
/// `triactory_shared::protocol::SetMapConfig`'s docs for the full list of places to revert to
/// remove this feature.
pub fn set_map_config_on_request(
    mut clients: Query<(&RemoteId, &mut MessageReceiver<SetMapConfig>), With<ClientOf>>,
    server: Single<&Server>,
    mut config: ResMut<MapGenConfig>,
    mut map: ResMut<TileMap>,
    mut sender: ServerMultiMessageSender,
) {
    for (client_id, mut receiver) in &mut clients {
        let Some(request) = receiver.receive().last() else {
            continue;
        };
        config.map_type = request.map_type;
        config.num_players = request.num_players;
        *map = TileMap::generate(config.map_type, config.num_players, random_seed())
            .expect("SetMapConfig should carry a valid map_type/num_players combination");
        info!(
            "Applied map config {:?}/{} players at request of client {:?}",
            config.map_type, config.num_players, client_id.0
        );
        send_map_state(&map, client_id.0, &server, &mut sender);
    }
}

/// Sends both halves of the map's current state to `client_id`: `TilesRevealed` (terrain) and
/// `BiomeTowersRevealed` (Biome Town placement + starting towers) — always together, since a
/// client with tiles but no town list (or vice versa) can't render "BT" labels correctly.
fn send_map_state(
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
    let tiles_message = TilesRevealed(tiles);
    match sender.send::<_, MapChannel>(&tiles_message, server, &NetworkTarget::Single(client_id)) {
        Ok(()) => info!("Sent {tile_count} tiles to client {client_id:?}"),
        Err(err) => error!("Failed to send TilesRevealed to client {client_id:?}: {err:?}"),
    }

    let towers_message = BiomeTowersRevealed {
        towns: map.towns.clone(),
        starting_towers: map.starting_towers.clone(),
    };
    match sender.send::<_, MapChannel>(&towers_message, server, &NetworkTarget::Single(client_id)) {
        Ok(()) => info!(
            "Sent {} biome towns ({} starting towers) to client {client_id:?}",
            map.towns.len(),
            map.starting_towers.len()
        ),
        Err(err) => error!("Failed to send BiomeTowersRevealed to client {client_id:?}: {err:?}"),
    }
}
