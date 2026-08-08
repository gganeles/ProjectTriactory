# `server/src/game/map` — seeded map generation

Runs once per match during the **Starting** phase. Deterministic from a seed (replayable maps,
testable generation). Pure library functions where possible so they can be unit-tested and
reused by a future map-preview tool.

## Files

- `mod.rs` — `TileMap` resource (`tiles: HashMap<TriCoord, TileData>` plus the generation
  pipeline's output: `towns`, `starting_towers`, `initial_texture_owners` — see `generation/`
  below), `MapGenConfig` (`map_type`, `num_players`, `seed`), and
  `TileMap::generate_hexagon(edge_tiles, seed)` (bare hexagon + legacy single-island terrain, no
  pipeline — kept for tests/tooling) / `TileMap::generate(map_type, num_players, seed)` (the full
  pipeline) / `generate_map_on_enter` (runs on `AppState::Game` entry, calls `generate`).
- `terrain.rs` — elevation + moisture noise (`noise::Fbm<Perlin>`, seeded, deterministic) over
  the triangle grid, with a `ShapeSpec` (from `generation/shape.rs`) falloff subtracted from
  elevation so the map forms the target land shape rather than tiling forever. If
  `ShapeSpec::target_water_fraction` is set, the raw elevation distribution is then remapped
  (`remap_to_water_fraction`) so *exactly* that fraction of tiles fall below `SEA_LEVEL` — a
  percentile pivot onto `SEA_LEVEL` itself, so `Terrain`/`TerrainType` still agree afterward — 
  rather than leaving the water amount to emerge incidentally from the noise/falloff geometry.
  Each tile's elevation/moisture sample is classified into both the gameplay-authoritative
  `Terrain` (`{ Field, Mountain, Water, Empty }`, via `SEA_LEVEL`/`MOUNTAIN_LEVEL` thresholds)
  and the elevation-driven `TerrainType` (`shared/src/game/map/terrain.rs` — the elevation/
  moisture biome table from Amit Patel's polygon map generation article), bundled together as
  `TileData`. `Terrain::Empty` is not produced by this table — it's reserved for the neutral
  wilderness gaps a future `biomes.rs` (below) will carve out. `TerrainType` as computed here is
  only ever a placeholder for land tiles, though — see `mod.rs`'s per-player overwrite below,
  which is what actually decides every land tile's rendered color.
- `generation/` — the player-count + `MapType`-aware pipeline, composed by
  `TileMap::generate`:
  - `sizing.rs` — derives `edge_tiles` and town count from `(num_players, map_type)`, with
    seeded variance on the town count.
  - `shape.rs` — `ShapeSpec`/`FalloffCenter`: per-`MapType` recipe of noise-falloff centers
    (`Continents`/`Pangea` = few large centers, `Archipelago` = many small ones,
    `Drylands`/`Lakes` = one map-covering center plus small water-forcing pockets, `Waterworld` =
    one small center for whatever sliver of land its near-total-water target leaves) controlling
    where water *clusters*, plus a `target_water_fraction` sampled from a per-`MapType` range
    controlling how *much* of the map is water — `terrain.rs` enforces the latter exactly via a
    percentile remap.
  - `farthest_point.rs` — `farthest_point_sample`: greedy farthest-point sampling (pick a random
    first point, repeatedly add whichever remaining candidate maximizes its minimum distance to
    points already chosen) — the shared "as equidistant as possible" placement algorithm used by
    both files below.
  - `towns.rs` — `place_towns`: spreads Biome Towns across every upward tile (water included) via
    `farthest_point_sample`; `TileMap::generate` then forces land under any town that landed on
    water, *and* under all of that town's edge-neighbors too (`force_land`, called twice per
    town) — a lone forced tile with only water neighbors would be a single-triangle land patch
    with no same-type neighbor, which every land tile is required to avoid (only water may stand
    alone). This matters most for `Waterworld`, where naturally-occurring land can be scarce or
    absent. `TileMap::generate` also runs `eliminate_isolated_land` right after `terrain::generate`
    — the noise/falloff shape alone can occasionally pinch a spit of land down to one tile between
    two water pockets, so this same guarantee is enforced map-wide, not just for forced towns.
  - `starting_towers.rs` — `select_starting_towers`: picks one starting town per player slot
    from the already-placed towns, via the same `farthest_point_sample`.
  - `texture_spread.rs` — `spread_textures`: round-synchronized multi-source BFS flood-fill
    from every starting town, over non-water tiles, giving each reachable land tile an initial
    `PlayerSlot` owner (see `TileMap::initial_texture_owners`'s docs for how this differs from
    the future live `BiomeOwner`-driven texture swap). Land cut off from every starting tower by
    water is left unclaimed by this BFS alone — `TileMap::generate` covers that gap with a
    nearest-starting-tower-by-distance fallback pass afterward, so every non-water tile ends up
    owned by exactly one player slot.
  - `rng.rs` — one independently-seeded `ChaCha8Rng` stream per pipeline stage.

  Once every land tile has an owner, `TileMap::generate` shuffles the 6 land `TerrainType`
  variants (`shared/src/game/map/terrain.rs`'s `LAND_TERRAIN_TYPES`) with a seeded
  `Stage::PlayerBiomeAssignment` RNG and hands one distinct variant to each `PlayerSlot`, then
  overwrites every owned tile's `terrain_type` to its owner's variant. This is a forward-looking
  hook for a future civilization system (e.g. Egyptians on `Sand`, Vikings on `Tundra`, Eskimos
  on `Snow`) — for now it just guarantees the number of distinct rendered land colors always
  equals `num_players`, regardless of what the elevation-driven classification in `terrain.rs`
  originally produced.

The hexagonal map *shape* itself (`hexagon_tiles(edge_tiles) -> Vec<TriCoord>`) lives in
[`shared/src/grid/hexagon.rs`](../../../shared/src/grid/hexagon.rs) rather than here — it's pure
and seedless, and the client needs the same shape for its local map preview before real netcode
exists, so it moved to the crate both apps share. `MapType`/`PlayerSlot` similarly live in
`shared/src/game/map/generation.rs` since a future lobby/map-preview UI needs them too.

## Planned files

- `biomes.rs` — actual biome entity spawning, using `TileMap::towns`/`starting_towers` as
  input (placement itself is done, above):
  1. For each `TileMap::towns` entry, claim its 13-tile neighborhood (`BIOME_OFFSETS` from
     `shared/grid/biome_shape`, still unimplemented too).
  2. Leave everything else as neutral wilderness — that is what biome territory grows into on
     level-up.
  3. Spawn per biome: `Biome` entity with `BiomeTower`, `BiomeLevel(1)`, `BiomeCapacity`,
     unowned `BiomeOwner`, and the initial `BiomeArcher` (BA) unit — `BiomeOwner` for
     `TileMap::starting_towers` entries starts assigned to the corresponding player.

## Design rules

- Everything derives from the seed — no `rand::thread_rng()` (`generation/rng.rs`'s per-stage
  `ChaCha8Rng` streams, seeded from `MapGenConfig::seed`).
- Unit tests (see `generation/*.rs` and this folder's `mod.rs`): town spread/placement
  invariants, starting-tower-per-player-slot count, texture partition balance and water
  exclusion, realized water fraction matching each `MapType`'s target range, determinism for a
  given seed, variation across seeds — mirroring `terrain.rs`'s existing
  `same_seed_is_deterministic`/`different_seeds_usually_differ` pattern. Also (this folder's
  `mod.rs`): no land tile is ever isolated (every non-water tile has ≥1 same-type edge-neighbor —
  only water may stand alone), every non-water tile has an owner, and the number of distinct
  rendered land colors always equals `num_players`.
