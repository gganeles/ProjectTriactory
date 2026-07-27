//! Elevation + moisture noise over the triangle grid, seeded and deterministic (same
//! `edge_tiles`/`seed` always produces the same map). For each tile this samples two
//! independent [`Fbm`] noise fields — elevation (with a radial falloff subtracted so the map
//! forms an island rather than tiling forever) and moisture — then derives both the
//! gameplay-authoritative [`Terrain`] and the cosmetic [`TerrainType`] from the same sample, so
//! the two always agree (see [`TileData`]'s docs).
//!
//! The elevation-noise-minus-radial-distance island trick and the biome table itself are from
//! Amit Patel's polygon map generation article — see `shared/src/game/map/terrain.rs`.

use noise::{Fbm, NoiseFn, Perlin};
use std::collections::HashMap;
use triactory_shared::game::map::terrain::{MOUNTAIN_LEVEL, SEA_LEVEL, Terrain, TerrainType, TileData};
use triactory_shared::grid::TriCoord;

/// World-space edge length tiles are generated at. Only used as a coordinate scale for noise
/// sampling here — unrelated to the client's rendering `EDGE_LEN`, since noise coordinates are
/// arbitrary as long as they're consistent within one generation pass.
const EDGE_LEN: f32 = 1.0;

/// Generates elevation + moisture for every tile in `coords` and classifies each into
/// [`TileData`]. Deterministic: the same `coords`/`seed` always produces the same output.
pub fn generate(coords: &[TriCoord], seed: u32) -> HashMap<TriCoord, TileData> {
    let elevation_noise = Fbm::<Perlin>::new(seed);
    let moisture_noise = Fbm::<Perlin>::new(seed.wrapping_add(1));

    let max_radius = coords
        .iter()
        .map(|c| c.center_world(EDGE_LEN).length())
        .fold(0.0f32, f32::max)
        .max(1.0);
    // A handful of noise cycles across the map radius gives a natural-looking single landmass
    // rather than either uniform noise or a perfect circle.
    let elevation_frequency = 1.5 / max_radius as f64;
    let moisture_frequency = 2.5 / max_radius as f64;

    coords
        .iter()
        .map(|&coord| {
            let pos = coord.center_world(EDGE_LEN);

            let raw_elevation = elevation_noise.get([
                (pos.x as f64) * elevation_frequency,
                (pos.y as f64) * elevation_frequency,
            ]);
            let normalized_elevation = ((raw_elevation + 1.0) / 2.0) as f32;
            let distance_from_center = (pos.length() / max_radius).clamp(0.0, 1.0);
            let elevation = (normalized_elevation - distance_from_center * distance_from_center)
                .clamp(0.0, 1.0);

            let raw_moisture = moisture_noise.get([
                (pos.x as f64) * moisture_frequency,
                (pos.y as f64) * moisture_frequency,
            ]);
            let moisture = (((raw_moisture + 1.0) / 2.0) as f32).clamp(0.0, 1.0);

            let terrain = if elevation < SEA_LEVEL {
                Terrain::Water
            } else if elevation > MOUNTAIN_LEVEL {
                Terrain::Mountain
            } else {
                Terrain::Field
            };
            let terrain_type = TerrainType::classify(elevation, moisture);

            (
                coord,
                TileData {
                    terrain,
                    terrain_type,
                    elevation,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use triactory_shared::grid::hexagon_tiles;

    #[test]
    fn same_seed_is_deterministic() {
        let coords = hexagon_tiles(15).unwrap();
        let a = generate(&coords, 7);
        let b = generate(&coords, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_usually_differ() {
        let coords = hexagon_tiles(15).unwrap();
        let a = generate(&coords, 1);
        let b = generate(&coords, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn produces_more_than_one_terrain_and_terrain_type() {
        let coords = hexagon_tiles(21).unwrap();
        let map = generate(&coords, 42);
        let terrains: std::collections::HashSet<_> = map.values().map(|t| t.terrain).collect();
        let terrain_types: std::collections::HashSet<_> =
            map.values().map(|t| t.terrain_type).collect();
        assert!(
            terrains.len() > 1,
            "expected varied Terrain, got {terrains:?}"
        );
        assert!(
            terrain_types.len() > 1,
            "expected varied TerrainType, got {terrain_types:?}"
        );
    }

    #[test]
    fn terrain_and_terrain_type_always_agree_on_water() {
        let coords = hexagon_tiles(21).unwrap();
        let map = generate(&coords, 42);
        for tile in map.values() {
            let is_water_terrain = tile.terrain == Terrain::Water;
            let is_ocean_type = tile.terrain_type == TerrainType::Ocean;
            assert_eq!(
                is_water_terrain, is_ocean_type,
                "Terrain::Water and TerrainType::Ocean must always agree, got {tile:?}"
            );
        }
    }
}
