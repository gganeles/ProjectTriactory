# Next Steps

Generated from the completed Game Architecture workflow. Full detail lives in
[`_bmad-output/game-architecture.md`](_bmad-output/game-architecture.md); rules AI agents must
follow when implementing code live in [`_bmad-output/project-context.md`](_bmad-output/project-context.md).

## Current priority

Fix the map-generation tile-area fairness bug — biome-count parity between tribes is not
sufficient, tile-area parity is the actual requirement. Tracked against:
- `server/src/game/map/generation/`
- `shared/src/game/map/generation.rs`

Land any fix with a regression test proving fairness, not just a manual check.

## Architecture setup (once map generation is resolved)

1. **Add avian2d** to the workspace (`Cargo.toml`), with the `enhanced-determinism` feature —
   non-optional, since physics steps touch `Predicted` components and must stay bit-consistent
   across rollback.
   ```bash
   cargo add avian2d --features "2d,enhanced-determinism" -p triactory_shared
   ```
   Wire its physics step into `FixedPostUpdate` in `shared/`.

2. **Add `AssetPlugin` to the server** — `MinimalPlugins` doesn't include it, but the RON
   tuning-data decision requires it on both apps.

3. **Create the shared `data/` directory** at repo root for RON gameplay-tuning files, loaded via
   `AssetServer` on both server and client.
   ```bash
   mkdir -p data
   ```

4. **Configure the Context7 MCP server** (selected during architecture) for up-to-date
   Bevy/Lightyear docs in AI sessions:
   ```bash
   claude mcp add context7 -- npx -y @upstash/context7-mcp
   ```

## Explicitly deferred (do not implement without a dedicated design pass)

- Fog of war / server-side interest management (E3)
- AI fill-in tribes / minimum-player lobby gating (E9)
- NRP → ARP → BT connection topology and the Biome Tower capture-timer rule (E4/E6)
- Mobile touch gesture set (E10) — GDD marks this a placeholder pending a `gds-ux` pass

## After architecture setup

- **Create Epics** — break the GDD into implementation-ready stories with
  `gds-create-epics-and-stories` (inputs: GDD + `game-architecture.md`)
- **Begin implementation** — every AI agent should read `project-context.md` before writing game
  code

---

_Generated 2026-08-08 via the GDS Architecture workflow._
