//! The player-count + `MapType`-aware generation pipeline: sizing -> shape -> terrain -> town
//! placement -> starting towers -> initial texture spread. See this crate's
//! `server/src/game/map/README.md` for the full pipeline description; `super::TileMap::generate`
//! is the entry point that composes all of these stages.

mod farthest_point;
mod rng;
mod shape;
mod sizing;
mod starting_towers;
mod texture_spread;
mod towns;

pub(crate) use rng::{Stage, rng_for};
pub(crate) use shape::{ShapeSpec, elevation_penalty, legacy_single_island, shape_spec_for};
pub(crate) use sizing::{edge_tiles_for, town_count_for};
pub(crate) use starting_towers::select_starting_towers;
pub(crate) use texture_spread::spread_textures;
pub(crate) use towns::place_towns;

use triactory_shared::grid::TriCoord;

/// World-space edge length tiles are generated at. Only used as a coordinate scale for noise/
/// distance sampling — unrelated to the client's rendering `EDGE_LEN`, since these coordinates
/// are arbitrary as long as they're consistent within one generation pass.
pub(crate) const EDGE_LEN: f32 = 1.0;

/// The distance from the world origin to the farthest tile — the "radius" every falloff shape
/// (see `shape.rs`) is expressed relative to.
pub(crate) fn max_radius_of(coords: &[TriCoord]) -> f32 {
    coords
        .iter()
        .map(|c| c.center_world(EDGE_LEN).length())
        .fold(0.0f32, f32::max)
        .max(1.0)
}

/// Errors from [`super::TileMap::generate`].
#[derive(Debug)]
pub enum MapGenError {
    Hex(triactory_shared::grid::HexMapError),
    InvalidPlayerCount(u8),
    NotEnoughTowns { needed: u8, placed: usize },
}

impl From<triactory_shared::grid::HexMapError> for MapGenError {
    fn from(e: triactory_shared::grid::HexMapError) -> Self {
        Self::Hex(e)
    }
}

impl std::fmt::Display for MapGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hex(e) => write!(f, "{e}"),
            Self::InvalidPlayerCount(n) => write!(
                f,
                "num_players must be in {}..={}, got {n}",
                triactory_shared::config::MIN_PLAYERS,
                triactory_shared::config::MAX_PLAYERS
            ),
            Self::NotEnoughTowns { needed, placed } => write!(
                f,
                "could only place {placed} town(s), but {needed} player(s) each need a starting town"
            ),
        }
    }
}

impl std::error::Error for MapGenError {}
