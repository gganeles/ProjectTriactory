//! Seeded map generation, run once per match during the **Starting** phase.
//!
//! Currently covers the hexagonal shape of the map (`hexagon_tiles`, which now lives in
//! `shared/src/grid/hexagon.rs` since the client's local map preview needs it too). Terrain
//! flavor and biome placement are separate follow-up steps (`terrain.rs`, `biomes.rs` — see
//! this folder's `README.md`) that will fill in the `Terrain` this module currently defaults to
//! `Field`.

use bevy::prelude::{Commands, Res, Resource};
use std::collections::HashMap;
use triactory_shared::components::tile::Terrain;
use triactory_shared::grid::{HexMapError, TriCoord, hexagon_tiles};

/// The server's authoritative terrain lookup. Tiles are not entities; clients learn terrain
/// through `TilesRevealed` messages as fog lifts (see `shared/src/protocol`).
#[derive(Resource, Debug, Default)]
pub struct TileMap {
    pub tiles: HashMap<TriCoord, Terrain>,
}

impl TileMap {
    /// Generates a hexagonal map whose edge row has `edge_tiles` triangles, with every tile
    /// defaulted to `Terrain::Field` pending real terrain generation.
    pub fn generate_hexagon(edge_tiles: i32) -> Result<Self, HexMapError> {
        let tiles = hexagon_tiles(edge_tiles)?
            .into_iter()
            .map(|coord| (coord, Terrain::default()))
            .collect();
        Ok(Self { tiles })
    }
}

/// How large a map to generate. A real lobby will eventually own/replace this; for now it's
/// just a resource so the generation system doesn't hardcode a magic number.
#[derive(Resource, Debug, Clone, Copy)]
pub struct MapGenConfig {
    pub edge_tiles: i32,
}

impl Default for MapGenConfig {
    fn default() -> Self {
        Self {
            edge_tiles: triactory_shared::config::DEFAULT_EDGE_TILES,
        }
    }
}

/// Run on [`triactory_shared::AppState::Game`] entry: builds the hexagonal [`TileMap`] and
/// inserts it as a resource.
pub fn generate_map_on_enter(mut commands: Commands, config: Res<MapGenConfig>) {
    let map = TileMap::generate_hexagon(config.edge_tiles)
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
        let map = TileMap::generate_hexagon(15).unwrap();
        assert_eq!(map.tiles.len(), 294);
        assert!(map.tiles.values().all(|t| *t == Terrain::Field));
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
        assert_eq!(map.tiles.len(), 294);
    }
}
