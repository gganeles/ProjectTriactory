# `client/src/ui` — screens and HUD

Bevy UI widgets and screens. Reads replicated state; issues command messages; renders results
(success arrives as replicated component changes, failure as `CommandRejected { reason }`
toasts).

Built with [`bevy_egui`](https://github.com/vladbat00/bevy_egui) (immediate-mode, touch-capable —
confirmed compatible with mobile via its `TouchInput` handling), not `bevy_lunex`: `bevy_lunex`
is pinned to Bevy 0.18 even on its dev branch, and this project is locked to Bevy 0.19
(`lightyear 0.28` requires it), so it can't be wired into this app's `App` at all — Cargo will
happily build it as a second, incompatible copy of every Bevy crate, but its `Plugin`/`Component`
types are then simply not the same types as this app's.

## Files

- `main_menu.rs` — the `AppState::MainMenu` screen: a "Start" button that calls
  `NextState<AppState>::set(Game)`. Deliberately minimal; the richer client-only screen state
  machine described below (`Connecting`, `Lobby`, etc.) isn't built yet.

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
