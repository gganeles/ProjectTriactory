//! One independently-seeded RNG stream per pipeline stage, so stage N's random draws never
//! shift stage N+1's draws (adding a town doesn't change which starting towers get picked).
//! Generalizes `terrain.rs`'s `seed.wrapping_add(1)` offset trick (used there for the moisture
//! noise channel) to an arbitrary number of channels via an explicit stage tag.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Stage {
    TownCount,
    ShapeCenters,
    TownPlacement,
    StartingTowers,
    TextureSpread,
    PlayerBiomeAssignment,
}

pub(crate) fn rng_for(seed: u32, stage: Stage) -> ChaCha8Rng {
    let tag = stage as u64;
    ChaCha8Rng::seed_from_u64(((seed as u64) << 8) | tag)
}
