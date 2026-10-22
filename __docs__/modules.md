# Modules

## Engine (`engine/src/modules/`)

| Module | Holds |
|--------|-------|
| `assets`, `bundles`, `project`, `flow` | Authored assets, bundles, project settings, the flow (splash, menu, levels) |
| `camera`, `input`, `physics`, `matches`, `ui` | Camera rigs, the input actions, physics (Bevy plugin over `xerxes_sim`), match runtime, UI (Canvas, widgets, touch) |
| `runtime`, `scene`, `clock`, `storage`, `safe_area` | Runtime plumbing, scenes, the clock, platform storage, safe-area insets |

The editor's bridge between the UI and Bevy is in `engine/src/editor/bridge/` (no Bevy, no Dioxus), and its Bevy side in `engine/src/editor/stage/runtime.rs`. The Dioxus viewport canvas is `GameViewport` in `engine/src/editor/host/web.rs`. These are editor-only, not engine modules.

## Backend (`backend/src/modules/`)

Two series: `api/` ([backend-api.md](backend-api.md)) and `multiplayer/` ([backend-multiplayer.md](backend-multiplayer.md)).

| Group | Holds |
|-------|-------|
| `api/engine`, `api/editor` | Local dev services: game catalog and `<name>.play.localhost`; project files, templates and jobs |
| `api/<name>` | Product modules (none yet) |
| `api/base/auth`, `api/base/users` | Login, session, user records |
| `api/system` | Health and private dev routes |
| `multiplayer/` | Rooms and realtime (the M track of Phase 3; only a health route so far) |

To add a backend module:

1. Build `backend/src/modules/api/<name>/` as router → service → repository → schemas (the same layout as `base/users`).
2. Merge the new router in `backend/src/modules/api/mod.rs`.
3. Add `backend/migrations/NNNN_<name>.sql` when tables change.
4. Add tests in `tests/backend/`.
