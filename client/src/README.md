# `client/src` — player-facing app

Rendering, input, UI, and prediction. Runs `DefaultPlugins` + `SharedPlugin` from
`triactory_shared` + Lightyear client plugins. **Mobile-first** (iOS/Android), but developed
desktop-first with mouse emulating touch; mobile packaging is milestone M10.

## Planned top-level files

- `main.rs` — app wiring: default plugins, shared plugin, netcode client, rendering/input/UI
  plugins.
- `app_state.rs` — the client's **own** top-level screen state machine, a Bevy `States` enum
  distinct from the server's authoritative `match_state.rs`:
  `MainMenu → Connecting → Lobby → Playing → Ended`. This exists because the client has
  screens the server doesn't know about (`MainMenu`, `Connecting`) and needs to render UI
  correctly before a connection exists at all. `ui/lobby.rs` and the rest of `ui/` read this
  state to decide what to show; it transitions in response to netcode connection events and
  replicated `MatchPhase` messages once connected.
- `netcode.rs` — Lightyear client: connect over UDP with a netcode token, tick sync with the
  server's 30 Hz fixed timestep. Mobile OSes suspend sockets on background → app-resume is
  treated as a reconnect (token re-request path exists from day one).
- `prediction.rs` — handling of Predicted/Interpolated entity spawns: the player's own hero is
  **Predicted** (client runs `shared::systems::movement` locally and rolls back on server
  correction); all other heroes are **Interpolated**; everything else is plainly replicated.
  Also frame interpolation for visually smoothing the predicted hero between fixed ticks.
- `world_model.rs` — the client's memory of the map: a `RevealedTiles` resource
  (`HashMap<TriCoord, Terrain>`) fed by `TilesRevealed` messages. Terrain memory survives fog
  (explored-but-not-visible tiles stay known); this is also what local A* pathfinding runs on.

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
