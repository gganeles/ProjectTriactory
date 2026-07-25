//! Tile data. Tiles are not entities — `TileData` is looked up by [`crate::grid::TriCoord`]
//! inside the server's `TileMap` resource (and the client's `RevealedTiles` resource).

use serde::{Deserialize, Serialize};

use super::terrain_type::TerrainType;

/// The environment a triangle tile is made of. Assigned during map generation and used for
/// movement (skill gating), placement/boost lookups, and rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Terrain {
    #[default]
    Field,
    Mountain,
    Water,
    Empty,
}

/// Everything generated for one map tile: the coarse, gameplay-authoritative [`Terrain`]
/// (movement/skill gating) plus the finer-grained [`TerrainType`] used only for biome coloring.
/// Both are derived from the same elevation/moisture sample during generation (see
/// `server/src/game/map/terrain.rs`), so they always agree (e.g. `Terrain::Water` tiles are
/// always `TerrainType::Ocean`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileData {
    pub terrain: Terrain,
    pub terrain_type: TerrainType,
}
