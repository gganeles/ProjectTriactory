# `client/src/rendering` — map and entity rendering

Everything drawn to the screen except UI widgets. 2D orthographic, budgeted for mobile GPUs.

## Files

- `camera.rs` — spawns the app's one persistent camera at `Startup` (`bevy_egui` binds its UI
  context to the first camera the app creates, so this must exist before any scene needs one).
  Scenes reposition it rather than spawning their own — a second camera would need explicit
  `order`/clear-color setup to layer correctly with the egui UI pass.
- `hex_map.rs` — **temporary prototype**, not the planned `grid_mesh.rs` system below: on
  `AppState::Game` entry, reads the replicated `world_model::RevealedTiles` and spawns one
  `Triangle3d` mesh entity per tile, colored by `TerrainType::color()` (one `StandardMaterial`
  per distinct biome present, cached in a `HashMap` so repeats reuse the same handle), then
  reframes the persistent camera as an angled orthographic "2.5D" view sized to fit. Also inserts
  the `MapBounds` resource (the map's radius from the origin), which `input::pan` and
  `input::zoom` read to keep the camera from drifting/zooming arbitrarily far from the board;
  removed again on `AppState::Game` exit. Will be replaced by the real, chunked-mesh pipeline
  below (the per-biome coloring here is the interim substitute for the `terrain.rs` texture atlas
  planned there).

## Planned files

- `grid_mesh.rs` — the triangle grid as **chunked meshes** (~16×16-triangle chunks). Only
  dirty chunks rebuild — a chunk is dirtied when tiles in it are revealed, change ownership,
  or change fog state. Never rebuild the whole map per frame.
- `terrain.rs` — single texture atlas for terrain types; **per-owner biome texture
  adaptation** (mechanics §1.10: a biome's look adapts to the occupying player) — swaps the
  biome's tile textures/tint when `BiomeOwner` changes.
- `fog.rs` — the three fog states, derived from `world_model::RevealedTiles` + currently
  replicated entities:
  1. **Unexplored** — tile never revealed → black.
  2. **Explored** — terrain known, no live vision → dimmed terrain, no entities.
  3. **Visible** — inside current tribe vision → full terrain + live entities.
  The server enforces this via interest management; fog rendering is presentation only.
- `entities.rs` — sprites for heroes, Biome Archers, NRPs/ARPs and the Biome Tower; line
  rendering for `ResourceLink` connections (and the rubber-band preview during a drag).

Pan (drag on empty space) and zoom (pinch/scroll), clamped to the map bounds, are already
implemented — see `input/pan.rs` and `input/zoom.rs` — rather than living in this folder's
`camera.rs`, which only spawns/positions the camera.

## Design rules

- Rendering reads replicated + local state; it never mutates simulation state.
- All systems here run at frame rate (`Update`/`PostUpdate`), never in `FixedUpdate`.
- Mobile budget: one atlas, chunked dirty-only rebuilds, MSAA off, target 60 fps.
