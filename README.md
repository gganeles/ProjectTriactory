# Project Triactory

A multiplayer territory-control game on a triangular-tile grid. Game design lives in
[Game Mechanics.md](Game%20Mechanics.md).

## Stack & locked decisions

- **Rust** (edition 2024), **Bevy 0.19**, **Lightyear 0.28**.
- **Mobile-first client** (iOS/Android), developed desktop-first and packaged for mobile last.
- **Match-based** world: one match per server process, all state in memory, no database.
- **Fully server-authoritative** simulation. The client predicts only its own hero's movement
  (with rollback); every other entity is interpolated or plainly replicated.
- Fixed-timestep simulation at **30 Hz** shared by both apps; rendering/UI at frame rate.

## Crate map

The repo will become a Cargo workspace with three members:

| Crate | Role |
|---|---|
| [shared/](shared/) | The Lightyear protocol crate: grid math, components, messages/channels/inputs, static game data, and the deterministic simulation systems that run on **both** server and client (prediction/rollback). |
| [server/](server/) | Headless authoritative app: netcode listener, match lifecycle, map generation, and every gameplay system that mutates state (occupation, combat, production, economy, vision). |
| [client/](client/) | Player-facing app: rendering, touch/mouse input, UI, prediction/interpolation setup, and the client's memory of revealed terrain. Owns [client/assets/](client/assets/) (textures, fonts, audio) — the server is headless and has none. |

## Core design rules

1. **Tiles are not entities.** Terrain is static after map generation. The server holds a
   `TileMap` resource; clients learn terrain through reliable `TilesRevealed` messages as fog
   lifts. Only dynamic things (heroes, biomes, biome archers, NRPs/ARPs) are ECS entities and
   are replicated.
2. **Any system that touches a Predicted component lives in `shared/`** and must be
   deterministic — it runs on the server and again on the client during rollback. Everything
   else that mutates game state is server-only.
3. **State flows through replicated components; messages are for the rest** — terrain reveal,
   request/response commands (purchases, builds, connections), and transient VFX events.
4. **Fog of war is enforced server-side** via interest management: a client is only sent
   entities inside its tribe's current vision or on tribe-owned biome tiles.

## Roadmap (milestones)

M0 workspace → M1 grid + local map render → M2 netcode + predicted hero movement →
M3 vision/fog → M4 occupation + economy → M5 dev tree + NRP → M6 ARP → M7 combat →
M8 biome leveling → M9 match lifecycle/lobby → M10 mobile builds.

The full architecture reference (coordinate system, replication table, channels, schedules,
testing strategy) is documented per-folder in each directory's `README.md`.
