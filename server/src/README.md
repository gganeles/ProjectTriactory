# `server/src` — headless authoritative app

The single source of truth. Runs `MinimalPlugins` + `bevy::log::LogPlugin` (no rendering; log
plugin is added explicitly since `MinimalPlugins` doesn't include one) + Lightyear's
`ServerPlugins` + `SharedPlugin` from `triactory_shared`. **One match per process**: state lives
in memory and dies with the process; a future matchmaker just spawns processes.

## Files

- `main.rs` — app wiring: `MinimalPlugins`, `LogPlugin`, `StatesPlugin` (has to come *before*
  `ServerPlugins` — Lightyear's server backend registers its own internal `States` type and
  expects the `StateTransition` schedule to already exist), `ServerPlugins`, then `SharedPlugin`
  (which registers the Lightyear protocol — see `shared/src/protocol/mod.rs` for why that must
  come after `ServerPlugins`). Since the server is headless with no menu, a `Startup` system
  moves `AppState` straight to `Game` (unlike the client, which waits for the player to tap
  Start).
- `netcode.rs` — Lightyear server setup: spawns the `NetcodeServer` + `LocalAddr` +
  `ServerUdpIo` entity and triggers `Start`, listening on
  `triactory_shared::config::DEV_SERVER_ADDR`. Connect tokens are validated against
  `Key::default()` (all-zero bytes) since there's no real auth service yet — fine for local
  development, not for a real deployment.
- `replication.rs` — currently just sends the whole map: on `Connected`, looks up the new
  client's `RemoteId` and sends the server's `TileMap` as one `TilesRevealed` message over
  `MapChannel`. Per-entity replication (`Replicate`, prediction/interpolation targets,
  owner-only components) described below is a later milestone, once there's an entity worth
  replicating (the hero).

## Planned top-level files

- `match_state.rs` — Bevy `States` for the match lifecycle plus the player registry
  (client id ↔ `TribeId`):
  - **Lobby** — accept connections, assign tribes, replicate roster, wait for `SetReady`.
  - **Starting** — seeded mapgen; per player: assign starting biome (BTF), spawn hero on its
    BT, grant starting money + 1 resource + nearest skill; send initial `TilesRevealed`;
    countdown.
  - **Playing** — full simulation. On disconnect the hero stays input-less for a reconnect
    window, then despawns; biomes stay tribe-owned.
  - **Ended** — victory condition met (initial rule: last tribe holding a biome), broadcast
    results, grace period, process exits.
- `visibility.rs` — interest management / fog of war: each tick compute every client's visible
  tile set (vision around all tribe heroes + all tribe-owned biome tiles) and toggle
  per-entity, per-client visibility. Start with direct visibility sets — O(entities × players)
  is fine for one match per process; Lightyear Rooms are a later optimization.

## Subfolders

- [map/](map/) — seeded map generation at match start.
- [systems/](systems/) — all authoritative gameplay systems.

## Design rules

- The server holds terrain as a `TileMap` resource (`HashMap<TriCoord, TileData>`), not as
  entities. Tiles are never replicated as entities; terrain ships via `TilesRevealed`.
- Every client command (`PurchaseDevNode`, `BuildProduction`, `CreateConnection`,
  `UpgradeBiome`, movement steps) is validated here. Reject with `CommandRejected { reason }` —
  never trust the client.
- No Bevy render/audio/winit features in this crate.
