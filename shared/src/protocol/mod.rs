//! Lightyear registration for everything that crosses the network.
//!
//! Currently just the map-replication message described below; the rest of the wire contract
//! (replicated components, gameplay messages/channels, native inputs) grows here as later
//! milestones need it — see this folder's `README.md` for the full planned shape.
//!
//! One `ProtocolPlugin` added by **both** apps via `SharedPlugin`, so registration order is
//! identical on both sides — Lightyear requires this. Must be added after
//! `ClientPlugins`/`ServerPlugins` but before any `Client`/`Server` entity is spawned (enforced
//! by both apps' `main.rs` ordering).

use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::map::generation::MapType;
use crate::game::map::terrain::TileData;
use crate::grid::TriCoord;

/// Reliable ordered channel carrying map/terrain traffic.
pub struct MapChannel;

/// Server → client: tiles and their terrain/biome data. There's no fog of war yet, so today
/// this is always the whole map sent once on connect, not a fog-driven reveal batch — the name
/// matches the planned message in `shared/src/protocol/README.md` since it'll grow into that
/// later.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TilesRevealed(pub Vec<(TriCoord, TileData)>);

/// Server → client: every placed Biome Town, plus which ones are a player's starting town.
/// `starting_towers[i]` is `PlayerSlot(i)`'s. Sent alongside `TilesRevealed` (see
/// `server/src/replication.rs`'s `send_map_state`) so the client can render "BT"/"BT<slot>"
/// labels (`client/src/rendering/hex_map.rs`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BiomeTowersRevealed {
    pub towns: Vec<TriCoord>,
    pub starting_towers: Vec<TriCoord>,
}

/// DEBUG, easy to remove: client → server, "reroll the map with a new random seed and send it
/// back to me." Backs the debug "Back to menu" button (`client/src/ui/debug_back_button.rs`) —
/// remove this message, `MapChannel`'s direction below (revert to `ServerToClient`), and
/// `server/src/replication.rs`'s `regenerate_map_on_request` together to fully strip the
/// feature.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegenerateMap;

/// DEBUG, easy to remove: client → server, "use this map type/player count from now on, and
/// regenerate the map accordingly." Backs the debug map-settings picker
/// (`client/src/ui/main_menu.rs`) — remove this message and
/// `server/src/replication.rs`'s `set_map_config_on_request` together to fully strip the
/// feature (leave `MapChannel`'s `Bidirectional` direction alone if `RegenerateMap` is kept).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SetMapConfig {
    pub map_type: MapType,
    pub num_players: u8,
}

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_channel::<MapChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        // Bidirectional (rather than ServerToClient) only because of the debug `RegenerateMap`
        // request below — see its docs.
        .add_direction(NetworkDirection::Bidirectional);

        app.register_message::<TilesRevealed>()
            .add_direction(NetworkDirection::ServerToClient);

        app.register_message::<BiomeTowersRevealed>()
            .add_direction(NetworkDirection::ServerToClient);

        app.register_message::<RegenerateMap>()
            .add_direction(NetworkDirection::ClientToServer);

        app.register_message::<SetMapConfig>()
            .add_direction(NetworkDirection::ClientToServer);
    }
}
