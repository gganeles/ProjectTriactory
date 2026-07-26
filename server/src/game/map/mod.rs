//! Seeded map generation, run once per match during the **Starting** phase.
//!
//! Covers the hexagonal shape of the map (`hexagon_tiles`, which lives in
//! `shared/src/grid/hexagon.rs` since the client's local map preview needs it too) and, via
//! `terrain.rs`, the elevation/moisture-noise `Terrain`/`TerrainType` for every tile. Biome
//! (territory) placement is a separate follow-up step (`biomes.rs` — see this folder's
//! `README.md`).

mod terrain;

use bevy::prelude::{Commands, Res, Resource};
use std::collections::HashMap;
use triactory_shared::game::map::terrain::TileData;
use triactory_shared::grid::{HexMapError, TriCoord, hexagon_tiles};

/// The server's authoritative terrain lookup. Tiles are not entities; clients learn terrain
/// through `TilesRevealed` messages as fog lifts (see `shared/src/protocol`).
#[derive(Resource, Debug, Default)]
pub struct TileMap {
    pub tiles: HashMap<TriCoord, TileData>,
}

impl TileMap {
    /// Generates a hexagonal map whose edge row has `edge_tiles` triangles, with terrain and
    /// biome coloring derived from elevation/moisture noise seeded by `seed`. Deterministic:
    /// the same `edge_tiles`/`seed` always produces the same map.
    pub fn generate_hexagon(edge_tiles: i32, seed: u32) -> Result<Self, HexMapError> {
        let coords = hexagon_tiles(edge_tiles)?;
        let tiles = terrain::generate(&coords, seed);
        Ok(Self { tiles })
    }
}

/// How large a map to generate. A real lobby will eventually own/replace this; for now it's
/// just a resource so the generation system doesn't hardcode a magic number.
#[derive(Resource, Debug, Clone, Copy)]
pub struct MapGenConfig {
    pub edge_tiles: i32,
    pub seed: u32,
}

impl Default for MapGenConfig {
    fn default() -> Self {
        Self {
            edge_tiles: triactory_shared::config::DEFAULT_EDGE_TILES,
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
    let map = TileMap::generate_hexagon(config.edge_tiles, config.seed)
        .expect("MapGenConfig::edge_tiles must be odd and >= 3");
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
        let m = (triactory_shared::config::DEFAULT_EDGE_TILES - 1) / 2;
        assert_eq!(map.tiles.len() as i32, 6 * m * m);
    }
}
