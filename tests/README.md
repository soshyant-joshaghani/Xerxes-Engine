# Tests

```bat
__ctrl__\xerxes-ctrl.bat test all
__ctrl__\xerxes-ctrl.bat test engine
__ctrl__\xerxes-ctrl.bat test backend
__ctrl__\xerxes-ctrl.bat test templates
__ctrl__\xerxes-ctrl.bat test games
__ctrl__\xerxes-ctrl.bat test contract
```

| Folder | Runner | What it covers |
|--------|--------|----------------|
| `engine/` | `cargo test` in `engine/` (`[[test]]` targets point here) | Bridge semantics; Bevy boundary on a headless app |
| `backend/` | `cargo test` in `backend/` (`[[test]]` targets point here) | Auth, users, private routes, engine catalog and play hosts. In-memory fakes; no database. |
| (games, templates) | `cargo test` in each `games/<name>/` / `games/__templates__/<name>/` | Each game's own tests, then its wasm32 check (`test games` / `test templates`) |
| `contract/` | `python tests/contract/contract_test.py --base URL --local --jobs` | A running API against [CONTRACT.md](../../../CONTRACT.md). |

`test engine` also checks that the engine app compiles for wasm32 with `--features web`. `test templates` and `test games` run each template's or game's own tests and its wasm32 check.
