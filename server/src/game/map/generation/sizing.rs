//! Derives map size and town count from `(num_players, map_type)`. All constants here are
//! placeholders — tune after playtesting, the *mechanism* is what matters: bigger player counts
//! and land-heavy map types get more space and more towns, with seeded per-generation variance
//! on the town count.

use rand::Rng;
use triactory_shared::config::MIN_PLAYERS;
use triactory_shared::game::map::generation::MapType;

// --- edge_tiles ---

const BASE_EDGE_TILES: i32 = 9;
const EDGE_TILES_PER_PLAYER: i32 = 3;

/// Hexagon edge length (triangles per side) for a match with `num_players` players on
/// `map_type`. Always odd and >= 3, as required by [`triactory_shared::grid::hexagon_tiles`].
pub(crate) fn edge_tiles_for(num_players: u8, map_type: MapType) -> i32 {
    let base = BASE_EDGE_TILES + EDGE_TILES_PER_PLAYER * (num_players as i32 - MIN_PLAYERS as i32);
    let scaled = (base as f32 * map_type_size_multiplier(map_type)).round() as i32;
    let odd = if scaled % 2 == 0 { scaled + 1 } else { scaled };
    odd.max(3)
}

/// Bigger multiplier = more world "canvas," so `shape.rs`'s per-type falloff centers have room
/// to spread out without crowding.
fn map_type_size_multiplier(map_type: MapType) -> f32 {
    match map_type {
        MapType::Drylands => 0.9,
        MapType::Lakes => 0.95,
        MapType::Continents => 1.0,
        MapType::Pangea => 1.0,
        MapType::Archipelago => 1.15,
        MapType::Waterworld => 1.25,
    }
}

// --- town_count ---

const BASE_TOWN_COUNT: f32 = 4.0;
const TOWN_COUNT_PER_PLAYER: f32 = 4.0;
/// +-15% seeded variance on the raw town count.
const TOWN_COUNT_VARIANCE: f32 = 0.15;

fn map_type_town_multiplier(map_type: MapType) -> f32 {
    match map_type {
        // Almost all land -> room for more towns.
        MapType::Drylands => 1.4,
        MapType::Lakes => 1.2,
        MapType::Continents => 1.0,
        MapType::Pangea => 1.0,
        MapType::Archipelago => 0.9,
        // Towns are manually forced onto land regardless (see `TileMap::generate`), but a
        // near-total-ocean world still feels sparser than a land-heavy one.
        MapType::Waterworld => 0.6,
    }
}

/// `town_count = (BASE + PER_PLAYER * num_players) * map_type_multiplier * (1 +- VARIANCE)`,
/// clamped so every player is guaranteed at least one town to start on.
pub(crate) fn town_count_for(num_players: u8, map_type: MapType, rng: &mut impl Rng) -> u32 {
    let base = BASE_TOWN_COUNT + TOWN_COUNT_PER_PLAYER * num_players as f32;
    let scaled = base * map_type_town_multiplier(map_type);
    let variance = rng.random_range((1.0 - TOWN_COUNT_VARIANCE)..=(1.0 + TOWN_COUNT_VARIANCE));
    ((scaled * variance).round() as u32).max(num_players as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    const ALL_MAP_TYPES: [MapType; 6] = [
        MapType::Drylands,
        MapType::Lakes,
        MapType::Continents,
        MapType::Pangea,
        MapType::Archipelago,
        MapType::Waterworld,
    ];

    #[test]
    fn edge_tiles_for_is_always_odd_and_at_least_3() {
        for num_players in MIN_PLAYERS..=triactory_shared::config::MAX_PLAYERS {
            for map_type in ALL_MAP_TYPES {
                let edge_tiles = edge_tiles_for(num_players, map_type);
                assert_eq!(
                    edge_tiles % 2,
                    1,
                    "edge_tiles_for({num_players}, {map_type:?}) = {edge_tiles} is not odd"
                );
                assert!(
                    edge_tiles >= 3,
                    "edge_tiles_for({num_players}, {map_type:?}) = {edge_tiles} is < 3"
                );
            }
        }
    }

    #[test]
    fn edge_tiles_for_is_monotonic_in_player_count() {
        for map_type in ALL_MAP_TYPES {
            let mut prev = edge_tiles_for(MIN_PLAYERS, map_type);
            for num_players in (MIN_PLAYERS + 1)..=triactory_shared::config::MAX_PLAYERS {
                let cur = edge_tiles_for(num_players, map_type);
                assert!(
                    cur >= prev,
                    "edge_tiles_for regressed from {prev} to {cur} going from {} to {num_players} players for {map_type:?}",
                    num_players - 1
                );
                prev = cur;
            }
        }
    }

    #[test]
    fn town_count_for_is_deterministic_for_same_seed() {
        let mut a = ChaCha8Rng::seed_from_u64(7);
        let mut b = ChaCha8Rng::seed_from_u64(7);
        assert_eq!(
            town_count_for(3, MapType::Continents, &mut a),
            town_count_for(3, MapType::Continents, &mut b)
        );
    }

    #[test]
    fn town_count_for_varies_across_seeds() {
        let counts: std::collections::HashSet<_> = (0..20u64)
            .map(|s| {
                let mut rng = ChaCha8Rng::seed_from_u64(s);
                town_count_for(3, MapType::Continents, &mut rng)
            })
            .collect();
        assert!(
            counts.len() > 1,
            "expected town_count_for to vary across seeds, got {counts:?}"
        );
    }

    #[test]
    fn town_count_for_is_never_below_num_players() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        for num_players in MIN_PLAYERS..=triactory_shared::config::MAX_PLAYERS {
            for map_type in ALL_MAP_TYPES {
                let count = town_count_for(num_players, map_type, &mut rng);
                assert!(count >= num_players as u32);
            }
        }
    }
}
