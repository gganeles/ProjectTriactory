# `client/src/rendering` — map and entity rendering

Everything drawn to the screen except UI widgets. 2D orthographic, budgeted for mobile GPUs.

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
- `camera.rs` — 2D orthographic camera with pan (drag on empty space) and zoom (pinch /
  scroll), clamped to the map bounds.

## Design rules

- Rendering reads replicated + local state; it never mutates simulation state.
- All systems here run at frame rate (`Update`/`PostUpdate`), never in `FixedUpdate`.
- Mobile budget: one atlas, chunked dirty-only rebuilds, MSAA off, target 60 fps.
