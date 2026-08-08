//! Multi-source BFS from every starting tower simultaneously, over non-water tiles only.
//! Round-synchronized (process one full BFS "ring" at a time) rather than a single shared FIFO
//! queue, specifically so tiles equidistant from two starting towers get a fair, seeded coin
//! flip instead of a bias toward whichever tower happens to be first in `starting_towers`.
//!
//! Water tiles are always left unclaimed (`None`/absent from the map) — per current design,
//! water almost never carries a player texture. Extension point: a future "certain character
//! type" override (no such character concept exists yet) would need its own pass after this
//! one, not a change to this function.
//!
//! This BFS only reaches land connected to a starting tower through a chain of non-water edges —
//! a landmass separated from every starting tower by water is left unclaimed by this function
//! alone. `TileMap::generate` (in `server/src/game/map/mod.rs`) covers that gap with a
//! nearest-starting-tower-by-distance fallback pass afterward, so every non-water tile ends up
//! owned by exactly one player slot in the end.

use rand::Rng;
use std::collections::HashMap;
use triactory_shared::game::map::generation::PlayerSlot;
use triactory_shared::game::map::terrain::{Terrain, TileData};
use triactory_shared::grid::TriCoord;

pub(crate) fn spread_textures(
    tiles: &HashMap<TriCoord, TileData>,
    starting_towers: &[TriCoord],
    rng: &mut impl Rng,
) -> HashMap<TriCoord, PlayerSlot> {
    let mut owner: HashMap<TriCoord, PlayerSlot> = HashMap::new();
    let mut frontier: Vec<TriCoord> = Vec::with_capacity(starting_towers.len());
    for (i, &t) in starting_towers.iter().enumerate() {
        owner.insert(t, PlayerSlot(i as u8));
        frontier.push(t);
    }

    while !frontier.is_empty() {
        // tile -> every slot that can reach it from the current ring
        let mut candidates: HashMap<TriCoord, Vec<PlayerSlot>> = HashMap::new();
        for &tile in &frontier {
            let slot = owner[&tile];
            for n in tile.edge_neighbors() {
                let Some(data) = tiles.get(&n) else {
                    continue;
                };
                if data.terrain == Terrain::Water || owner.contains_key(&n) {
                    continue;
                }
                candidates.entry(n).or_default().push(slot);
            }
        }

        // Deterministic iteration order (TriCoord: Ord) so the sequence of rng draws below is
        // reproducible regardless of HashMap iteration order.
        let mut keys: Vec<TriCoord> = candidates.keys().copied().collect();
        keys.sort();

        let mut next_frontier = Vec::with_capacity(keys.len());
        for tile in keys {
            let mut slots = candidates.remove(&tile).unwrap();
            slots.sort_by_key(|s| s.0);
            slots.dedup();
            let winner = if slots.len() == 1 {
                slots[0]
            } else {
                slots[rng.random_range(0..slots.len())]
            };
            owner.insert(tile, winner);
            next_frontier.push(tile);
        }
        frontier = next_frontier;
    }

    owner
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::generation::{legacy_single_island, select_starting_towers, shape_spec_for};
    use crate::game::map::terrain;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use triactory_shared::game::map::generation::MapType;
    use triactory_shared::grid::hexagon_tiles;

    #[test]
    fn spread_textures_never_claims_water() {
        let coords = hexagon_tiles(21).unwrap();
        let shape = legacy_single_island(&coords);
        let tiles = terrain::generate(&coords, 42, &shape);
        let starting: Vec<TriCoord> = tiles
            .iter()
            .filter(|(c, d)| d.terrain != Terrain::Water && c.is_upward())
            .map(|(c, _)| *c)
            .take(3)
            .collect();
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let owners = spread_textures(&tiles, &starting, &mut rng);
        for coord in owners.keys() {
            assert_ne!(tiles[coord].terrain, Terrain::Water);
        }
    }

    #[test]
    fn spread_textures_covers_every_reachable_land_tile() {
        let coords = hexagon_tiles(21).unwrap();
        let shape = legacy_single_island(&coords);
        let tiles = terrain::generate(&coords, 42, &shape);
        let land: Vec<TriCoord> = tiles
            .iter()
            .filter(|(_, d)| d.terrain != Terrain::Water)
            .map(|(c, _)| *c)
            .collect();
        let starting: Vec<TriCoord> = land
            .iter()
            .copied()
            .filter(|c| c.is_upward())
            .take(2)
            .collect();
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let owners = spread_textures(&tiles, &starting, &mut rng);

        // Every land tile reachable from a starting tower through non-water tiles must be owned.
        let mut reachable: std::collections::HashSet<TriCoord> = starting.iter().copied().collect();
        let mut frontier: Vec<TriCoord> = starting.clone();
        while let Some(tile) = frontier.pop() {
            for n in tile.edge_neighbors() {
                if let Some(data) = tiles.get(&n)
                    && data.terrain != Terrain::Water
                    && reachable.insert(n)
                {
                    frontier.push(n);
                }
            }
        }
        for coord in &reachable {
            assert!(owners.contains_key(coord), "{coord:?} should be owned");
        }
    }

    #[test]
    fn spread_textures_starting_towers_own_themselves() {
        let coords = hexagon_tiles(21).unwrap();
        let shape = legacy_single_island(&coords);
        let tiles = terrain::generate(&coords, 42, &shape);
        let starting: Vec<TriCoord> = tiles
            .iter()
            .filter(|(c, d)| d.terrain != Terrain::Water && c.is_upward())
            .map(|(c, _)| *c)
            .take(3)
            .collect();
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let owners = spread_textures(&tiles, &starting, &mut rng);
        for (i, t) in starting.iter().enumerate() {
            assert_eq!(owners[t], PlayerSlot(i as u8));
        }
    }

    #[test]
    fn spread_textures_is_deterministic_for_same_seed() {
        let coords = hexagon_tiles(21).unwrap();
        let shape = legacy_single_island(&coords);
        let tiles = terrain::generate(&coords, 42, &shape);
        let starting: Vec<TriCoord> = tiles
            .iter()
            .filter(|(c, d)| d.terrain != Terrain::Water && c.is_upward())
            .map(|(c, _)| *c)
            .take(3)
            .collect();
        let mut a = ChaCha8Rng::seed_from_u64(5);
        let mut b = ChaCha8Rng::seed_from_u64(5);
        assert_eq!(
            spread_textures(&tiles, &starting, &mut a),
            spread_textures(&tiles, &starting, &mut b)
        );
    }

    #[test]
    fn spread_textures_partitions_are_roughly_balanced() {
        // A large, land-heavy map so the flood fill has plenty of room to balance out.
        let coords = hexagon_tiles(31).unwrap();
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let shape = shape_spec_for(MapType::Drylands, &coords, &mut rng);
        let tiles = terrain::generate(&coords, 1, &shape);

        // 3 starting towers, spread apart by the same farthest-point selection real generation
        // uses, so the fixture is representative of what spread_textures actually sees.
        let candidates: Vec<TriCoord> = tiles
            .iter()
            .filter(|(c, d)| d.terrain != Terrain::Water && c.is_upward())
            .map(|(c, _)| *c)
            .collect();
        let mut select_rng = ChaCha8Rng::seed_from_u64(3);
        let starting = select_starting_towers(&candidates, 3, &mut select_rng);

        let mut spread_rng = ChaCha8Rng::seed_from_u64(2);
        let owners = spread_textures(&tiles, &starting, &mut spread_rng);
        let mut counts = [0usize; 3];
        for slot in owners.values() {
            counts[slot.0 as usize] += 1;
        }
        let total: usize = counts.iter().sum();
        for count in counts {
            assert!(
                count as f32 >= total as f32 * 0.1,
                "expected a roughly balanced partition, got {counts:?}"
            );
        }
    }
}
