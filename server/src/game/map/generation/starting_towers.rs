//! Picks each player's starting town from the already-placed Biome Towns, spread apart via the
//! same greedy farthest-point sampling used for town placement itself (`generation::farthest_point`).

use rand::Rng;
use triactory_shared::grid::TriCoord;

use super::farthest_point::farthest_point_sample;

/// Returns exactly `num_players` towns; index `i` is `PlayerSlot(i)`'s starting town.
/// Precondition (enforced by the caller, `TileMap::generate`): `towns.len() >= num_players`.
pub(crate) fn select_starting_towers(towns: &[TriCoord], num_players: u8, rng: &mut impl Rng) -> Vec<TriCoord> {
    debug_assert!(towns.len() >= num_players as usize);
    farthest_point_sample(towns, num_players as usize, rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn select_starting_towers_returns_num_players_distinct_entries_from_towns() {
        let towns: Vec<TriCoord> = (0..10).map(|i| TriCoord::new(i, -i, 1)).collect();
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let starting = select_starting_towers(&towns, 5, &mut rng);
        assert_eq!(starting.len(), 5);
        let unique: std::collections::HashSet<_> = starting.iter().collect();
        assert_eq!(unique.len(), 5);
        for t in &starting {
            assert!(towns.contains(t));
        }
    }

    #[test]
    fn select_starting_towers_spreads_across_a_clustered_and_a_spread_group() {
        // 5 towns tightly clustered near the origin, 5 towns far apart around a ring.
        let mut towns: Vec<TriCoord> = (0..5).map(|i| TriCoord::new(i, -i, 1)).collect();
        towns.extend([
            TriCoord::new(40, -39, 0),
            TriCoord::new(-40, 41, 0),
            TriCoord::new(40, 0, -39),
            TriCoord::new(-40, 0, 41),
            TriCoord::new(0, 40, -39),
        ]);
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let starting = select_starting_towers(&towns, 5, &mut rng);
        let spread_group = &towns[5..10];
        let from_spread_group = starting.iter().filter(|t| spread_group.contains(t)).count();
        assert!(
            from_spread_group >= 4,
            "expected selection to favor the spread-out group, got {starting:?}"
        );
    }

    #[test]
    fn select_starting_towers_is_deterministic_for_same_seed() {
        let towns: Vec<TriCoord> = (0..10).map(|i| TriCoord::new(i, -i, 1)).collect();
        let mut a = ChaCha8Rng::seed_from_u64(9);
        let mut b = ChaCha8Rng::seed_from_u64(9);
        assert_eq!(
            select_starting_towers(&towns, 4, &mut a),
            select_starting_towers(&towns, 4, &mut b)
        );
    }
}
