//! Per-map-type falloff shape: which regions of noise get land-boosted vs water-forced. Plugs
//! directly into `terrain::generate` — the raw elevation noise sample for a tile has
//! [`elevation_penalty`] subtracted from it before thresholding into `Terrain`/`TerrainType`,
//! generalizing the original single-radial-falloff-from-origin trick to an arbitrary number of
//! land-boosting (and, for `Drylands`/`Lakes`, water-forcing) centers.
//!
//! The falloff centers only control the *pattern* of where water tends to cluster (a few big
//! blobs for Continents, many small ones for Archipelago, etc.) — the *amount* of water is a
//! separate, explicit knob: [`ShapeSpec::target_water_fraction`], sampled per generation from
//! each `MapType`'s allowed range and enforced exactly by `terrain::generate` via a percentile
//! remap (see that function's docs), rather than emerging incidentally from the falloff
//! geometry. `Waterworld`'s extreme target can leave close to zero naturally-occurring land —
//! `TileMap::generate` handles that by forcing land directly under every Biome Town regardless
//! of what this shape produces (see its docs).

use bevy::math::Vec2;
use rand::Rng;
use std::f32::consts::TAU;
use std::ops::RangeInclusive;
use triactory_shared::game::map::generation::MapType;
use triactory_shared::grid::TriCoord;

use super::max_radius_of;

#[derive(Debug, Clone, Copy)]
pub(crate) struct FalloffCenter {
    pub pos: Vec2,
    pub radius: f32,
    pub strength: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct ShapeSpec {
    /// Tiles close to any of these (relative to that center's own radius) get an elevation
    /// boost — this is what makes land instead of the whole map defaulting to ocean.
    pub land_centers: Vec<FalloffCenter>,
    /// Tiles close to any of these get an elevation *penalty* on top of `land_centers` — used
    /// only by `Drylands`/`Lakes` to carve small lakes out of an otherwise all-land map.
    pub water_pockets: Vec<FalloffCenter>,
    pub elevation_frequency: f64,
    /// The exact fraction of tiles `terrain::generate` should make water, via a percentile
    /// remap of the raw elevation distribution — `None` only for [`legacy_single_island`], which
    /// keeps the old fixed-`SEA_LEVEL` behavior for tests/tooling that don't go through a
    /// `MapType`.
    pub target_water_fraction: Option<f32>,
}

// --- placeholder-tunable constants, one block per MapType ---

// Drylands/Lakes share the same "one map-covering land center + scattered water pockets" recipe
// — Lakes just gets more/bigger pockets, since its higher target water fraction should read as
// numerous scattered lakes rather than one connected sea.
const DRYLANDS_WATER_POCKET_COUNT: RangeInclusive<u32> = 1..=4;
const DRYLANDS_WATER_POCKET_RADIUS_FRAC: RangeInclusive<f32> = 0.03..=0.08;
const DRYLANDS_ELEVATION_FREQ_NUM: f64 = 1.0;

const LAKES_WATER_POCKET_COUNT: RangeInclusive<u32> = 5..=10;
const LAKES_WATER_POCKET_RADIUS_FRAC: RangeInclusive<f32> = 0.05..=0.12;
const LAKES_ELEVATION_FREQ_NUM: f64 = 1.3;

const CONTINENTS_CENTER_COUNT: RangeInclusive<u32> = 1..=3;
const CONTINENTS_RADIUS_FRAC: RangeInclusive<f32> = 0.45..=0.75;
const CONTINENTS_ELEVATION_FREQ_NUM: f64 = 1.2;

// Pangea is a single dominant landmass (bigger/softer than any one Continents center) plus a
// small chance of 1-2 tiny, distant "occasional island" specks.
const PANGEA_RADIUS_FRAC: RangeInclusive<f32> = 0.65..=0.85;
const PANGEA_EXTRA_ISLAND_COUNT: RangeInclusive<u32> = 0..=2;
const PANGEA_EXTRA_ISLAND_RADIUS_FRAC: RangeInclusive<f32> = 0.04..=0.09;
const PANGEA_ELEVATION_FREQ_NUM: f64 = 1.1;

const ARCHIPELAGO_CENTER_COUNT: RangeInclusive<u32> = 6..=12;
const ARCHIPELAGO_RADIUS_FRAC: RangeInclusive<f32> = 0.10..=0.22;
const ARCHIPELAGO_ELEVATION_FREQ_NUM: f64 = 2.2;

// Waterworld's target water fraction (90-100%) dominates regardless of shape — this just gives
// whatever sliver of "driest" tiles survives the percentile remap somewhere coherent to cluster
// instead of scattering as pure noise speckle.
const WATERWORLD_RADIUS_FRAC: RangeInclusive<f32> = 0.15..=0.30;
const WATERWORLD_ELEVATION_FREQ_NUM: f64 = 2.5;

// --- target water fraction ranges, one per MapType, straight from the design table ---

const DRYLANDS_WATER_FRACTION: RangeInclusive<f32> = 0.00..=0.10;
const LAKES_WATER_FRACTION: RangeInclusive<f32> = 0.25..=0.30;
const CONTINENTS_WATER_FRACTION: RangeInclusive<f32> = 0.40..=0.70;
const PANGEA_WATER_FRACTION: RangeInclusive<f32> = 0.40..=0.60;
const ARCHIPELAGO_WATER_FRACTION: RangeInclusive<f32> = 0.60..=0.80;
const WATERWORLD_WATER_FRACTION: RangeInclusive<f32> = 0.90..=1.00;

pub(crate) fn shape_spec_for(map_type: MapType, coords: &[TriCoord], rng: &mut impl Rng) -> ShapeSpec {
    let max_radius = max_radius_of(coords);
    match map_type {
        MapType::Drylands => ShapeSpec {
            // One center covering (and slightly overshooting) the whole map -> ~zero land
            // penalty everywhere.
            land_centers: vec![FalloffCenter {
                pos: Vec2::ZERO,
                radius: max_radius * 1.5,
                strength: 1.0,
            }],
            water_pockets: random_centers(
                max_radius,
                DRYLANDS_WATER_POCKET_COUNT,
                DRYLANDS_WATER_POCKET_RADIUS_FRAC,
                1.0,
                rng,
            ),
            elevation_frequency: DRYLANDS_ELEVATION_FREQ_NUM / max_radius as f64,
            target_water_fraction: Some(rng.random_range(DRYLANDS_WATER_FRACTION)),
        },
        MapType::Lakes => ShapeSpec {
            land_centers: vec![FalloffCenter {
                pos: Vec2::ZERO,
                radius: max_radius * 1.5,
                strength: 1.0,
            }],
            water_pockets: random_centers(
                max_radius,
                LAKES_WATER_POCKET_COUNT,
                LAKES_WATER_POCKET_RADIUS_FRAC,
                1.0,
                rng,
            ),
            elevation_frequency: LAKES_ELEVATION_FREQ_NUM / max_radius as f64,
            target_water_fraction: Some(rng.random_range(LAKES_WATER_FRACTION)),
        },
        MapType::Continents => ShapeSpec {
            land_centers: random_centers(max_radius, CONTINENTS_CENTER_COUNT, CONTINENTS_RADIUS_FRAC, 1.0, rng),
            water_pockets: vec![],
            elevation_frequency: CONTINENTS_ELEVATION_FREQ_NUM / max_radius as f64,
            target_water_fraction: Some(rng.random_range(CONTINENTS_WATER_FRACTION)),
        },
        MapType::Pangea => {
            let mut land_centers = vec![FalloffCenter {
                pos: Vec2::ZERO,
                radius: rng.random_range(PANGEA_RADIUS_FRAC) * max_radius,
                strength: 1.0,
            }];
            land_centers.extend(random_centers(
                max_radius,
                PANGEA_EXTRA_ISLAND_COUNT,
                PANGEA_EXTRA_ISLAND_RADIUS_FRAC,
                1.0,
                rng,
            ));
            ShapeSpec {
                land_centers,
                water_pockets: vec![],
                elevation_frequency: PANGEA_ELEVATION_FREQ_NUM / max_radius as f64,
                target_water_fraction: Some(rng.random_range(PANGEA_WATER_FRACTION)),
            }
        }
        MapType::Archipelago => ShapeSpec {
            land_centers: random_centers(max_radius, ARCHIPELAGO_CENTER_COUNT, ARCHIPELAGO_RADIUS_FRAC, 1.0, rng),
            water_pockets: vec![],
            elevation_frequency: ARCHIPELAGO_ELEVATION_FREQ_NUM / max_radius as f64,
            target_water_fraction: Some(rng.random_range(ARCHIPELAGO_WATER_FRACTION)),
        },
        MapType::Waterworld => ShapeSpec {
            land_centers: vec![FalloffCenter {
                pos: Vec2::ZERO,
                radius: rng.random_range(WATERWORLD_RADIUS_FRAC) * max_radius,
                strength: 1.0,
            }],
            water_pockets: vec![],
            elevation_frequency: WATERWORLD_ELEVATION_FREQ_NUM / max_radius as f64,
            target_water_fraction: Some(rng.random_range(WATERWORLD_WATER_FRACTION)),
        },
    }
}

/// Uniformly-random centers within the map disc, count/radius drawn from the given ranges.
fn random_centers(
    max_radius: f32,
    count_range: RangeInclusive<u32>,
    radius_frac_range: RangeInclusive<f32>,
    strength: f32,
    rng: &mut impl Rng,
) -> Vec<FalloffCenter> {
    let count = rng.random_range(count_range);
    (0..count)
        .map(|_| {
            let angle = rng.random_range(0.0..TAU);
            let dist = rng.random_range(0.0..max_radius * 0.6);
            let pos = Vec2::new(angle.cos(), angle.sin()) * dist;
            let radius = rng.random_range(radius_frac_range.clone()) * max_radius;
            FalloffCenter { pos, radius, strength }
        })
        .collect()
}

/// Reproduces the exact old single-center-at-origin behavior, for `TileMap::generate_hexagon`
/// and any caller that doesn't go through the new per-`MapType` pipeline. No target water
/// fraction — water is left wherever the fixed `SEA_LEVEL` constant naturally falls.
pub(crate) fn legacy_single_island(coords: &[TriCoord]) -> ShapeSpec {
    let max_radius = max_radius_of(coords);
    ShapeSpec {
        land_centers: vec![FalloffCenter {
            pos: Vec2::ZERO,
            radius: max_radius,
            strength: 1.0,
        }],
        water_pockets: vec![],
        elevation_frequency: 1.5 / max_radius as f64,
        target_water_fraction: None,
    }
}

/// Combined elevation penalty for one world position: the best (lowest) land-center penalty,
/// plus the strongest applicable water-pocket bonus.
pub(crate) fn elevation_penalty(pos: Vec2, shape: &ShapeSpec) -> f32 {
    let land_penalty = shape
        .land_centers
        .iter()
        .map(|c| {
            let d = pos.distance(c.pos) / c.radius;
            d * d * c.strength
        })
        .fold(f32::MAX, f32::min);
    let water_bonus = shape
        .water_pockets
        .iter()
        .map(|c| {
            let d = (pos.distance(c.pos) / c.radius).clamp(0.0, 1.0);
            (1.0 - d) * (1.0 - d) * c.strength
        })
        .fold(0.0f32, f32::max);
    land_penalty + water_bonus
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use triactory_shared::grid::hexagon_tiles;

    #[test]
    fn shape_spec_for_matches_center_count_range() {
        let coords = hexagon_tiles(21).unwrap();
        let cases: [(MapType, RangeInclusive<u32>); 2] = [
            (MapType::Continents, CONTINENTS_CENTER_COUNT),
            (MapType::Archipelago, ARCHIPELAGO_CENTER_COUNT),
        ];
        for (map_type, range) in cases {
            let mut rng = ChaCha8Rng::seed_from_u64(42);
            let spec = shape_spec_for(map_type, &coords, &mut rng);
            assert!(
                range.contains(&(spec.land_centers.len() as u32)),
                "{map_type:?} produced {} land centers, expected within {range:?}",
                spec.land_centers.len()
            );
        }
    }

    #[test]
    fn target_water_fraction_is_within_each_map_types_range() {
        let coords = hexagon_tiles(21).unwrap();
        let cases: [(MapType, RangeInclusive<f32>); 6] = [
            (MapType::Drylands, DRYLANDS_WATER_FRACTION),
            (MapType::Lakes, LAKES_WATER_FRACTION),
            (MapType::Continents, CONTINENTS_WATER_FRACTION),
            (MapType::Pangea, PANGEA_WATER_FRACTION),
            (MapType::Archipelago, ARCHIPELAGO_WATER_FRACTION),
            (MapType::Waterworld, WATERWORLD_WATER_FRACTION),
        ];
        for (map_type, range) in cases {
            for seed in [1u64, 2, 3] {
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                let spec = shape_spec_for(map_type, &coords, &mut rng);
                let target = spec.target_water_fraction.expect("real MapType always sets a target");
                assert!(
                    range.contains(&target),
                    "{map_type:?} target_water_fraction {target} outside {range:?}"
                );
            }
        }
    }

    #[test]
    fn shape_spec_drylands_has_water_pockets_and_one_dominant_land_center() {
        let coords = hexagon_tiles(21).unwrap();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let spec = shape_spec_for(MapType::Drylands, &coords, &mut rng);
        assert_eq!(spec.land_centers.len(), 1);
        assert!(DRYLANDS_WATER_POCKET_COUNT.contains(&(spec.water_pockets.len() as u32)));
    }

    #[test]
    fn shape_spec_pangea_has_one_dominant_land_center_plus_occasional_islands() {
        let coords = hexagon_tiles(21).unwrap();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let spec = shape_spec_for(MapType::Pangea, &coords, &mut rng);
        assert!(!spec.land_centers.is_empty());
        let extra = spec.land_centers.len() as u32 - 1;
        assert!(PANGEA_EXTRA_ISLAND_COUNT.contains(&extra));
    }

    #[test]
    fn same_seed_is_deterministic() {
        let coords = hexagon_tiles(21).unwrap();
        let mut a = ChaCha8Rng::seed_from_u64(7);
        let mut b = ChaCha8Rng::seed_from_u64(7);
        let spec_a = shape_spec_for(MapType::Archipelago, &coords, &mut a);
        let spec_b = shape_spec_for(MapType::Archipelago, &coords, &mut b);
        assert_eq!(spec_a.land_centers.len(), spec_b.land_centers.len());
        for (ca, cb) in spec_a.land_centers.iter().zip(spec_b.land_centers.iter()) {
            assert_eq!(ca.pos, cb.pos);
            assert_eq!(ca.radius, cb.radius);
        }
    }

    #[test]
    fn different_seeds_usually_differ() {
        let coords = hexagon_tiles(21).unwrap();
        let mut a = ChaCha8Rng::seed_from_u64(1);
        let mut b = ChaCha8Rng::seed_from_u64(2);
        let spec_a = shape_spec_for(MapType::Archipelago, &coords, &mut a);
        let spec_b = shape_spec_for(MapType::Archipelago, &coords, &mut b);
        let positions_match = spec_a.land_centers.len() == spec_b.land_centers.len()
            && spec_a
                .land_centers
                .iter()
                .zip(spec_b.land_centers.iter())
                .all(|(ca, cb)| ca.pos == cb.pos);
        assert!(!positions_match, "expected shapes to differ across seeds");
    }
}
