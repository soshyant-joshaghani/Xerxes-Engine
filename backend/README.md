# Backend (Rust / Axum)

Implements the FoxG wire contract ([CONTRACT.md](../../../CONTRACT.md)), plus local-only engine dev services (`src/modules/api/engine/`: the game catalog and `<name>.play.localhost`). Layers: Router -> Service -> Repository -> Postgres.

```text
src/core/      config, db + migrations, cache, jobs, security, errors, state, openapi
src/modules/   two series, one process today, neither imports the other:
                 api/          base/{auth,users}, system, engine, editor, <product modules>  (/api/v1)
                 multiplayer/  rooms and realtime, Phase 6                          (/multiplayer)
src/bin/       api (HTTP server), worker (Redis list worker)
migrations/    plain SQL, applied at startup in name order
../tests/backend/  integration tests (in-memory fakes, no Postgres/Redis needed)
```

## Commands

```sh
cd backend
# Cargo.lock is committed; `cargo generate-lockfile` recreates it
cargo test                # integration tests from ../tests/backend
cargo run --bin api       # listens on APP_HOST:APP_PORT (default 0.0.0.0:8000)
cargo run --bin worker    # BRPOP foxg:jobs
```

`SERVICES=all|api|multiplayer` (default `all`) picks which series a process serves, so the multiplayer series can move to its own container without code changes ([__docs__/backend.md](../__docs__/backend.md)).

Configuration is read from the environment, then `.env` in `.`, `..`, `../..`. Migrations come from
`MIGRATIONS_DIR`, `./migrations` or `../migrations`. Docs: `/docs` (Swagger), `/sdoc` (Scalar),
`/api/v1/openapi.json`. Optional extra env: `BCRYPT_COST` (default 12).

Docker: `docker build -f backend/Dockerfile .` from the kit root. The image runs `/app/api`; the worker is `/app/worker`.
