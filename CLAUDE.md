# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Project Triactory — a multiplayer territory-control game on a triangular-tile grid, built in
**Rust** (edition 2024) with **Bevy 0.19** and **Lightyear 0.28** (netcode/replication/prediction).
Game design lives in [Game Mechanics.md](Game%20Mechanics.md); architecture/testing strategy is
documented per-folder in each directory's `README.md` — read the relevant one before working in a
folder, they are kept current and detailed.

Locked decisions: mobile-first client (developed desktop-first), match-based world (one server
process per match, all state in memory, no database), fully server-authoritative simulation with
client-side prediction of only the local hero, fixed 30 Hz simulation shared by both apps.

## Commands

Cargo workspace with three members: `shared` (crate `triactory_shared`), `server`, `client`.

```sh
cargo build                          # build everything
cargo build -p server                # build one member
cargo test                           # run all unit tests (workspace)
cargo test -p triactory_shared       # run tests for one member
cargo test grid::coords              # run tests matching a path/name
cargo run -p server                  # run the headless server (binds shared::config::DEV_SERVER_ADDR)
cargo run -p client                  # run the desktop client (connects to the same address)
cargo clippy --workspace             # lint
cargo fmt                            # format
```

To exercise client+server together locally, run `cargo run -p server` in one terminal and
`cargo run -p client` in another.

There is no CI config and no separate test harness — tests are plain `#[cfg(test)]` modules
colocated with the code they test (concentrated in `shared`, plus `server/src/game/map`'s
generation code). `shared/README.md` and `shared/src/grid/README.md` describe what's covered
(grid property tests, terrain classification boundaries, etc.) and what's still missing.

## Crate architecture

Dependency direction: `client` and `server` both depend on `shared` (`triactory_shared`); they
never depend on each other.

- **`shared/`** — the Lightyear *protocol crate*: everything client and server must agree on
  byte-for-byte. Grid math (`grid/`), Lightyear registration (`protocol/`, one `ProtocolPlugin`
  added by both apps in identical order), and the game domain (`game/`: `map/` — terrain, biome
  territory, combat, production; `player/` — economy, tech, vision, input; `entities/` —
  projectiles). Each leaf module owns its component definitions and static/tunable data
  (`data.rs`) together, registered via that domain's `Plugin`. **No rendering, no netcode
  transport, no I/O** — must compile for headless server, desktop client, and mobile alike.
  Systems here must be deterministic (pure functions of inputs + component state, no wall-clock
  time, no local randomness) because they run on both the server and the client during rollback.
- **`server/`** — headless authoritative app (`MinimalPlugins` + `LogPlugin` + Lightyear
  `ServerPlugins` + `SharedPlugin`). Holds terrain as a `TileMap` resource
  (`HashMap<TriCoord, TileData>`) — tiles are **never** entities, they ship to clients via
  reliable `TilesRevealed` messages as fog lifts. Every client command (purchases, builds,
  connections, movement steps) is validated here and rejected with `CommandRejected { reason }` —
  never trust the client. No render/audio/winit features.
- **`client/`** — player-facing app (`DefaultPlugins` + `EguiPlugin` + Lightyear `ClientPlugins` +
  `SharedPlugin`). Never authoritative: sends inputs/commands, renders replicated state,
  optimistically predicts only its own hero's movement (rollback on server correction); other
  heroes are interpolated, everything else plainly replicated. Owns `client/assets/` (textures,
  fonts, audio) — the server has none. UI built with `bevy_egui` (immediate-mode, touch-capable),
  not `bevy_lunex` (incompatible Bevy version pin).

### Core design rules (apply project-wide)

1. **Tiles are not entities.** Terrain is static after map generation, held server-side in
   `TileMap`. Only dynamic things (heroes, biomes, biome archers, NRPs/ARPs) are ECS entities and
   get replicated.
2. **Any system that touches a Predicted component lives in `shared/`** and must be
   deterministic — it runs on the server and again on the client during rollback. Everything else
   that mutates game state is server-only.
3. **State flows through replicated components; messages are for the rest** — terrain reveal,
   request/response commands (purchases, builds, connections), transient VFX events.
4. **Fog of war is enforced server-side** via interest management: a client is only sent entities
   inside its tribe's current vision or on tribe-owned biome tiles.

### Plugin/app wiring order matters

Both `main.rs` files add Lightyear's plugins (`ServerPlugins` / `ClientPlugins`) *before*
`SharedPlugin` (which registers the Lightyear protocol via `protocol::ProtocolPlugin`) — Lightyear
must own protocol registration timing. The server additionally must add `StatesPlugin` before
`ServerPlugins`, since `MinimalPlugins` doesn't include one and Lightyear's server backend expects
the `StateTransition` schedule to already exist; the client doesn't need this because
`DefaultPlugins` already includes `StatesPlugin`.

### Grid system

The triangular grid (`shared/src/grid/`) underlies movement, vision, attack range, biome layout,
rendering, and touch picking. Coordinates are `TriCoord { a, b, c }` with invariant
`a + b + c ∈ {1, 2}` (1 = upward triangle, 2 = downward) — this avoids up/down case explosion for
neighbors, rotation, and distance. Distance metric is `d = |Δa| + |Δb| + |Δc|`, exactly the
minimum edge-crossing count. See `shared/src/grid/README.md` for the full model, including the
planned 13-tile biome shape and A* pathfinding.

### Simulation timing

Fixed-timestep simulation at 30 Hz (`shared::config::TICK_RATE_HZ`) shared by both apps;
everything visual (rendering, UI, camera) runs at frame rate instead — in the client, simulation-
facing systems (e.g. input emission) belong in `FixedUpdate`, everything else in `Update`/
`PostUpdate`.

## Current state / known gaps

This is a very early-stage project (see the milestone roadmap in [README.md](README.md): M0
workspace → M10 mobile builds — currently early in this sequence). Notably:
- No real player identity or auth: the client hardcodes client id `0`; server netcode validates
  connect tokens against the all-zero default key. Fine for local dev, not for deployment.
- No fog of war yet — the server currently sends the whole map once on connect.
- No entity replication yet (no hero exists as an entity yet); prediction/interpolation targets
  are unverified beyond Lightyear's own examples.
- Several planned files are documented but not yet implemented (noted as "Planned files" in the
  relevant folder's `README.md`) — check the folder's README before assuming a described system
  exists.
