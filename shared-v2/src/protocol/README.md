# `shared/src/protocol` — the wire contract

Lightyear registration for everything that crosses the network. One `ProtocolPlugin` added by
**both** apps via `SharedPlugin` — sharing the plugin guarantees identical registration order on
both sides, which Lightyear requires. Must be added after `ClientPlugins`/`ServerPlugins` but
before any `Client`/`Server` entity is spawned — both apps' `main.rs` enforce that ordering.

## Files

- `mod.rs` — `ProtocolPlugin`, currently registering just the map: `MapChannel` (reliable
  ordered) and the `TilesRevealed(Vec<(TriCoord, Terrain)>)` message, server→client only. There's
  no fog of war yet, so today it's always the whole map sent once right after connecting (see
  `server/src/replication.rs`), not a fog-driven reveal batch — it has the same name as the
  message planned below since it'll grow into that. Everything else planned for this file
  (replicated components, gameplay messages/channels, native inputs) will split out into
  `components.rs` / `messages.rs` / `channels.rs` / `inputs.rs` as those get implemented.

## Planned files

- `components.rs` — replication registration only (the structs themselves live in
  [../game/](../game/)). Encodes the replication table:

  | Component | Replicated to | Client mode |
  |---|---|---|
  | `HeroTile`, `HeroKinematics` | vision-based | **Predicted** (own hero), **Interpolated** (others) |
  | `TribeId`, `Hero`, `TribeLeader` | vision-based | plain |
  | `PlayerBank`, `PlayerResources`, `DevTree` | **owner only** | plain |
  | `Biome`, `BiomeOwner`, `BiomeLevel`, `BiomeCapacity`, `UnderAttack` | vision + owning tribe always | plain |
  | `OccupationTimer` | players near the BT | plain (drives capture-ring UI) |
  | `BiomeArcher`, `Health` | vision-based | plain |
  | `Nrp`, `Arp`, `ProductionRate`, `ProductionHalted`, `Stockpile`, `ResourceLink` | vision + owning tribe always | plain |

  `ResourceLink` holds `Entity` references → must be registered with Lightyear's entity
  mapping.
- `messages.rs` — client→server commands: `PurchaseDevNode`, `BuildProduction { kind, tile }`,
  `CreateConnection { from, to }`, `UpgradeBiome`, `SetReady`. Server→client:
  `CommandRejected { reason }`, `BiomeCaptured`, `MatchPhase`, `CombatVfx` (`TilesRevealed` is
  already implemented — see `mod.rs` above).
- `channels.rs` — `CommandChannel` (reliable ordered: commands + results),
  `EventChannel` (sequenced unreliable: cosmetic VFX events) (`MapChannel` is already
  implemented — see `mod.rs` above).
- `inputs.rs` — **native Lightyear inputs** (not leafwing):
  `HeroInput { step: Option<EdgeDir> }` where `EdgeDir` picks one of the triangle's 3 edges.
  The client turns a tap into an A* path and emits one `HeroInput` per fixed tick; the server
  validates each step (adjacency, skill-gated terrain). Clients never send "teleport to X".

## Design rules

- State the client cares about persistently flows through **replicated components**; messages
  are only for terrain reveal, request/response, and transient events.
- Lightyear 0.28 is a post-rewrite, entity-component API (a `Client`/`Server` role is just an
  entity with the right components, e.g. `NetcodeClient` + `Link` + `LocalAddr`/`PeerAddr`, not a
  config struct). Connectivity, message registration (`register_message` + `add_channel`), and
  message send/receive (`ServerMultiMessageSender`, `MessageReceiver<T>`) are verified working
  against the 0.28 `simple_setup` / `simple_box` examples. Replicated-component prediction/
  interpolation targets (`PredictionTarget`, `InterpolationTarget`, entity mapping) are still
  unverified — check those same examples again when the first replicated entity (the hero)
  is implemented.
