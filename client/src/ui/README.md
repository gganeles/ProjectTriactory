# `client/src/ui` — screens and HUD

Bevy UI widgets and screens. Reads replicated state; issues command messages; renders results
(success arrives as replicated component changes, failure as `CommandRejected { reason }`
toasts).

## Planned files

- `hud.rs` — persistent overlay: money (`PlayerBank`), resource counts (`PlayerResources`),
  and the **capture-progress ring** drawn over a Biome Tower while `OccupationTimer` advances
  (the replicated timer drives it; freezing shows contested state).
- `dev_tree_ui.rs` — the Development Tree screen: renders the static DAG from
  `shared/data/dev_tree_def` with per-node unlock state from the replicated `DevTree`
  component; tapping an affordable node sends `PurchaseDevNode`. Sections: Skills, Tech,
  Build, Resource Production, Logistics.
- `biome_panel.rs` — inspection panel opened by long-press on a biome: level, capacities,
  production status, owner; the **Upgrade** button sends `UpgradeBiome` when resource + money
  requirements are met (requirements shown either way).
- `lobby.rs` — pre/post-match screens: connect screen, lobby roster with ready-up
  (`SetReady`), match countdown, and the end-of-match results screen.

## Design rules

- UI is optimistic only in presentation (e.g. disable a button after tapping); actual state
  changes always come back through replication. Never locally mutate bank/resources.
- Touch-first layout: large hit targets, panels reachable one-handed; runs at frame rate.
