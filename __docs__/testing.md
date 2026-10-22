# Testing

```bat
__ctrl__\xerxes-ctrl.bat test all
__ctrl__\xerxes-ctrl.bat test engine
__ctrl__\xerxes-ctrl.bat test backend
__ctrl__\xerxes-ctrl.bat test contract
```

`test engine` runs `cargo test` in `engine/`: the tests in `tests/engine/*.rs` (the bridge's ordering, change detection, subscribers and threads; the Bevy boundary on a headless `App`) and the engine app's unit tests. Then the engine app must type-check for wasm32 with `--features web`.

`test templates` and `test games` run `cargo test` in each template or game, then type-check its web build for wasm32.

`test backend` runs `cargo test` in `backend/`. Integration tests live in `tests/backend/*.rs`; they inject in-memory repositories, cache and job queue, so they need neither Postgres nor Redis.

`test contract` runs `tests/contract/contract_test.py` against a running API (default `http://localhost:8000`).

`test all` runs the backend, engine, templates and games.
