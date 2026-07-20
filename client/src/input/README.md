# `client/src/input` — touch/mouse input

Turns raw touches (or mouse, on desktop) into game intents. Mobile-first: the whole model is
taps, drags, pinches, and long-presses.

## The gesture model

Disambiguation is by **what the gesture starts on**, centralized in one classifier:

| Gesture | Starts on | Meaning |
|---|---|---|
| Tap | a tile | Move hero there (A* path + path preview) |
| Drag | an **owned NRP/ARP** | Connection drag: rubber-band line; release on a valid target (ARP/BT) sends `CreateConnection`, otherwise cancels |
| Drag | anywhere else | Camera pan |
| Pinch | — | Camera zoom |
| Long-press | biome / building | Open inspection panel |

## Planned files

- `touch.rs` — the gesture classifier built on Bevy `Touches`, with mouse fallback on desktop
  (click = tap, click-drag = drag, scroll wheel = pinch). Emits high-level gesture events;
  nothing else in the app reads raw touches.
- `movement.rs` — tap-to-move: picks the tile via `TriCoord::from_world`, runs A*
  (`shared/grid/path`) over `RevealedTiles`, then emits one
  `HeroInput { step }` per fixed tick following the path. Input emission runs in
  `FixedUpdate` (`SimSet::ApplyInputs`) because it feeds prediction.
- `connect_drag.rs` — connection dragging state machine: validates the start entity is an
  owned NRP/ARP, drives the rubber-band preview, sends `CreateConnection { from, to }` on a
  valid release. The server re-validates; this is UX-side filtering only.

## Design rules

- The client never sends positions — only per-tick edge-step inputs and command messages.
- Keep the classifier the single owner of gesture disambiguation; adding a gesture means
  editing one file.
