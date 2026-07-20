# `server/src/systems` — authoritative gameplay systems

Every system that mutates game state but does **not** touch a Predicted component. These run
only on the server, in `FixedUpdate` (30 Hz) inside `SimSet::Gameplay` unless noted. They are
the refined versions of the systems sketched in the mechanics doc's ECS section.

## Planned files

- `occupation.rs` — the 3-second capture rule: for each Biome Tower, count distinct tribes
  standing on it. Exactly one → advance `OccupationTimer`; two or more → freeze (contested);
  zero → reset. At `OCCUPY_SECS`, transfer `BiomeOwner`, reset the timer, emit
  `BiomeCaptured` (triggers client-side owner retexturing).
- `combat.rs` — Biome Archer auto-attack: target the nearest enemy hero within `BA_RANGE`
  (lane distance) of the biome's border tiles; apply damage on cooldown; set `UnderAttack`
  on the biome while defense is engaged (halts production); apply `LEADER_DEFENSE_MULT`
  (1.5x) while the tribe leader's hero stands on any tile of the biome; clear `UnderAttack`
  when no enemies remain in range.
- `production.rs` — per NRP with a valid `ResourceLink` chain to its BT: skip if
  `ProductionHalted` / biome `UnderAttack`; output = `base_rate × terrain_boost` (1.5x when
  the NRP sits on its affinity terrain); clamp against `BiomeCapacity`; deposit into the
  tribe's `PlayerResources`.
- `advanced_production.rs` — per ARP: verify NRP→ARP and ARP→BT links exist, consume recipe
  inputs (x of NR1 + y of NR2) from the stockpile, emit the advanced resource. Throughput
  naturally scales when the feeding NRPs are terrain-boosted.
- `biome_mgmt.rs` — handles `UpgradeBiome`: validate resource + money cost, bump `BiomeLevel`,
  grow territory to the level's radius (`tiles_within`, skipping already-owned tiles), raise
  capacity and coin rate from `shared/data/biome_levels`.
- `economy.rs` — coin income per owned biome per tick into `PlayerBank`; shared purchase
  validation helpers (sufficient funds, etc.).
- `devtree.rs` — handles `PurchaseDevNode` / `BuildProduction` / `CreateConnection`: check
  prerequisites against `shared/data/dev_tree_def`, costs, the 1-NRP + 1-ARP per biome caps,
  NRP placement immutability (once placed, never moved), and link validity. Reject invalid
  commands with `CommandRejected`.
- `vision.rs` — (`SimSet::PostSim`) per-player permanent reveal set: diff newly-in-range tiles
  (`d ≤ VISION_RANGE`) on hero movement into batched `TilesRevealed` messages. Explored
  terrain never un-reveals. Live entity visibility is handled by `../visibility.rs`.

## Design rules

- All command handling is validate-then-apply; the client's optimistic UI is never trusted.
- These systems may use server-only resources (`TileMap`, player registry) freely — they never
  run on the client, so determinism constraints are looser than in `shared/src/systems`,
  but keep them seed-deterministic where cheap (replay/testing value).
- Headless sim tests target this folder: bare `App` + `SharedPlugin` + these systems, scripted
  inputs, asserting occupation timing, contested freeze, production halt under attack, 1.5x
  leader defense.
