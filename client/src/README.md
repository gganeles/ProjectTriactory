# `client/src` — player-facing app

Rendering, input, UI, and prediction. Runs `DefaultPlugins` + Lightyear's `ClientPlugins` +
`SharedPlugin` from `triactory_shared`. **Mobile-first** (iOS/Android), but developed
desktop-first with mouse emulating touch; mobile packaging is milestone M10.

## Files

- `main.rs` — app wiring: `DefaultPlugins`, `EguiPlugin`, `ClientPlugins`, then `SharedPlugin`
  (which registers the Lightyear protocol — see `shared/src/protocol/mod.rs` for why that must
  come after `ClientPlugins`), then the netcode-connect `Startup` system and the
  rendering/input/UI/world-model plugins. `DefaultPlugins` already includes `StatesPlugin`
  (unlike the server's `MinimalPlugins`), so no extra ordering care is needed there.
- `netcode.rs` — connects to `triactory_shared::config::DEV_SERVER_ADDR` once at `Startup` with
  a locally-generated netcode token (`Authentication::Manual`, matching the server's
  `Key::default()`). No real player identity yet (client id is hardcoded to `0`), no tick-sync
  tuning beyond Lightyear's defaults, and no reconnect-on-resume handling — all later work
  (milestones M9/M10).
- `world_model.rs` — the client's memory of the map: a `RevealedTiles` resource
  (`HashMap<TriCoord, TileData>`) fed by `TilesRevealed` messages. There's no fog yet, so this
  just accumulates whatever the server has sent (currently the whole map, once); this is where
  fog-surviving terrain memory and local A* pathfinding will read from once those exist.

## Planned top-level files

- `app_state.rs` — the client's **own** top-level screen state machine, a Bevy `States` enum
  distinct from the server's authoritative `match_state.rs`:
  `MainMenu → Connecting → Lobby → Playing → Ended`. Not implemented yet — the client currently
  reuses the *shared* `AppState { MainMenu, Game }` from `shared/src/states.rs` directly, so
  there's no client-only `Connecting`/`Lobby`/`Ended` screen yet, and `AppState::Game` doesn't
  actually wait for the netcode connection to succeed before showing the map.
- `prediction.rs` — handling of Predicted/Interpolated entity spawns: the player's own hero is
  **Predicted** (client runs `shared::systems::movement` locally and rolls back on server
  correction); all other heroes are **Interpolated**; everything else is plainly replicated.
  Also frame interpolation for visually smoothing the predicted hero between fixed ticks.

## Subfolders

- [rendering/](rendering/) — grid mesh, terrain/fog, entity sprites, camera.
- [input/](input/) — touch/mouse gesture classification, tap-to-move, connection dragging.
- [ui/](ui/) — HUD, development tree screen, biome panel, lobby.

## Design rules

- The client is **never authoritative**: it sends inputs and commands, renders replicated
  state, and optimistically predicts only its own hero's movement. All command results
  (purchases, builds, captures) arrive as replicated component changes or `CommandRejected`.
- Simulation-facing logic (input emission) runs in `FixedUpdate`; everything visual runs at
  frame rate.
- Rendering budget targets mobile: 2D orthographic, chunked meshes, one texture atlas,
  MSAA off, 60 fps render over the 30 Hz sim.
