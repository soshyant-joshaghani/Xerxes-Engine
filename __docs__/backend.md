# Backend: two series, one process, built to split

The backend (`backend/`) holds two series of modules. Today they run together in one process; the layout keeps the multiplayer series ready to move to its own container when WebSocket load calls for it.

```text
backend/src/
  core/                 shared: config, db, cache, jobs, security, errors, state, openapi
  modules/
    api/                the HTTP API, mounted at /api/v1 (+ /docs, /sdoc)
      base/             auth, users
      system/           health, private dev routes
      engine/           local dev services: game catalog, <name>.play.localhost
      editor/           local dev services: project files, templates, jobs
      <name>/           product modules (router → service → repository → schemas)
    multiplayer/        rooms and realtime (the M track of Phase 3), mounted at /multiplayer
```

## The rules that keep the cut cheap

1. **Neither series imports the other.** `multiplayer` never uses `modules::api`, and `api` never uses `modules::multiplayer`. Shared code goes in `core`. `tests/backend/modules.rs` fails the build when one imports the other. Needing a user or a rule from the other side means moving it into `core`, or calling it over HTTP.
2. **Each series mounts under its own prefix** (`/api/v1`, `/multiplayer`), so a reverse proxy can send each prefix to its own container.
3. **`SERVICES` picks what a process serves:** `all` (default), `api` or `multiplayer`. A `multiplayer` process answers `/multiplayer/*` (with `/multiplayer/health-check` as its probe) and nothing else; an `api` process does not know `/multiplayer`.

`build_router` composes the series (`api_router`, `multiplayer_router` in `backend/src/lib.rs`), with CORS and the access log around them.

## Today: together

```bat
__ctrl__\xerxes-ctrl.bat backend run dev          REM infra + API (with multiplayer) + worker
__ctrl__\xerxes-ctrl.bat backend run dev --slim   REM no Redis, no worker
__ctrl__\xerxes-ctrl.bat backend run prod         REM compose.yml: one `backend` service, SERVICES=all
```

## Later: apart

Because both series are routed by prefix and configured by `SERVICES`, the split needs no code changes:

1. In `compose.yml`, set `SERVICES: api` on `backend` and uncomment the `multiplayer` service (same image, `SERVICES: multiplayer`).
2. Give Traefik a `PathPrefix(/multiplayer)` router for the new service; the API keeps its host.
3. When the multiplayer container needs different dependencies (no SQLx, say), add `src/bin/multiplayer.rs` that calls `multiplayer_router()` and give it its own image. `AppState` already comes from `core`.

## Adding to a series

- API: `backend/src/modules/api/<name>/` and merge its router in `api/mod.rs` (see [modules.md](modules.md)).
- Multiplayer: `backend/src/modules/multiplayer/<name>/` with its own router, service and schemas, merged in `multiplayer::router()`. The M track (PROGRESS.md) builds the rooms, stage by stage.
