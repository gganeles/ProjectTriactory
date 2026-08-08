//! Seeded map generation, run once per match during the **Starting** phase.
//!
//! Covers the hexagonal shape of the map (`hexagon_tiles`, which lives in
//! `shared/src/grid/hexagon.rs` since the client's local map preview needs it too), the
//! elevation/moisture-noise `Terrain`/`TerrainType` for every tile (`terrain.rs`), and — via
//! `generation/` — the player-count/`MapType`-aware pipeline that sizes the map, places Biome
//! Towns, picks each player's starting town, and spreads an initial per-player texture
//! partition outward from them. Actual `Biome`/`BiomeTower`/`BiomeOwner` entity spawning at the
//! placed towns is a separate follow-up step (`biomes.rs` — see this folder's `README.md`).

mod generation;
mod terrain;

use bevy::prelude::{Commands, Res, Resource};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use triactory_shared::config::{MAX_PLAYERS, MIN_PLAYERS};
use triactory_shared::game::map::generation::{MapType, PlayerSlot};
use triactory_shared::game::map::terrain::{LAND_TERRAIN_TYPES, SEA_LEVEL, Terrain, TerrainType, TileData};
use triactory_shared::grid::{HexMapError, TriCoord, hexagon_tiles};

pub use generation::MapGenError;

/// The server's authoritative terrain + biome-layout lookup. Tiles are not entities; clients
/// learn terrain through `TilesRevealed` messages as fog lifts (see `shared/src/protocol`).
#[derive(Resource, Debug, Default)]
pub struct TileMap {
    pub tiles: HashMap<TriCoord, TileData>,
    /// Every placed Biome Town (map-generation-time only; spawning the actual `Biome`
    /// entity/components at these coords is future work — see this folder's README).
    pub towns: Vec<TriCoord>,
    /// Index `i` is `PlayerSlot(i)`'s starting town. Length equals the `num_players` passed to
    /// [`TileMap::generate`].
    pub starting_towers: Vec<TriCoord>,
    /// Static, generation-time-only per-tile texture ownership (see
    /// `generation/texture_spread.rs`). NOT the future live `BiomeOwner`-driven texture-swap
    /// system described in `client/src/rendering/README.md` — that's runtime,
    /// territory-capture-driven, and doesn't exist yet. Water tiles are absent from this map.
    pub initial_texture_owners: HashMap<TriCoord, PlayerSlot>,
}

impl TileMap {
    /// Generates a bare hexagonal map whose edge row has `edge_tiles` triangles, with terrain
    /// derived from elevation/moisture noise seeded by `seed`, using the original single-island
    /// falloff shape. No towns/starting towers/texture spread — kept for tests/tooling that
    /// don't care about player count or `MapType` at all. Deterministic: the same
    /// `edge_tiles`/`seed` always produces the same map.
    pub fn generate_hexagon(edge_tiles: i32, seed: u32) -> Result<Self, HexMapError> {
        let coords = hexagon_tiles(edge_tiles)?;
        let shape = generation::legacy_single_island(&coords);
        let tiles = terrain::generate(&coords, seed, &shape);
        Ok(Self {
            tiles,
            ..Default::default()
        })
    }

    /// The full pipeline: sizing -> shape/terrain -> town placement (spread across every upward
    /// tile, then land forced under any that landed on water — see the town-placement step
    /// below, needed most for `MapType::Waterworld`) -> starting towers -> initial texture
    /// spread. Deterministic: the same `(map_type, num_players, seed)` always produces the same
    /// `TileMap`.
    pub fn generate(map_type: MapType, num_players: u8, seed: u32) -> Result<Self, MapGenError> {
        if !(MIN_PLAYERS..=MAX_PLAYERS).contains(&num_players) {
            return Err(MapGenError::InvalidPlayerCount(num_players));
        }

        let edge_tiles = generation::edge_tiles_for(num_players, map_type);
        let coords = hexagon_tiles(edge_tiles)?;

        let town_count = generation::town_count_for(
            num_players,
            map_type,
            &mut generation::rng_for(seed, generation::Stage::TownCount),
        );
        let shape = generation::shape_spec_for(
            map_type,
            &coords,
            &mut generation::rng_for(seed, generation::Stage::ShapeCenters),
        );
        let mut tiles = terrain::generate(&coords, seed, &shape);
        eliminate_isolated_land(&mut tiles);

        // Sorted rather than left in HashMap iteration order: `tiles`'s hasher is randomized
        // per-process, so an unsorted `town_sites` would make `place_towns`'s rng draws pick a
        // different actual tile on every run even for the same seed, breaking determinism.
        //
        // Every upward tile is a candidate here, water included — restricting to non-water
        // tiles would leave `Waterworld` (90-100% target water) with too few, badly-clustered
        // candidates to spread `town_count` towns across. Instead, towns are placed for maximum
        // spread first, then forced onto land below: a Biome Town always needs somewhere to
        // stand, water or not.
        let mut town_sites: Vec<TriCoord> = tiles.keys().copied().filter(|c| c.is_upward()).collect();
        town_sites.sort();
        let towns = generation::place_towns(
            &town_sites,
            town_count,
            &mut generation::rng_for(seed, generation::Stage::TownPlacement),
        );
        if (towns.len() as u8) < num_players {
            return Err(MapGenError::NotEnoughTowns {
                needed: num_players,
                placed: towns.len(),
            });
        }

        for &town in &towns {
            force_land(&mut tiles, town);
            // A lone forced tile surrounded entirely by water would be a single-triangle land
            // patch with no same-type edge-neighbor — forbidden (only water may stand alone).
            // Forcing its neighbors too guarantees every forced tile always has one.
            for neighbor in town.edge_neighbors() {
                force_land(&mut tiles, neighbor);
            }
        }

        let starting_towers = generation::select_starting_towers(
            &towns,
            num_players,
            &mut generation::rng_for(seed, generation::Stage::StartingTowers),
        );
        let mut initial_texture_owners = generation::spread_textures(
            &tiles,
            &starting_towers,
            &mut generation::rng_for(seed, generation::Stage::TextureSpread),
        );

        // `spread_textures`'s BFS only reaches land connected to a starting tower through
        // non-water edges — a landmass cut off from every starting tower by water (or a
        // forced-land patch stranded in open ocean) would otherwise be left with no owner at
        // all, showing up as an unintended extra color. Every non-water tile must belong to
        // exactly one of the `num_players` slots, so whatever's left unclaimed here is handed to
        // its nearest starting tower by raw distance instead.
        let mut unclaimed: Vec<TriCoord> = tiles
            .iter()
            .filter(|(coord, data)| data.terrain != Terrain::Water && !initial_texture_owners.contains_key(coord))
            .map(|(coord, _)| *coord)
            .collect();
        unclaimed.sort();
        for coord in unclaimed {
            let nearest_slot = (0..starting_towers.len())
                .min_by_key(|&i| coord.distance(&starting_towers[i]))
                .expect("starting_towers is non-empty: num_players is checked against MIN_PLAYERS above");
            initial_texture_owners.insert(coord, PlayerSlot(nearest_slot as u8));
        }

        // Each player gets one land `TerrainType` for the whole match — a forward-looking hook
        // for a future civilization system (e.g. Egyptians on Sand, Vikings on Tundra, Eskimos
        // on Snow). Every owned tile is repainted to its owner's type below, so the number of
        // distinct rendered land colors always equals `num_players`, regardless of how many
        // `TerrainType`s the elevation-driven `classify` happened to produce.
        let mut player_biomes = LAND_TERRAIN_TYPES;
        player_biomes.shuffle(&mut generation::rng_for(seed, generation::Stage::PlayerBiomeAssignment));
        for (&coord, &slot) in &initial_texture_owners {
            if let Some(data) = tiles.get_mut(&coord) {
                data.terrain_type = player_biomes[slot.0 as usize];
            }
        }

        Ok(Self {
            tiles,
            towns,
            starting_towers,
            initial_texture_owners,
        })
    }
}

/// Forces one edge-neighbor of every land tile that has none (i.e. every edge-neighbor present
/// on the map is water) to become land too, so it's no longer a single-triangle land patch with
/// no same-type neighbor — only water may stand alone. This can happen from noise/falloff alone
/// (a thin spit of land pinched down to one tile between two water pockets), not just from the
/// forced-land-under-towns step below (which has its own, town-specific version of this same
/// guarantee). Deterministic and order-independent: which coordinate is isolated, and which of
/// its neighbors gets forced, both depend only on the (immutable at check-time) tile map and
/// `TriCoord::edge_neighbors`'s fixed order — never on `tiles`' HashMap iteration order.
fn eliminate_isolated_land(tiles: &mut HashMap<TriCoord, TileData>) {
    let isolated: Vec<TriCoord> = tiles
        .iter()
        .filter(|(coord, data)| {
            data.terrain != Terrain::Water
                && coord
                    .edge_neighbors()
                    .iter()
                    .all(|n| tiles.get(n).is_none_or(|d| d.terrain == Terrain::Water))
        })
        .map(|(&coord, _)| coord)
        .collect();
    for coord in isolated {
        if let Some(neighbor) = coord.edge_neighbors().into_iter().find(|n| tiles.contains_key(n)) {
            force_land(tiles, neighbor);
        }
    }
}

/// Forces `coord` to land if it's currently water: reclaims exactly enough elevation to clear
/// `SEA_LEVEL` (`TerrainType::classify` reads the same value, so it lands on `Sand` — the
/// classification's own lowest-elevation land band — pending this pipeline's later per-player
/// `terrain_type` overwrite). No-op for tiles that are already land or absent from `tiles`.
fn force_land(tiles: &mut HashMap<TriCoord, TileData>, coord: TriCoord) {
    if let Some(data) = tiles.get_mut(&coord)
        && data.terrain == Terrain::Water
    {
        data.elevation = SEA_LEVEL;
        data.terrain = Terrain::Field;
        data.terrain_type = TerrainType::classify(data.elevation, 0.5);
    }
}

/// Which map to generate for a match. A real lobby will eventually own/replace this; for now
/// it's just a resource so the generation system doesn't hardcode magic numbers.
#[derive(Resource, Debug, Clone, Copy)]
pub struct MapGenConfig {
    pub map_type: MapType,
    pub num_players: u8,
    pub seed: u32,
}

impl Default for MapGenConfig {
    fn default() -> Self {
        Self {
            map_type: MapType::Continents,
            num_players: MIN_PLAYERS,
            seed: random_seed(),
        }
    }
}

/// A different seed per process, so matches don't all generate the identical map. Not itself
/// deterministic — that's the point; `TileMap::generate_hexagon`'s determinism is about a *given*
/// seed always producing the same map, not about the default seed being fixed. A real lobby will
/// eventually pick (and share with clients) the seed for a match instead of this per-process
/// default. `pub(crate)` rather than private so `replication.rs`'s debug `regenerate_map_on_request`
/// can reroll it too — see that function's docs.
pub(crate) fn random_seed() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .subsec_nanos()
}

/// Run on [`triactory_shared::AppState::Game`] entry: builds the hexagonal [`TileMap`] and
/// inserts it as a resource.
pub fn generate_map_on_enter(mut commands: Commands, config: Res<MapGenConfig>) {
    let map = TileMap::generate(config.map_type, config.num_players, config.seed)
        .expect("MapGenConfig should hold a valid map_type/num_players combination");
    commands.insert_resource(map);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;
    use triactory_shared::AppState;

    #[test]
    fn generate_hexagon_populates_every_tile() {
        let map = TileMap::generate_hexagon(15, 0).unwrap();
        assert_eq!(map.tiles.len(), 294);
    }

    #[test]
    fn generate_hexagon_is_deterministic_for_a_given_seed() {
        let a = TileMap::generate_hexagon(15, 3).unwrap();
        let b = TileMap::generate_hexagon(15, 3).unwrap();
        assert_eq!(a.tiles, b.tiles);
    }

    #[test]
    fn map_is_generated_on_entering_game_state() {
        let mut app = App::new();
        app.add_plugins(StatesPlugin)
            .init_state::<AppState>()
            .init_resource::<MapGenConfig>()
            .add_systems(OnEnter(AppState::Game), generate_map_on_enter);

        app.update();
        assert!(
            app.world().get_resource::<TileMap>().is_none(),
            "map should not exist before entering Game"
        );

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();

        let map = app
            .world()
            .get_resource::<TileMap>()
            .expect("map should be generated on entering Game");
        assert!(!map.tiles.is_empty());
        assert_eq!(map.starting_towers.len(), MapGenConfig::default().num_players as usize);
    }

    const ALL_MAP_TYPES: [MapType; 6] = [
        MapType::Drylands,
        MapType::Lakes,
        MapType::Continents,
        MapType::Pangea,
        MapType::Archipelago,
        MapType::Waterworld,
    ];

    #[test]
    fn generate_is_deterministic_for_same_seed() {
        let a = TileMap::generate(MapType::Archipelago, 4, 11).unwrap();
        let b = TileMap::generate(MapType::Archipelago, 4, 11).unwrap();
        assert_eq!(a.tiles, b.tiles);
        assert_eq!(a.towns, b.towns);
        assert_eq!(a.starting_towers, b.starting_towers);
        assert_eq!(a.initial_texture_owners, b.initial_texture_owners);
    }

    #[test]
    fn generate_rejects_out_of_range_player_counts() {
        for invalid in [0, 1, MAX_PLAYERS + 1] {
            let err = TileMap::generate(MapType::Continents, invalid, 1).unwrap_err();
            assert!(matches!(err, MapGenError::InvalidPlayerCount(n) if n == invalid));
        }
    }

    #[test]
    fn generate_produces_num_players_starting_towers_for_every_map_type() {
        for num_players in MIN_PLAYERS..=MAX_PLAYERS {
            for map_type in ALL_MAP_TYPES {
                let map = TileMap::generate(map_type, num_players, 123).unwrap_or_else(|e| {
                    panic!("generate({map_type:?}, {num_players}, 123) failed: {e}")
                });
                assert_eq!(
                    map.starting_towers.len(),
                    num_players as usize,
                    "wrong starting tower count for {map_type:?}/{num_players}"
                );
            }
        }
    }

    #[test]
    fn generate_towns_are_all_non_water_and_upward() {
        // Waterworld (90-100% target water) is the strongest test of the "force land under
        // every town" step — plain Continents/Archipelago rarely leave a town's chosen site on
        // water in the first place, so this wouldn't meaningfully exercise the forcing path.
        let map = TileMap::generate(MapType::Waterworld, 6, 7).unwrap();
        for town in &map.towns {
            assert!(town.is_upward(), "{town:?} should be upward");
            assert_ne!(
                map.tiles[town].terrain,
                Terrain::Water,
                "{town:?} should not be on water"
            );
        }
    }

    #[test]
    fn generate_never_produces_an_isolated_land_tile() {
        // Only water may stand alone as a single triangle — every land tile must have at least
        // one edge-neighbor that's also land. Waterworld is included specifically because it
        // exercises the "force land under a town, plus its neighbors" path most (see
        // `force_land`'s caller), which is where a bare single-tile force (without the neighbor
        // fix) used to leave a lone land triangle surrounded by water.
        for map_type in ALL_MAP_TYPES {
            for seed in [1u32, 2, 3] {
                let map = TileMap::generate(map_type, 4, seed).unwrap();
                for (&coord, data) in &map.tiles {
                    if data.terrain == Terrain::Water {
                        continue;
                    }
                    let has_land_neighbor = coord
                        .edge_neighbors()
                        .iter()
                        .any(|n| map.tiles.get(n).is_some_and(|d| d.terrain != Terrain::Water));
                    assert!(
                        has_land_neighbor,
                        "{coord:?} ({map_type:?}/seed {seed}) is an isolated land tile with no \
                         same-type edge-neighbor"
                    );
                }
            }
        }
    }

    #[test]
    fn generate_distinct_land_colors_always_match_player_count() {
        // Every player is assigned exactly one distinct land TerrainType, and every land tile is
        // owned by exactly one player slot (see the full-coverage fallback pass in `generate`) —
        // so the number of distinct land colors rendered must always equal num_players, whatever
        // the map_type/seed.
        for num_players in MIN_PLAYERS..=MAX_PLAYERS {
            for map_type in ALL_MAP_TYPES {
                let map = TileMap::generate(map_type, num_players, 42).unwrap();
                let land_types: std::collections::HashSet<TerrainType> = map
                    .tiles
                    .values()
                    .filter(|d| d.terrain != Terrain::Water)
                    .map(|d| d.terrain_type)
                    .collect();
                assert_eq!(
                    land_types.len(),
                    num_players as usize,
                    "{map_type:?}/{num_players} players: expected {num_players} distinct land \
                     colors, got {land_types:?}"
                );
            }
        }
    }

    #[test]
    fn generate_every_non_water_tile_has_an_owner() {
        for map_type in ALL_MAP_TYPES {
            let map = TileMap::generate(map_type, 5, 99).unwrap();
            for (coord, data) in &map.tiles {
                if data.terrain != Terrain::Water {
                    assert!(
                        map.initial_texture_owners.contains_key(coord),
                        "{coord:?} ({map_type:?}) is land but has no owner"
                    );
                }
            }
        }
    }
}
