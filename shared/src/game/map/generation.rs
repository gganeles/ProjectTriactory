//! Types needed by both map generation (server) and, eventually, lobby/map-preview UI (client) —
//! no generation *logic* here, just the vocabulary. Mirrors `shared/src/grid/hexagon.rs`'s
//! split: pure/seedless shape types live in `shared`, the seeded algorithms that use them live
//! in `server/src/game/map/generation/`.

use serde::{Deserialize, Serialize};

/// How land is distributed across the map. Drives both sizing (`edge_tiles`, town count) and
/// the noise-falloff shape used to carve land from water — see
/// `server/src/game/map/generation/shape.rs`. Each variant's doc comment gives its target water
/// percentage range; the exact ranges live next to `shape.rs`'s per-`MapType` constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapType {
    /// 0-10% water. Almost no water.
    Drylands,
    /// 25-30% water. Moderate wetness results in scattered inland lakes.
    Lakes,
    /// 40-70% water. High wetness shrinks continents, expands surrounding ocean.
    Continents,
    /// 40-60% water. Large land mass surrounded by ocean and the occasional island.
    Pangea,
    /// 60-80% water. High wetness leads to small island chains and more ocean.
    Archipelago,
    /// 90-100% water. Almost all tiles become water or ocean; land is manually forced for
    /// Biome Towns (see `TileMap::generate`'s town-placement step).
    Waterworld,
}

/// A player's index within this match, `0..num_players`. Not a real player identity — a future
/// lobby maps its `TribeId`/player list onto these slots (see `shared/src/game/player/mod.rs`'s
/// planned `TribeId`). Kept as a plain newtype so map generation has zero dependency on the
/// not-yet-built lobby system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlayerSlot(pub u8);
