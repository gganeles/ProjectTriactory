# `shared/src` — module layout

| Module | Purpose |
|---|---|
| `lib.rs` | Public module tree + `SharedPlugin`: registers `AppState`/`GameMode` now, will grow to configure the 30 Hz fixed timestep, add the `ProtocolPlugin`, and register shared simulation systems. Added by **both** the server and client apps. |
| `states.rs` | `AppState { MainMenu, Game }` and `GameMode { BuildMode, PlayMode }` (a `SubStates` of `AppState::Game` — only exists while in `Game`). Shared so both apps stay in lockstep; the server's finer-grained match lifecycle (`match_state.rs`) is a separate, later concern. |
| `config.rs` | All tuning constants (tick rate, occupation seconds, BA/vision ranges, multipliers, port, protocol id, `DEFAULT_EDGE_TILES`). Single place to adjust the `(?)` values from the mechanics doc. |
| [grid/](grid/) | Triangle-grid coordinate system, neighbors, distance metric, 13-tile biome shape, A* pathfinding. |
| [protocol/](protocol/) | Lightyear registration: components (with prediction/interpolation modes), messages, channels, inputs. The client/server wire contract. |
| [components/](components/) | Plain component structs for players, tiles, biomes, combat, production, connections, dev tree, vision. |
| [data/](data/) | Static game-design data: resource definitions, ARP recipes, development-tree DAG, biome level tables. |
| [systems/](systems/) | Deterministic simulation systems that run on both server and client (rollback), plus the `SimSet` schedule sets. |

Dependency direction inside the crate: `systems` → `components`/`grid`/`data`/`config`;
`protocol` → `components`/`grid`. `grid` and `data` depend on nothing but `config`.
