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

/// DEBUG, easy to remove: client → server, "reroll the map with a new random seed and send it
/// back to me." Backs the debug "Back to menu" button (`client/src/ui/debug_back_button.rs`) —
/// remove this message, `MapChannel`'s direction below (revert to `ServerToClient`), and
/// `server/src/replication.rs`'s `regenerate_map_on_request` together to fully strip the
/// feature.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegenerateMap;

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

        app.register_message::<RegenerateMap>()
            .add_direction(NetworkDirection::ClientToServer);
    }
}
