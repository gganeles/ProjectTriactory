# `shared` — crate `triactory_shared`

The Lightyear **protocol crate**: everything the client and server must agree on byte-for-byte.
Both apps depend on it; neither may duplicate anything defined here.

Reorganized (see `restructure` in git history) from an earlier flat `components/` / `data/` /
`systems/` layout into domain folders under `game/` — each domain owns its own component
definitions, static data, and (eventually) systems together, instead of splitting them across
three parallel trees that all had to be kept in sync by hand.

## What lives here

- **Grid math** ([src/grid/](src/grid/)) — triangle coordinates, neighbors, distance, biome
  shape, pathfinding.
- **Protocol registration** ([src/protocol/](src/protocol/)) — one `ProtocolPlugin` that
  registers every component, message, channel, and input on both sides in identical order.
- **Game domain** ([src/game/](src/game/)) — `map/` (terrain, biome territory, combat,
  production), `player/` (economy, tech, vision, input), `entities/` (projectiles). Each leaf
  module owns its own component definitions (`mod.rs`) and static/tunable data (`data.rs`)
  together, registered via that domain's `Plugin`.

Top-level files:

- `src/lib.rs` — public modules + `SharedPlugin` (state registration, protocol, `game::GamePlugin`).
- `src/config.rs` — tuning constants: `TICK_RATE_HZ`, `DEFAULT_EDGE_TILES`, `PROTOCOL_ID`,
  `SERVER_PORT`, `DEV_SERVER_ADDR`. Occupation seconds, BA/vision ranges, and multipliers from
  the mechanics doc are still `(?)` placeholders, not yet added.
- `src/states.rs` — `AppState { MainMenu, Game }` and `GameMode { BuildMode, PlayMode }`.

## Design rules

- Systems here must be **deterministic**: pure functions of inputs + component state, no
  wall-clock time, no local randomness. Rollback reconciliation depends on it.
- If a system touches a Predicted component, it belongs here. If not, it belongs in `server/`.
- This crate has **no rendering, no netcode transport, no I/O** — it must compile for the
  headless server, the desktop client, and mobile targets alike.
- Components are dumb data: no methods with game logic beyond trivial accessors/invariants.
- Anything replicated must stay `serde`-serializable and cheap to diff.

## Testing

This crate carries the bulk of the pure unit tests: grid properties (neighbor symmetry,
distance == BFS edge count, biome shape invariants, world↔coord round-trips), terrain
classification boundaries, recipe math, dev-tree DAG acyclicity, level-table monotonicity,
capacity clamping.
