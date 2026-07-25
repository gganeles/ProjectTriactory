# `server/src/game/map` — seeded map generation

Runs once per match during the **Starting** phase. Deterministic from a seed (replayable maps,
testable generation). Pure library functions where possible so they can be unit-tested and
reused by a future map-preview tool.

## Files

- `mod.rs` — `TileMap` resource (`HashMap<TriCoord, TileData>`), `MapGenConfig` (`edge_tiles`,
  defaulting to `shared::config::DEFAULT_EDGE_TILES`, and `seed`), and
  `TileMap::generate_hexagon(edge_tiles, seed)` / `generate_map_on_enter` (runs on
  `AppState::Game` entry), which lay out the hexagon shape and delegate terrain generation to
  `terrain.rs`.
- `terrain.rs` — elevation + moisture noise (`noise::Fbm<Perlin>`, seeded, deterministic) over
  the triangle grid, with a radial falloff subtracted from elevation so the map forms a single
  island rather than tiling forever. Each tile's elevation/moisture sample is classified into
  both the gameplay-authoritative `Terrain` (`{ Field, Mountain, Water, Empty }`, via
  `SEA_LEVEL`/`MOUNTAIN_LEVEL` thresholds) and the cosmetic `TerrainType`
  (`shared/src/components/terrain_type.rs` — the elevation/moisture biome table from Amit
  Patel's polygon map generation article), bundled together as `TileData`. `Terrain::Empty` is
  not produced by this table — it's reserved for the neutral wilderness gaps `biomes.rs` (below)
  will carve out.

The hexagonal map *shape* itself (`hexagon_tiles(edge_tiles) -> Vec<TriCoord>`) now lives in
[`shared/src/grid/hexagon.rs`](../../../shared/src/grid/hexagon.rs) rather than here — it's pure
and seedless, and the client needs the same shape for its local map preview before real netcode
exists, so it moved to the crate both apps share.

## Planned files

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
