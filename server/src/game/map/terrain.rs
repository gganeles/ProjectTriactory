//! Elevation + moisture noise over the triangle grid, seeded and deterministic (same
//! `coords`/`seed`/`shape` always produces the same map). For each tile this samples two
//! independent [`Fbm`] noise fields — elevation (with a per-`MapType` falloff subtracted, via
//! `shape.rs`'s [`ShapeSpec`], so the map forms the target land shape rather than tiling
//! forever) and moisture — then derives both the gameplay-authoritative [`Terrain`] and the
//! cosmetic [`TerrainType`] from the same sample, so the two always agree (see [`TileData`]'s
//! docs).
//!
//! The elevation-noise-minus-falloff island trick and the biome table itself are from Amit
//! Patel's polygon map generation article — see `shared/src/game/map/terrain.rs`.

use noise::{Fbm, NoiseFn, Perlin};
use std::collections::HashMap;
use triactory_shared::game::map::terrain::{MOUNTAIN_LEVEL, SEA_LEVEL, Terrain, TerrainType, TileData};
use triactory_shared::grid::TriCoord;

use super::generation::{EDGE_LEN, ShapeSpec, elevation_penalty, max_radius_of};

/// Generates elevation + moisture for every tile in `coords` and classifies each into
/// [`TileData`]. `shape` controls where land gets boosted (and, for `Drylands`/`Lakes`, where
/// water gets forced) — see `generation/shape.rs`. If `shape.target_water_fraction` is `Some`, the raw
/// elevation distribution is remapped (see [`remap_to_water_fraction`]) so *exactly* that
/// fraction of tiles fall below `SEA_LEVEL`, rather than whatever fraction the noise/falloff
/// happens to produce. Deterministic: the same `coords`/`seed`/`shape` always produces the same
/// output.
pub fn generate(coords: &[TriCoord], seed: u32, shape: &ShapeSpec) -> HashMap<TriCoord, TileData> {
    let elevation_noise = Fbm::<Perlin>::new(seed);
    let moisture_noise = Fbm::<Perlin>::new(seed.wrapping_add(1));

    let max_radius = max_radius_of(coords);
    // A handful of noise cycles across the map radius gives a natural-looking landmass shape
    // rather than either uniform noise or a perfect circle.
    let moisture_frequency = 2.5 / max_radius as f64;

    let mut samples: Vec<(TriCoord, f32, f32)> = coords
        .iter()
        .map(|&coord| {
            let pos = coord.center_world(EDGE_LEN);

            let raw_elevation = elevation_noise.get([
                (pos.x as f64) * shape.elevation_frequency,
                (pos.y as f64) * shape.elevation_frequency,
            ]);
            let normalized_elevation = ((raw_elevation + 1.0) / 2.0) as f32;
            // Deliberately unclamped here: clamping before the percentile remap below would
            // collapse every heavily-penalized tile (e.g. far outside a small Archipelago land
            // circle) to the same exact 0.0, creating a tie block that can swallow the target
            // percentile entirely (see `remap_to_water_fraction`'s docs). The legacy
            // (`target_water_fraction: None`) path clamps below instead, preserving its old
            // exact behavior.
            let elevation = normalized_elevation - elevation_penalty(pos, shape);

            let raw_moisture = moisture_noise.get([
                (pos.x as f64) * moisture_frequency,
                (pos.y as f64) * moisture_frequency,
            ]);
            let moisture = (((raw_moisture + 1.0) / 2.0) as f32).clamp(0.0, 1.0);

            (coord, elevation, moisture)
        })
        .collect();

    match shape.target_water_fraction {
        Some(target_water_fraction) => remap_to_water_fraction(&mut samples, target_water_fraction),
        None => {
            for (_, elevation, _) in samples.iter_mut() {
                *elevation = elevation.clamp(0.0, 1.0);
            }
        }
    }

    samples
        .into_iter()
        .map(|(coord, elevation, moisture)| {
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

/// Rescales every tile's elevation (the second tuple field, **unclamped** — see `generate`'s
/// caller-side comment on why) so that *exactly* `target_water_fraction` of them land below
/// [`SEA_LEVEL`] — a monotonic, piecewise-linear remap that pivots the target-fraction
/// percentile of the raw distribution onto `SEA_LEVEL` itself: everything below that percentile
/// stretches into `[0, SEA_LEVEL)`, everything above stretches into `[SEA_LEVEL, 1]`. This keeps
/// the existing fixed `SEA_LEVEL`/`MOUNTAIN_LEVEL` thresholds working unchanged afterward —
/// `Terrain` and `TerrainType` still read the same remapped value, so they still always agree —
/// while giving each `MapType` exact control over its water amount instead of leaving it to
/// emerge from the noise/falloff shape.
fn remap_to_water_fraction(samples: &mut [(TriCoord, f32, f32)], target_water_fraction: f32) {
    let n = samples.len();
    if n == 0 {
        return;
    }

    let mut sorted: Vec<f32> = samples.iter().map(|(_, elevation, _)| *elevation).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = (((n - 1) as f32) * target_water_fraction.clamp(0.0, 1.0)).round() as usize;
    let threshold = sorted[idx];
    let min = sorted[0];
    let max = sorted[n - 1];

    for (_, elevation, _) in samples.iter_mut() {
        *elevation = if *elevation < threshold {
            if threshold > min {
                SEA_LEVEL * (*elevation - min) / (threshold - min)
            } else {
                0.0
            }
        } else if max > threshold {
            SEA_LEVEL + (1.0 - SEA_LEVEL) * (*elevation - threshold) / (max - threshold)
        } else {
            SEA_LEVEL
        }
        .clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::generation::legacy_single_island;
    use triactory_shared::game::map::generation::MapType;
    use triactory_shared::grid::hexagon_tiles;

    #[test]
    fn same_seed_is_deterministic() {
        let coords = hexagon_tiles(15).unwrap();
        let shape = legacy_single_island(&coords);
        let a = generate(&coords, 7, &shape);
        let b = generate(&coords, 7, &shape);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_usually_differ() {
        let coords = hexagon_tiles(15).unwrap();
        let shape = legacy_single_island(&coords);
        let a = generate(&coords, 1, &shape);
        let b = generate(&coords, 2, &shape);
        assert_ne!(a, b);
    }

    #[test]
    fn produces_more_than_one_terrain_and_terrain_type() {
        let coords = hexagon_tiles(21).unwrap();
        let shape = legacy_single_island(&coords);
        let map = generate(&coords, 42, &shape);
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
        let shape = legacy_single_island(&coords);
        let map = generate(&coords, 42, &shape);
        for tile in map.values() {
            let is_water_terrain = tile.terrain == Terrain::Water;
            let is_ocean_type = tile.terrain_type == TerrainType::Ocean;
            assert_eq!(
                is_water_terrain, is_ocean_type,
                "Terrain::Water and TerrainType::Ocean must always agree, got {tile:?}"
            );
        }
    }

    #[test]
    fn drylands_produce_far_less_water_than_waterworld() {
        use crate::game::map::generation::{ShapeSpec, Stage, rng_for, shape_spec_for};

        // Non-overlapping target ranges (Drylands 0-10%, Waterworld 90-100%), so this holds for
        // any seed — unlike e.g. Continents/Archipelago, whose ranges now overlap.
        let coords = hexagon_tiles(31).unwrap();
        let mut drylands_rng = rng_for(42, Stage::ShapeCenters);
        let mut waterworld_rng = rng_for(42, Stage::ShapeCenters);
        let drylands_shape = shape_spec_for(MapType::Drylands, &coords, &mut drylands_rng);
        let waterworld_shape = shape_spec_for(MapType::Waterworld, &coords, &mut waterworld_rng);

        let water_fraction = |shape: &ShapeSpec| {
            let map = generate(&coords, 42, shape);
            let water = map.values().filter(|t| t.terrain == Terrain::Water).count();
            water as f32 / map.len() as f32
        };

        assert!(
            water_fraction(&waterworld_shape) > water_fraction(&drylands_shape),
            "expected Waterworld to produce more water than Drylands on the same coords/seed"
        );
    }

    #[test]
    fn realized_water_fraction_matches_shape_target() {
        use crate::game::map::generation::{Stage, rng_for, shape_spec_for};

        let coords = hexagon_tiles(31).unwrap();
        // Wide enough to absorb discretization noise on a few thousand tiles, tight enough to
        // catch a regression back to "whatever the falloff shape happens to produce."
        const TOLERANCE: f32 = 0.05;

        for map_type in [
            MapType::Drylands,
            MapType::Lakes,
            MapType::Continents,
            MapType::Pangea,
            MapType::Archipelago,
            MapType::Waterworld,
        ] {
            for seed in [1u32, 2, 3] {
                let shape = shape_spec_for(map_type, &coords, &mut rng_for(seed, Stage::ShapeCenters));
                let target = shape
                    .target_water_fraction
                    .expect("every real MapType sets a target water fraction");
                let map = generate(&coords, seed, &shape);
                let realized =
                    map.values().filter(|t| t.terrain == Terrain::Water).count() as f32 / map.len() as f32;
                assert!(
                    (realized - target).abs() <= TOLERANCE,
                    "{map_type:?}/seed {seed}: realized water fraction {realized:.3} too far from target {target:.3}"
                );
            }
        }
    }
}
