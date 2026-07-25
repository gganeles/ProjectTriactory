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

## Files

- `pan.rs` — drag-to-pan, active only during `AppState::Game` + `GameMode::BuildMode` (see
  `shared/src/states.rs`). Reads `Touches`/mouse directly rather than through a classifier —
  there's only one gesture so far, nothing to disambiguate against yet. Converts the drag's
  screen-space position to a world-space point each frame via `Camera::viewport_to_world`
  intersected with the map's `Y = 0` ground plane, and moves the camera by the delta between
  frames, so the map tracks the finger exactly (not a fixed pan speed). Clamps the camera's XZ
  position to within `MapBounds::radius` (scaled by the current zoom) of
  `MapBounds::camera_home_xz` — deliberately *not* the world origin, since the resting camera
  translation is already offset in Z for its angled "2.5D" tilt, and clamping against the origin
  instead snaps the camera the instant a drag starts. Should fold into `touch.rs`'s classifier
  once tap-to-move or connection-dragging need to disambiguate against it.
- `zoom.rs` — zoom from three independent input sources, active any time `AppState::Game` is
  active (not mode-gated — zoom isn't specific to Build/Play the way dragging is): touchscreen
  pinch (first two active `Touches`), external-mouse scroll wheel (`AccumulatedMouseScroll`),
  and macOS/iOS trackpad pinch (`bevy::input::gestures::PinchGesture` — a distinct native
  gesture, not reported through `MouseWheel`; requires the `gestures` feature enabled on `bevy`
  in the workspace `Cargo.toml`). Adjusts the camera's `OrthographicProjection::scale` directly
  and clamps it to a fixed `MIN_SCALE..MAX_SCALE` range.

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
