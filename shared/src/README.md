# `shared/src` — module layout

| Module | Purpose |
|---|---|
| `lib.rs` | Public module tree + `SharedPlugin`: registers `AppState`/`GameMode`, `protocol::ProtocolPlugin`, and `game::GamePlugin`. Must be added after `ClientPlugins`/`ServerPlugins` in each app's `main.rs` (see `protocol/mod.rs`). Added by **both** the server and client apps. |
| `states.rs` | `AppState { MainMenu, Game }` and `GameMode { BuildMode, PlayMode }` (a `SubStates` of `AppState::Game` — only exists while in `Game`). Shared so both apps stay in lockstep; the server's finer-grained match lifecycle (`match_state.rs`) is a separate, later concern. |
| `config.rs` | Tuning constants: `TICK_RATE_HZ`, `PROTOCOL_ID`, `SERVER_PORT`, `DEV_SERVER_ADDR` (all implemented, though `DEV_SERVER_ADDR` is a hardcoded loopback address good for local dev only), `DEFAULT_EDGE_TILES`. Occupation seconds, BA/vision ranges, and multipliers from the mechanics doc are still `(?)` placeholders, not yet added. |
| [grid/](grid/) | Triangle-grid coordinate system, neighbors, distance metric, 13-tile biome shape, A* pathfinding. |
| [protocol/](protocol/) | Lightyear registration: currently just the map message/channel (plus a DEBUG client→server regenerate-map request); other messages/channels, replicated components, and inputs are still planned. The client/server wire contract. |
| [game/](game/) | The game domain, split by area rather than by data-shape: `map/` (`terrain` — `Terrain`/`TerrainType`/`TileData`, generated per-tile; `biome` — territory/tower ownership, **not** the same concept as `TerrainType`, see `map/terrain.rs`'s docs; `combat`; `production`), `player/` (`economy`, `tech`, `vision`, `input`), `entities/` (`projectiles`). Each leaf owns a `mod.rs` (components + its `Plugin`) and a `data.rs` (static/tunable tables) together. |

Dependency direction inside the crate: `game` → `grid`/`config`; `protocol` → `game`/`grid`.
`grid` depends on nothing but `config`.
