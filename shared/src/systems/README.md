# `shared/src/systems` — shared simulation systems

Systems that run on the **server** (authority) and again on the **client** during prediction
rollback. This is the most constrained code in the project.

## Rule of placement

> If a system touches a Predicted component, it lives here. Otherwise it is server-only and
> belongs in `server/src/systems`.

Currently that means exactly one gameplay system: hero movement. Everything else (occupation,
combat, production, economy, vision) mutates non-predicted state and stays on the server.

## Planned files

- `mod.rs` — the `FixedUpdate` system sets, in order:
  `SimSet::ApplyInputs → SimSet::Movement → SimSet::Gameplay → SimSet::PostSim`.
  All simulation runs in `FixedUpdate` at 30 Hz (`config::TICK_RATE`); rendering and UI run at
  frame rate in the client only.
- `movement.rs` — `hero_movement`: consumes the per-tick `HeroInput { step: Option<EdgeDir> }`
  and advances `HeroTile` / `HeroKinematics` by one edge crossing. Validates adjacency and
  skill-gated terrain (water/mountain need the unlocked skill).

## Determinism requirements

- Pure function of inputs + component state + static data. No wall-clock time (`Time<Fixed>`
  only), no randomness, no iteration-order dependence on `HashMap`.
- Client and server run *the same function*; rollback reconciliation converges only if the
  results match bit-for-bit.
- A client predicting a step onto terrain it hasn't revealed yet may mispredict — the server
  correction rolls it back. That is acceptable and self-healing; do not try to special-case it.
