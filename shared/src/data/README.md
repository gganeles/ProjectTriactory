# `shared/src/data` — static game-design data

The design-tunable content of the game, as plain Rust constants/tables. Both apps need it: the
server to validate and simulate, the client to render UI (dev-tree screen, costs, recipe
tooltips) without asking the server.

## Planned files

- `resources_def.rs` — the normal and advanced resource kinds, and the **terrain affinity
  table**: which resource gets the 1.5x production boost on which terrain (e.g. rock on
  mountain).
- `recipes.rs` — ARP recipes: `x` of NR1 + `y` of NR2 → 1 AR. Consumed by the advanced
  production system; displayed by the client UI.
- `dev_tree_def.rs` — the static Development Tree DAG: node id, kind
  (`Skill | Tech | Build | ResourceProduction | Logistics`), money cost, prerequisite edges.
  Per-player unlock *state* is the `DevTree` component in [../components/](../components/).
- `biome_levels.rs` — per-level tables: resource capacity (normal + advanced), territory
  radius (lane-distance from the BT that the biome claims on level-up), coin production rate,
  and upgrade cost (resources + money).

## Design rules

- Data here is **static and identical on both sides** — never mutated at runtime. Balance
  changes are edits to these tables, not to systems.
- Keep it in code (not asset files) until the tables stabilize; unit tests assert structural
  invariants: dev-tree DAG has no cycles, level tables are monotonic, every resource has a
  defined affinity.
