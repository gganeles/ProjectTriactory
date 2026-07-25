# `shared/src/components` — component definitions

Plain data structs for every networked or simulated component. Definitions live here (shared
by both apps); their replication/prediction modes are registered separately in
[../protocol/](../protocol/).

## Planned files

- `player.rs` — `Hero` (marker), `TribeId`, `TribeLeader`, `HeroTile` (current `TriCoord`),
  `HeroKinematics` (progress along the current edge crossing — the predicted movement state),
  `PlayerBank` (money), `PlayerResources` (normal + advanced stockpiles).
- `tile.rs` — `Terrain` enum `{ Field, Mountain, Water, Empty }` (done; `TilePos` still planned
  if a placement wrapper type turns out to be needed beyond `TriCoord` itself). Note: tiles are
  **not** entities — `Terrain` is used inside the server's `TileMap` resource and the client's
  `RevealedTiles` resource, plus placement/boost lookups.
- `biome.rs` — `Biome` (marker + claimed tile set), `BiomeTower` (the BT tile),
  `BiomeOwner (Option<TribeId>)`, `BiomeLevel`, `BiomeCapacity` (normal/advanced resource +
  building caps), `OccupationTimer` (3-second capture progress; frozen when contested),
  `UnderAttack` flag (halts production, drives defense buff).
- `combat.rs` — `BiomeArcher { range, damage, cooldown }` (the BA unit every biome starts
  with), `Health` (heroes and archers).
- `production.rs` — `Nrp` / `Arp` markers, `ProductionRate` (base rate + terrain multiplier),
  `ProductionHalted`, `Recipe` (x of NR1 + y of NR2 → AR), `Stockpile`.
- `connection.rs` — `ResourceLink { from: Entity, to: Entity }` for the dragged logistics
  links: NRP→BT, NRP→ARP, ARP→BT. Entity-mapped over the network.
- `devtree.rs` — `DevTree` (bitset of unlocked nodes), `DevNodeId`. Static node definitions
  live in [../data/](../data/); this is only the per-player unlock state.
- `vision.rs` — `VisionSource { range }` (heroes reveal `VISION_RANGE` around themselves).

## Design rules

- Components are dumb data: no methods with game logic beyond trivial accessors/invariants.
  Logic lives in `shared/src/systems` (deterministic) or `server/src/systems` (authoritative).
- Anything here that is replicated must stay `serde`-serializable and cheap to diff.
