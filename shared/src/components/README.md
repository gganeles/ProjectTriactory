# `shared/src/components` — component definitions

Plain data structs for every networked or simulated component. Definitions live here (shared
by both apps); their replication/prediction modes are registered separately in
[../protocol/](../protocol/).

## Files

- `tile.rs` — `Terrain` enum `{ Field, Mountain, Water, Empty }`, the gameplay-authoritative
  environment used for movement/skill gating and placement/boost lookups; plus `TileData`, which
  bundles it with `terrain_type::TerrainType` for the map's per-tile record. Tiles are **not**
  entities — `TileData` is used inside the server's `TileMap` resource and the client's
  `RevealedTiles` resource. (`TilePos` still planned if a placement wrapper type turns out to be
  needed beyond `TriCoord` itself.)
- `terrain_type.rs` — `TerrainType`, a *cosmetic* biome classification (grassland, taiga, ocean,
  snow, ...) derived from elevation + moisture per the table in Amit Patel's polygon map
  generation article, plus `color()` for rendering. Distinct on purpose from the `Biome`
  (territory/tower-ownership) concept below — see the module docs for the naming rationale.
  Generated alongside `Terrain` in `server/src/game/map/terrain.rs`, never mutates gameplay
  logic.

## Planned files

- `player.rs` — `Hero` (marker), `TribeId`, `TribeLeader`, `HeroTile` (current `TriCoord`),
  `HeroKinematics` (progress along the current edge crossing — the predicted movement state),
  `PlayerBank` (money), `PlayerResources` (normal + advanced stockpiles).
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
