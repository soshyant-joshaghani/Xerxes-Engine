# Backend: API

The API is the **global** backend: one service for the whole platform, used by every game. It holds accounts, saved data, leaderboards and the other services a game needs outside a match. It holds **no game logic**. Games reach it through the engine bridge `engine/src/modules/api`.

For realtime play see [backend-multiplayer.md](backend-multiplayer.md). Stages are in [PROGRESS.md](../__plans__/PROGRESS.md).

## What games need from it

| Need | Used for | Stage |
|---|---|---|
| Accounts and sessions | sign up, log in, the player id that rooms and boards use | done (wire contract) |
| Health and dev routes | probes, local development | done |
| Saved data | a player's save: a named record per player, with a version, read and written by the save module | 3.6 |
| Leaderboards | a board per game and per level: submit a score or time, read the top N and the player's place | M5 |
| Ratings | pairwise ratings (Glicko-2) for duels and brackets | M5 |
| Tournaments | brackets stored here; each round plays out in rooms | M5 |
| Live listing | which 24/7 rooms are open, for the menu | M5 |

Each row is added when a game needs it. A row counts as done only when a darius or cyrus scene uses it.

## Structure

Route → Service → Repository → PostgreSQL. Redis holds the cache and the job queue, and the cache degrades to nothing when Redis is down.

```text
backend/src/modules/api/
  base/        auth, users                  (done)
  system/      health, private dev routes   (done)
  engine/      local development only: game catalog, <name>.play.localhost
  editor/      local development only: project files, templates, jobs
  <name>/      product modules, one folder per service above
```

- Each module is an `aide` router with typed extractors. Swagger (`/docs`) and Scalar (`/sdoc`) are generated from the handlers, not written by hand.
- Errors are `ApiError`; the body is always `{"detail": "..."}` with the status from the wire contract.
- Tables are added as `backend/migrations/NNNN_name.sql` with `IF NOT EXISTS`, applied when the API starts.
- `api/engine` and `api/editor` are scaffolding for the local editor and catalog. They are not used in production and hold no gameplay.

## Game data in the API

A game declares what the API needs in its `<name>.project.rs` (`BackendSettings`): its backend URL, and the ids of its boards. The API never holds a game's rules, scores' meaning or level data. It stores a number under a board id and returns it.

## The engine bridge: `engine/src/modules/api`

The only API code a game calls. Bevy-side, Rust, the same calls on web, windows and android.

- Sign in and keep the session; read and write saved data; submit to a board and read it.
- Each call is added with the service it uses, not before.
- A new endpoint is added in two places: the route in `backend/src/modules/api/<name>/`, and the call in the bridge.

## Running it

```bat
__ctrl__\xerxes-ctrl.bat backend run dev     REM API and multiplayer together, for development
__ctrl__\xerxes-ctrl.bat backend run prod    REM the production compose file
```

In production the API runs as its own service, next to Postgres and Redis. The API serves `/api/v1`; the multiplayer series is a separate service ([backend-multiplayer.md](backend-multiplayer.md)).
