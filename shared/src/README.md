# `shared/src` — module layout

| Module | Purpose |
|---|---|
| `lib.rs` | Public module tree + `SharedPlugin`: registers `AppState`/`GameMode` and adds `protocol::ProtocolPlugin`. Must be added after `ClientPlugins`/`ServerPlugins` in each app's `main.rs` (see `protocol/mod.rs`). Will grow to register shared simulation systems. Added by **both** the server and client apps. |
| `states.rs` | `AppState { MainMenu, Game }` and `GameMode { BuildMode, PlayMode }` (a `SubStates` of `AppState::Game` — only exists while in `Game`). Shared so both apps stay in lockstep; the server's finer-grained match lifecycle (`match_state.rs`) is a separate, later concern. |
| `config.rs` | Tuning constants: `TICK_RATE_HZ`, `PROTOCOL_ID`, `SERVER_PORT`, `DEV_SERVER_ADDR` (all implemented, though `DEV_SERVER_ADDR` is a hardcoded loopback address good for local dev only), `DEFAULT_EDGE_TILES`. Occupation seconds, BA/vision ranges, and multipliers from the mechanics doc are still `(?)` placeholders, not yet added. |
| [grid/](grid/) | Triangle-grid coordinate system, neighbors, distance metric, 13-tile biome shape, A* pathfinding. |
| [protocol/](protocol/) | Lightyear registration: currently just the map message/channel; components (with prediction/interpolation modes), other messages/channels, and inputs are still planned. The client/server wire contract. |
| [components/](components/) | Plain component structs for players, tiles, biomes, combat, production, connections, dev tree, vision. |
| [data/](data/) | Static game-design data: resource definitions, ARP recipes, development-tree DAG, biome level tables. |
| [systems/](systems/) | Deterministic simulation systems that run on both server and client (rollback), plus the `SimSet` schedule sets. |

Dependency direction inside the crate: `systems` → `components`/`grid`/`data`/`config`;
`protocol` → `components`/`grid`. `grid` and `data` depend on nothing but `config`.
