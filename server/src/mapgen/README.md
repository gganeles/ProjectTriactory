# `server/src/mapgen` — seeded map generation

Runs once per match during the **Starting** phase. Deterministic from a seed (replayable maps,
testable generation). Pure library functions where possible so they can be unit-tested and
reused by a future map-preview tool.

## Planned files

- `mod.rs` — orchestration: seed → terrain → biome placement → entity spawning, filling the
  server's `TileMap` resource and spawning biome entities.
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
