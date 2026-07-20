# `shared` — crate `triactory_shared`

The Lightyear **protocol crate**: everything the client and server must agree on byte-for-byte.
Both apps depend on it; neither may duplicate anything defined here.

## What lives here

- **Grid math** ([src/grid/](src/grid/)) — triangle coordinates, neighbors, distance, biome
  shape, pathfinding.
- **Protocol registration** ([src/protocol/](src/protocol/)) — one `ProtocolPlugin` that
  registers every component, message, channel, and input on both sides in identical order.
- **Component definitions** ([src/components/](src/components/)) — the plain data structs.
- **Static game data** ([src/data/](src/data/)) — resource kinds, ARP recipes, the development
  tree DAG, biome level tables.
- **Shared simulation systems** ([src/systems/](src/systems/)) — the deterministic systems
  that run on the server *and* on the client during prediction rollback.

Planned top-level files:

- `src/lib.rs` — public modules + `SharedPlugin` (fixed timestep config, protocol, shared systems).
- `src/config.rs` — tuning constants: `TICK_RATE = 30 Hz`, `OCCUPY_SECS = 3.0`, `BA_RANGE = 2`,
  `VISION_RANGE = 2`, `LEADER_DEFENSE_MULT = 1.5`, `TERRAIN_BOOST = 1.5`, `SERVER_PORT`,
  `PROTOCOL_ID`. Every `(?)` value from the mechanics doc lives here so it can be tuned
  without touching logic.

## Design rules

- Systems here must be **deterministic**: pure functions of inputs + component state, no
  wall-clock time, no local randomness. Rollback reconciliation depends on it.
- If a system touches a Predicted component, it belongs here. If not, it belongs in `server/`.
- This crate has **no rendering, no netcode transport, no I/O** — it must compile for the
  headless server, the desktop client, and mobile targets alike.

## Testing

This crate carries the bulk of the pure unit tests: grid properties (neighbor symmetry,
distance == BFS edge count, biome shape invariants, world↔coord round-trips), recipe math,
dev-tree DAG acyclicity, level-table monotonicity, capacity clamping.
