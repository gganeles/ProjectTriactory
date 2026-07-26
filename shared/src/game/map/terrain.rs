//! Tile data. Tiles are not entities — `TileData` is looked up by [`crate::grid::TriCoord`]
//! inside the server's `TileMap` resource (and the client's `RevealedTiles` resource).

use bevy::prelude::Color;
use serde::{Deserialize, Serialize};

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

/// Fine-grained biome classification, purely for rendering variety — not to be confused with
/// the planned `Biome` (territory/tower-ownership) concept in `game::map::biome`.
/// [`TerrainType`] plays no role in movement/skill gating; that's still [`Terrain`], derived
/// alongside it from the same elevation/moisture sample (see `server/src/game/map/terrain.rs`).
///
/// [`classify`](TerrainType::classify) and [`color`](TerrainType::color) started as a port of
/// the full elevation + moisture biome table and `displayColors` from Amit Patel's polygon map
/// generation article
/// (<http://www-cs-students.stanford.edu/~amitp/game-programming/polygon-map-generation/>,
/// `getBiome`/`displayColors` in the reference `mapgen2` source); pared down to 4 elevation
/// bands (`Ocean`/`Beach`/`Tundra`/`Snow`) for a simpler look. Re-adding the dropped
/// moisture-driven variants (forest/desert/grassland/...) is just restoring the removed
/// branches in `classify`/`color`/the `every_variant_has_a_distinct_color` test below — see
/// git history on this file for the full table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainType {
    Ocean,
    Beach,
    Snow,
    Tundra,
}

/// Elevation below this is `Ocean`.
pub const SEA_LEVEL: f32 = 0.1;
/// Elevation in `[SEA_LEVEL, BEACH_LEVEL)` is `Beach`.
pub const BEACH_LEVEL: f32 = 0.14;
/// Elevation above this is `Snow` (mountain peaks); everything between `BEACH_LEVEL` and this
/// is `Tundra`.
pub const MOUNTAIN_LEVEL: f32 = 0.8;

impl TerrainType {
    /// Classifies a tile from its elevation, expected in `[0.0, 1.0]`. `moisture` (also
    /// `[0.0, 1.0]`) is accepted but currently unused — kept in the signature so callers don't
    /// need to change if a future biome split reintroduces it (see the type's docs).
    pub fn classify(elevation: f32, _moisture: f32) -> Self {
        if elevation < SEA_LEVEL {
            Self::Ocean
        } else if elevation < BEACH_LEVEL {
            Self::Beach
        } else if elevation > MOUNTAIN_LEVEL {
            Self::Snow
        } else {
            Self::Tundra
        }
    }

    /// The biome's display color. `Ocean`/`Beach`/`Snow`/`Tundra` values are ported verbatim
    /// from `mapgen2`'s `displayColors` table (see the type's docs).
    pub fn color(&self) -> Color {
        let (r, g, b) = match self {
            Self::Ocean => (0x44, 0x44, 0x7a),
            Self::Beach => (0xa0, 0x90, 0x77),
            Self::Snow => (0xff, 0xff, 0xff),
            Self::Tundra => (0xbb, 0xbb, 0xaa),
        };
        Color::srgb_u8(r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_sea_level_is_ocean() {
        assert_eq!(TerrainType::classify(0.0, 0.5), TerrainType::Ocean);
        assert_eq!(
            TerrainType::classify(SEA_LEVEL - 0.01, 0.5),
            TerrainType::Ocean
        );
    }

    #[test]
    fn beach_band_is_thin_and_dry_land_starts_after_it() {
        assert_eq!(TerrainType::classify(SEA_LEVEL, 0.5), TerrainType::Beach);
        assert_ne!(TerrainType::classify(BEACH_LEVEL, 0.5), TerrainType::Beach);
        assert_ne!(TerrainType::classify(BEACH_LEVEL, 0.5), TerrainType::Ocean);
    }

    #[test]
    fn above_mountain_level_is_snow() {
        assert_eq!(TerrainType::classify(0.9, 0.05), TerrainType::Snow);
        assert_eq!(TerrainType::classify(0.9, 0.9), TerrainType::Snow);
    }

    #[test]
    fn between_beach_and_mountain_level_is_tundra() {
        assert_eq!(TerrainType::classify(0.2, 0.9), TerrainType::Tundra);
        assert_eq!(
            TerrainType::classify(MOUNTAIN_LEVEL, 0.05),
            TerrainType::Tundra
        );
    }

    #[test]
    fn every_variant_has_a_distinct_color() {
        let variants = [
            TerrainType::Ocean,
            TerrainType::Beach,
            TerrainType::Snow,
            TerrainType::Tundra,
        ];
        let mut colors: Vec<[u8; 3]> = variants
            .iter()
            .map(|t| {
                let srgba = t.color().to_srgba();
                [
                    (srgba.red * 255.0).round() as u8,
                    (srgba.green * 255.0).round() as u8,
                    (srgba.blue * 255.0).round() as u8,
                ]
            })
            .collect();
        let before = colors.len();
        colors.sort_unstable();
        colors.dedup();
        assert_eq!(colors.len(), before, "two biomes share the same color");
    }
}
