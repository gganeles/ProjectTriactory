# `client/assets/textures`

Source images for `client/src/rendering`:

- Terrain tile textures (Field, Mountain, Water, Empty) — per `shared::game::map::terrain::Terrain`.
- Per-owner biome texture variants / tint maps (mechanics §1.10: a biome's look adapts to its
  occupying player) — consumed by `rendering/terrain.rs`.
- Entity sprites: hero, Biome Archer, Biome Tower, NRP, ARP.
- Fog overlay textures (unexplored / explored dimming) — consumed by `rendering/fog.rs`.
- UI chrome: buttons, panels, icons for `client/src/ui`.

Packed into a single texture atlas at build/load time (see `rendering/terrain.rs`,
`rendering/entities.rs`) to keep draw calls low on mobile GPUs — avoid adding stray
one-off-sized images that don't atlas cleanly.
