# `shared/src` — module layout

| Module | Purpose |
|---|---|
| `lib.rs` | Public module tree + `SharedPlugin`: configures the 30 Hz fixed timestep, adds the `ProtocolPlugin`, and registers shared simulation systems. Added by **both** the server and client apps. |
| `config.rs` | All tuning constants (tick rate, occupation seconds, BA/vision ranges, multipliers, port, protocol id). Single place to adjust the `(?)` values from the mechanics doc. |
| [grid/](grid/) | Triangle-grid coordinate system, neighbors, distance metric, 13-tile biome shape, A* pathfinding. |
| [protocol/](protocol/) | Lightyear registration: components (with prediction/interpolation modes), messages, channels, inputs. The client/server wire contract. |
| [components/](components/) | Plain component structs for players, tiles, biomes, combat, production, connections, dev tree, vision. |
| [data/](data/) | Static game-design data: resource definitions, ARP recipes, development-tree DAG, biome level tables. |
| [systems/](systems/) | Deterministic simulation systems that run on both server and client (rollback), plus the `SimSet` schedule sets. |

Dependency direction inside the crate: `systems` → `components`/`grid`/`data`/`config`;
`protocol` → `components`/`grid`. `grid` and `data` depend on nothing but `config`.
