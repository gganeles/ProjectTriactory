# `shared/src/grid` — triangular grid model

The geometric foundation of the whole game. Everything (movement, vision, attack range, biome
layout, rendering, touch picking) sits on this module. Lives in `shared` because both the
server (authority) and client (prediction, rendering, picking) need identical grid math.

## Coordinate system: lane coordinates

`TriCoord { a: i32, b: i32, c: i32 }` with the invariant `a + b + c ∈ {1, 2}`:

- `sum == 1` → **upward**-pointing triangle
- `sum == 2` → **downward**-pointing triangle

Each of `a`/`b`/`c` indexes a family of parallel lines at 60° to each other. This scheme makes
neighbors, rotation, and distance trivial with no up/down case explosion.

## Files

- `coords.rs` — `TriCoord`, orientation, world-space conversion:
  `center_world(edge_len) -> Vec2` / `corners_world()` and the inverse `from_world(Vec2)`
  (used by client rendering and touch picking).
- `neighbors.rs` — edge neighbors (always exactly 3, opposite orientation):
  up → `(a+1,b,c), (a,b+1,c), (a,b,c+1)`; down → `(a-1,b,c), (a,b-1,c), (a,b,c-1)`.
  Also `distance` and `tiles_within(center, range)`.
  **Distance metric**: `d = |Δa| + |Δb| + |Δc|` — exactly the minimum number of edge
  crossings. Used for Biome Archer attack range and player vision.
- `hexagon.rs` — `hexagon_tiles(edge_tiles) -> Vec<TriCoord>`: the hexagonal map *shape*. Pure,
  seedless (a function of `edge_tiles` alone). Built as a bounding box directly in lane
  coordinates — the same trick standard hex-grid cube coordinates use, adapted for the
  `a + b + c ∈ {1, 2}` constraint. `edge_tiles` is the number of triangles along one of the
  hexagon's 6 sides; the total tile count is `6 * m^2` where `m = (edge_tiles - 1) / 2`.
  Requires `edge_tiles` odd (so the hexagon has a well-defined center vertex) and `>= 3`. Used
  by the server to build the authoritative `TileMap` and by the client's local map preview.

## Planned files

- `biome_shape.rs` — vertex neighbors, plus the 13-tile 2-1-2-1-2-1 biome polygon. It is precisely *the set of
  triangles sharing at least one vertex with the Biome Tower tile*: BT + 3 edge neighbors +
  9 vertex-only neighbors. Precomputed as `BIOME_OFFSETS: [(i32,i32,i32); 13]` relative to an
  upward BT. **Convention: all Biome Tower tiles are upward triangles** (avoids a mirrored
  offset table). Also the biome anchor super-lattice used by map generation.
- `path.rs` — A* over tiles honoring `Terrain` and the player's unlocked skills (water and
  mountain tiles are impassable without the matching skill). Used by the client for
  tap-to-move and by the server to validate movement.

## Notes

- 13-tile biomes (7 up + 6 down) **cannot tile the plane** — neutral wilderness fills the gaps
  between biomes, which is exactly what biome territory grows into on level-up.
- Unit tests here are the project's foundation: neighbor symmetry, distance == BFS edge count
  (property test), `BIOME_OFFSETS` has 13 tiles / 7-up-6-down / equals the computed
  vertex-neighborhood, `from_world(center_world(t)) == t`.
