# `server/src` — headless authoritative app

The single source of truth. Runs `MinimalPlugins` + `ScheduleRunnerPlugin` (no rendering) +
`SharedPlugin` from `triactory_shared`. **One match per process**: state lives in memory and
dies with the process; a future matchmaker just spawns processes.

## Planned top-level files

- `main.rs` — app wiring: minimal Bevy app, shared plugin, Lightyear server plugins, match
  states, all server systems.
- `netcode.rs` — Lightyear server setup: UDP transport + netcode.io auth, start listening on
  `config::SERVER_PORT`. Connect tokens are locally generated (shared private key) until an
  auth service exists.
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
- `replication.rs` — attaches replication components on spawn: `Replicate`, prediction
  targeted at the owning client + interpolation for everyone else (heroes), and per-component
  owner-only targets (`PlayerBank`, `PlayerResources`, `DevTree`).
- `visibility.rs` — interest management / fog of war: each tick compute every client's visible
  tile set (vision around all tribe heroes + all tribe-owned biome tiles) and toggle
  per-entity, per-client visibility. Start with direct visibility sets — O(entities × players)
  is fine for one match per process; Lightyear Rooms are a later optimization.

## Subfolders

- [mapgen/](mapgen/) — seeded map generation at match start.
- [systems/](systems/) — all authoritative gameplay systems.

## Design rules

- The server holds terrain as a `TileMap` resource (`HashMap<TriCoord, TileData>`), not as
  entities. Tiles are never replicated as entities; terrain ships via `TilesRevealed`.
- Every client command (`PurchaseDevNode`, `BuildProduction`, `CreateConnection`,
  `UpgradeBiome`, movement steps) is validated here. Reject with `CommandRejected { reason }` —
  never trust the client.
- No Bevy render/audio/winit features in this crate.
