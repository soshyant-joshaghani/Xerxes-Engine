# CLI

```bat
__ctrl__\xerxes-ctrl.bat <command>
```

| Command | Effect |
|---------|--------|
| `setup-local` | Install Rust, dx and the wasm32 target if missing; `cargo fetch` backend and engine |
| `backend run dev [--slim]` | Infra (Postgres, Redis, Traefik, Adminer), then the API (with multiplayer) and worker on the host. `--slim`: no Redis, no worker |
| `backend run prod\|production [--slim]` | Production compose on this machine (`compose.yml`) |
| `backend stop\|down\|purge\|reset dev\|prod` | Stop (containers kept), remove containers, wipe data volumes, wipe and run again. `--only infra\|apps` narrows `dev` |
| `backend logs dev\|prod <svc>` | Follow host or compose logs |
| `engine build <platform\|all>` | Compile the engine app's dev build, so `run` starts fast |
| `engine run <platform> [--built] [--release]` | Live (web: `dx serve` on :5100, `engine.localhost`, hot reload) or the published build from `dist/engine/` |
| `engine publish <platform\|all>` | Release build into `dist/engine/<platform>/` |
| `game list` | Games in `games/`, A-Z from 1 |
| `game build [index\|name] [platform\|all]` | Compile the dev build |
| `game dev [index\|name] [platform]` | `dx serve` on :5200 (`game.localhost`), `cargo run`, or `cargo apk run` on the adb device |
| `game run [index\|name] [platform] [--built]` | Same as `dev`; `--built` runs the published build from `dist/<name>/<platform>/` |
| `game publish [index\|name] [platform\|all]` | Release build into `dist/<name>/<platform>/` |
| `game new <name> [--from <template>]` | Copy a template from `games/__templates__/` into `games/<name>`, git init |
| `template list\|build\|dev\|run\|publish` | The same for `games/__templates__/` |
| `cleanup [--dry-run] [--yes]` | Delete everything Git ignores (`target/`, `dist/`, generated `public/`, logs, caches) in the engine repo and in each game's own repo. Keeps `.env`, `.env.local`, `.vscode/`, `letsencrypt/`, `__temp__/` (your scratch and reference material), `__ctrl__/.venv/`, `__ctrl__/.run/`; never deletes a game |
| `test all` | Backend, engine, templates, games |
| `test engine` / `templates` / `games` / `backend` | One part |
| `test contract [--no-jobs]` | Wire contract against a running API (`--no-jobs` in slim mode) |
| `prod backup-acme\|restore-acme\|prune-build\|setup-ubuntu` | Production chores (SSL, build cache, VM setup); the stack itself is `backend run prod` |
| `logs` | Production logs over SSH |
| `flatten` / `restore-flat` | Single-root git history |
| `ping`, `clone`, `env`, `start`, `stop`, `status`, `update` | SSH operations from `servers.json` |

Platforms: `web`, `windows` (alias `win`), `android`, `ios` (macOS with Xcode only; `all` skips it). Pipeline details: [games.md](games.md). Linux and macOS use `xerxes-ctrl.sh`. Details: [`__ctrl__/README.md`](../__ctrl__/README.md).
