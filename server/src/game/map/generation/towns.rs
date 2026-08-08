//! Places Biome Towns as close to equidistant as possible, via greedy farthest-point sampling
//! (`generation::farthest_point`) over every non-water land tile — see [`place_towns`].

use rand::Rng;
use triactory_shared::grid::TriCoord;

use super::farthest_point::farthest_point_sample;

/// Places up to `town_count` towers on tiles drawn from `land_tiles` (caller's responsibility to
/// pre-filter: non-`Terrain::Water`, upward-only per the "Biome Towers are always upward
/// triangles" convention — see `shared/src/grid/README.md`). Spread as evenly as a greedy
/// farthest-point algorithm can manage, rather than a fixed minimum spacing — this scales
/// gracefully from crowded small maps to sparse large ones with no separate "give up and relax
/// spacing" fallback needed. Never panics: returns `min(town_count, land_tiles.len())` towns.
pub(crate) fn place_towns(land_tiles: &[TriCoord], town_count: u32, rng: &mut impl Rng) -> Vec<TriCoord> {
    farthest_point_sample(land_tiles, town_count as usize, rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use triactory_shared::grid::hexagon_tiles;

    fn upward_land_tiles(edge_tiles: i32) -> Vec<TriCoord> {
        hexagon_tiles(edge_tiles)
            .unwrap()
            .into_iter()
            .filter(|c| c.is_upward())
            .collect()
    }

    #[test]
    fn place_towns_spreads_towns_apart() {
        let land_tiles = upward_land_tiles(41);
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let towns = place_towns(&land_tiles, 10, &mut rng);
        assert_eq!(towns.len(), 10);
        let mut min_pairwise = i32::MAX;
        for i in 0..towns.len() {
            for j in (i + 1)..towns.len() {
                min_pairwise = min_pairwise.min(towns[i].distance(&towns[j]));
            }
        }
        // With only 10 towns spread over a large board, a greedy farthest-point search should
        // comfortably clear the old fixed "6 lane units" minimum this replaces.
        assert!(
            min_pairwise >= 6,
            "expected towns to be well spread, got min pairwise distance {min_pairwise}"
        );
    }

    #[test]
    fn place_towns_never_exceeds_available_land_tiles() {
        let land_tiles = upward_land_tiles(9);
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let towns = place_towns(&land_tiles, 1000, &mut rng);
        assert!(towns.len() <= land_tiles.len());
    }

    #[test]
    fn place_towns_is_deterministic_for_same_seed() {
        let land_tiles = upward_land_tiles(41);
        let mut a = ChaCha8Rng::seed_from_u64(3);
        let mut b = ChaCha8Rng::seed_from_u64(3);
        assert_eq!(
            place_towns(&land_tiles, 10, &mut a),
            place_towns(&land_tiles, 10, &mut b)
        );
    }

    #[test]
    fn place_towns_differs_across_seeds() {
        let land_tiles = upward_land_tiles(41);
        let mut a = ChaCha8Rng::seed_from_u64(1);
        let mut b = ChaCha8Rng::seed_from_u64(2);
        assert_ne!(
            place_towns(&land_tiles, 10, &mut a),
            place_towns(&land_tiles, 10, &mut b)
        );
    }
}
