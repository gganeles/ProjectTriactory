# `server/src/game/map` — seeded map generation

Runs once per match during the **Starting** phase. Deterministic from a seed (replayable maps,
testable generation). Pure library functions where possible so they can be unit-tested and
reused by a future map-preview tool.

## Files

- `mod.rs` — `TileMap` resource (`HashMap<TriCoord, Terrain>`), `MapGenConfig` (currently just
  `edge_tiles`, defaulting to `shared::config::DEFAULT_EDGE_TILES`), and
  `TileMap::generate_hexagon(edge_tiles)` / `generate_map_on_enter` (runs on
  `AppState::Game` entry), which lay out the hexagon shape and default every tile to
  `Terrain::Field` pending real terrain generation.

The hexagonal map *shape* itself (`hexagon_tiles(edge_tiles) -> Vec<TriCoord>`) now lives in
[`shared/src/grid/hexagon.rs`](../../../shared/src/grid/hexagon.rs) rather than here — it's pure
and seedless, and the client needs the same shape for its local map preview before real netcode
exists, so it moved to the crate both apps share.

## Planned files

- `terrain.rs` — terrain noise over the triangle grid, plus per-biome environment flavor
  (mechanics §1.9: a biome can be mountain-heavy, field-heavy, empty, etc.). Assigns each
  tile a `Terrain` from `{ Field, Mountain, Water, Empty }`.
- `biomes.rs` — biome layout and spawning:
  1. Place Biome Tower anchors on a triangular super-lattice with spacing ≥ 6 lane units
     (guarantees max-level territories cannot overlap). All BTs are **upward** triangles,
     per the `shared/grid` convention.
  2. Claim each anchor's 13-tile neighborhood (`BIOME_OFFSETS` from `shared/grid/biome_shape`).
  3. Leave everything else as neutral wilderness — that is what biome territory grows into on
     level-up.
  4. Spawn per biome: `Biome` entity with `BiomeTower`, `BiomeLevel(1)`, `BiomeCapacity`,
     unowned `BiomeOwner`, and the initial `BiomeArcher` (BA) unit.
  5. Pick each player's starting biome (BTF) spaced fairly apart.

## Design rules

- Everything derives from the seed — no `rand::thread_rng()`.
- Unit test: generated biome claims never overlap, every biome has exactly 13 tiles, starting
  biomes respect minimum separation.
