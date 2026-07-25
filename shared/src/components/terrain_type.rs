//! Fine-grained biome classification, purely for rendering variety — not to be confused with
//! the planned `Biome` (territory/tower-ownership) concept described in this directory's
//! `README.md`. [`TerrainType`] plays no role in movement/skill gating; that's still
//! [`super::tile::Terrain`], derived alongside it from the same elevation/moisture sample (see
//! `server/src/game/map/terrain.rs`).
//!
//! [`classify`](TerrainType::classify) and [`color`](TerrainType::color) port the elevation +
//! moisture biome table and `displayColors` from Amit Patel's polygon map generation article
//! (<http://www-cs-students.stanford.edu/~amitp/game-programming/polygon-map-generation/>,
//! `getBiome`/`displayColors` in the reference `mapgen2` source), minus the `MARSH`/`LAKE`/`ICE`
//! variants, which depend on flood-filling polygon adjacency to distinguish ocean from inland
//! water — this grid has no such water-body detection yet, so elevation alone decides `Ocean`
//! vs. `Beach` vs. dry land.

use bevy::prelude::Color;
use serde::{Deserialize, Serialize};

/// Elevation below this is `Ocean`.
pub const SEA_LEVEL: f32 = 0.1;
/// Elevation in `[SEA_LEVEL, BEACH_LEVEL)` is `Beach`.
pub const BEACH_LEVEL: f32 = 0.14;
/// Elevation above this is the snow-line band (`Scorched`/`Bare`/`Tundra`/`Snow`).
pub const MOUNTAIN_LEVEL: f32 = 0.8;

/// A tile's biome, classified from elevation and moisture, each in `[0.0, 1.0]`. Purely
/// cosmetic (see module docs) — pick a color for it with [`Self::color`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainType {
    Ocean,
    Beach,
    Snow,
    Tundra,
    Bare,
    Scorched,
    Taiga,
    Shrubland,
    TemperateDesert,
    TemperateRainForest,
    TemperateDeciduousForest,
    Grassland,
    SubtropicalDesert,
    TropicalRainForest,
    TropicalSeasonalForest,
}

impl TerrainType {
    /// Classifies a tile from its elevation and moisture, both expected in `[0.0, 1.0]`.
    /// Mirrors `mapgen2`'s `getBiome` (see module docs) for everything above sea level.
    pub fn classify(elevation: f32, moisture: f32) -> Self {
        if elevation < SEA_LEVEL {
            Self::Ocean
        } else if elevation < BEACH_LEVEL {
            Self::Beach
        } else if elevation > MOUNTAIN_LEVEL {
            if moisture > 0.50 {
                Self::Snow
            } else if moisture > 0.33 {
                Self::Tundra
            } else if moisture > 0.16 {
                Self::Bare
            } else {
                Self::Scorched
            }
        } else if elevation > 0.6 {
            if moisture > 0.66 {
                Self::Taiga
            } else if moisture > 0.33 {
                Self::Shrubland
            } else {
                Self::TemperateDesert
            }
        } else if elevation > 0.3 {
            if moisture > 0.83 {
                Self::TemperateRainForest
            } else if moisture > 0.50 {
                Self::TemperateDeciduousForest
            } else if moisture > 0.16 {
                Self::Grassland
            } else {
                Self::TemperateDesert
            }
        } else if moisture > 0.66 {
            Self::TropicalRainForest
        } else if moisture > 0.33 {
            Self::TropicalSeasonalForest
        } else if moisture > 0.16 {
            Self::Grassland
        } else {
            Self::SubtropicalDesert
        }
    }

    /// The biome's display color, ported verbatim from `mapgen2`'s `displayColors` table.
    pub fn color(&self) -> Color {
        let (r, g, b) = match self {
            Self::Ocean => (0x44, 0x44, 0x7a),
            Self::Beach => (0xa0, 0x90, 0x77),
            Self::Snow => (0xff, 0xff, 0xff),
            Self::Tundra => (0xbb, 0xbb, 0xaa),
            Self::Bare => (0x88, 0x88, 0x88),
            Self::Scorched => (0x55, 0x55, 0x55),
            Self::Taiga => (0x99, 0xaa, 0x77),
            Self::Shrubland => (0x88, 0x99, 0x77),
            Self::TemperateDesert => (0xc9, 0xd2, 0x9b),
            Self::TemperateRainForest => (0x44, 0x88, 0x55),
            Self::TemperateDeciduousForest => (0x67, 0x94, 0x59),
            Self::Grassland => (0x88, 0xaa, 0x55),
            Self::SubtropicalDesert => (0xd2, 0xb9, 0x8b),
            Self::TropicalRainForest => (0x33, 0x77, 0x55),
            Self::TropicalSeasonalForest => (0x55, 0x99, 0x44),
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
    fn high_dry_elevation_is_scorched() {
        assert_eq!(TerrainType::classify(0.9, 0.05), TerrainType::Scorched);
    }

    #[test]
    fn high_wet_elevation_is_snow() {
        assert_eq!(TerrainType::classify(0.9, 0.9), TerrainType::Snow);
    }

    #[test]
    fn low_wet_elevation_is_tropical_rain_forest() {
        assert_eq!(
            TerrainType::classify(0.2, 0.9),
            TerrainType::TropicalRainForest
        );
    }

    #[test]
    fn low_dry_elevation_is_subtropical_desert() {
        assert_eq!(
            TerrainType::classify(0.2, 0.05),
            TerrainType::SubtropicalDesert
        );
    }

    #[test]
    fn every_variant_has_a_distinct_color() {
        let variants = [
            TerrainType::Ocean,
            TerrainType::Beach,
            TerrainType::Snow,
            TerrainType::Tundra,
            TerrainType::Bare,
            TerrainType::Scorched,
            TerrainType::Taiga,
            TerrainType::Shrubland,
            TerrainType::TemperateDesert,
            TerrainType::TemperateRainForest,
            TerrainType::TemperateDeciduousForest,
            TerrainType::Grassland,
            TerrainType::SubtropicalDesert,
            TerrainType::TropicalRainForest,
            TerrainType::TropicalSeasonalForest,
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
