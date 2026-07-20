# `client/assets` — bundled game assets

Loaded at runtime via Bevy's `AssetServer`. **Client only** — the server is headless and never
touches a texture, font, or sound. In development, Bevy resolves this folder relative to the
running executable (`<crate_root>/assets`); mobile packaging (M10) bundles the same folder
into the iOS asset catalog / Android assets via the platform build tooling — the asset paths
referenced in code should not need to change between desktop dev and mobile builds.

## Subfolders

- [textures/](textures/) — terrain tiles, entity sprites, UI chrome, the texture atlas source
  images consumed by `client/src/rendering`.
- [fonts/](fonts/) — UI typefaces consumed by `client/src/ui`.
- [audio/](audio/) — sound effects and music. Not yet scoped by the mechanics doc; the folder
  exists so asset paths are stable once audio direction is decided, but stays empty until
  then.

## Design rules

- Source art (e.g. `.psd`/`.aseprite` working files) does **not** belong here — only the
  exported runtime assets (`.png`, `.ttf`, `.ogg`, etc.) that ship with the app. Keep working
  files elsewhere (a separate design repo/folder) to avoid bloating the game's asset bundle
  and this repo's history with large binary revisions.
- Filenames should be stable identifiers referenced directly in code/atlas definitions —
  renaming an asset is a code change, not just a file swap.
