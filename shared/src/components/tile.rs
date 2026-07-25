//! Tile data. Tiles are not entities — `Terrain` is looked up by [`crate::grid::TriCoord`]
//! inside the server's `TileMap` resource (and the client's future `RevealedTiles` resource).

/// The environment a triangle tile is made of. Assigned during map generation and used for
/// movement (skill gating), placement/boost lookups, and rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Terrain {
    #[default]
    Field,
    Mountain,
    Water,
    Empty,
}
