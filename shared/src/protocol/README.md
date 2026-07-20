# `shared/src/protocol` — the wire contract

Lightyear registration for everything that crosses the network. One `ProtocolPlugin` added by
**both** apps — sharing the plugin guarantees identical registration order on both sides,
which Lightyear requires.

## Planned files

- `mod.rs` — `ProtocolPlugin`: calls every `register_component` / `add_message` /
  `add_channel` / input registration.
- `components.rs` — replication registration only (the structs themselves live in
  [../components/](../components/)). Encodes the replication table:

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
  `TilesRevealed { Vec<(TriCoord, Terrain)> }`, `CommandRejected { reason }`, `BiomeCaptured`,
  `MatchPhase`, `CombatVfx`.
- `channels.rs` — `CommandChannel` (reliable ordered: commands + results),
  `MapChannel` (reliable unordered: idempotent `TilesRevealed` batches),
  `EventChannel` (sequenced unreliable: cosmetic VFX events).
- `inputs.rs` — **native Lightyear inputs** (not leafwing):
  `HeroInput { step: Option<EdgeDir> }` where `EdgeDir` picks one of the triangle's 3 edges.
  The client turns a tap into an A* path and emits one `HeroInput` per fixed tick; the server
  validates each step (adjacency, skill-gated terrain). Clients never send "teleport to X".

## Design rules

- State the client cares about persistently flows through **replicated components**; messages
  are only for terrain reveal, request/response, and transient events.
- Lightyear 0.28 is a post-rewrite API — verify exact type names (`PredictionTarget`,
  `InterpolationTarget`, entity-mapping and channel-config calls) against the 0.28
  `simple_box` / `interest_management` examples during milestone M2.
