# Modules

## Engine (`engine/src/modules/`)

| Module | Holds |
|--------|-------|
| `bridge` | UI → runtime commands, runtime → UI snapshots. No Bevy, no Dioxus |
| `runtime` | Bevy side: `primary_window`, `UiBridgePlugin`, `UiBridge`, `UiCommand`, `run_once` |
| `viewport` | Dioxus side: `GameViewport` canvas, `use_runtime_snapshot` |
| `scene` | `Scene` trait (Bevy world + Dioxus HUD) and `launch` |

## Backend (`backend/src/modules/`)

Two series, see [backend.md](backend.md): `api/` and `multiplayer/`.

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
