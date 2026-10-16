# xerxes `__ctrl__`

The **control layer** for Xerxes — one predictable CLI for the project lifecycle.

`__ctrl__` is not a loose collection of scripts. It is the official interface for:

- Starting and stopping dev infrastructure and apps
- Selecting runtime profiles (Full / Slim)
- Running and building games on the Xerxes engine
- Running tests
- Deploying to production via SSH

```text
Xerxes
│
├── Engine + Games        (engine/ library crate, games/<name> crates)
├── Application Layer     (backend: API + worker)
├── Infrastructure Layer  (db, redis, workers, proxy)
└── Control Layer         __ctrl__/  ← you are here
```

Prefer `__ctrl__` commands over ad-hoc `docker compose` or manual process management unless you have a specific reason.

This is **not** foxg-ctrl. FoxG platform VMs stay under `foxg-ctrl`; Xerxes ops live here.

## Quick start (Windows)

From `xerxes-engine/__ctrl__/`:

```bat
xerxes-ctrl.bat
```

Interactive prompt, or one-shot:

```bat
xerxes-ctrl.bat setup-local
xerxes-ctrl.bat backend run dev
xerxes-ctrl.bat engine run web
xerxes-ctrl.bat game list
xerxes-ctrl.bat test all
xerxes-ctrl.bat list
xerxes-ctrl.bat connect
```

Linux/mac:

```bash
chmod +x xerxes-ctrl.sh
./xerxes-ctrl.sh status
```

## Command map

| Area | Commands |
|------|----------|
| Local tooling | `setup-local [--force]` · `cleanup [--dry-run] [--yes]` (every `target/` and `dist/`) |
| Backend | `backend run\|stop\|down\|purge\|reset {dev,prod\|production}` · `--slim` (no Redis/worker) · `--only infra\|apps` (dev) |
| Engine app | `engine build\|run [--built]\|publish <web\|windows\|android\|ios\|all>` |
| Games | `game list` · `game build [index\|name] [platform\|all]` · `game dev\|run [index\|name] [web\|windows\|android]` · `game publish [index\|name] [platform\|all]` · `game new <name> [--from <template>]` |
| Templates | `template list` · `template build ...` · `template dev\|run ...` · `template publish ...` |
| Tests | `test {all,backend,engine,templates,games,contract}` (`all` = backend + engine + templates + games; `contract --no-jobs` in slim mode) |
| Logs | `logs <svc>` (VM) · `backend logs dev <svc>` · `backend logs prod <svc>` |
| Production chores | `prod backup-acme\|restore-acme\|prune-build\|migrate-acme\|setup-ubuntu` |
| SSH / VM | `setup`, `pubkey`, `clone`, `env`, `start`, `stop`, `update`, `reset`, `backup-acme`, `connect`, … |

On-VM bash/bat scripts (what SSH `start`/`stop` invoke) live in [`remote/`](remote/README.md).

## Layout

| Path | Role |
|------|------|
| `servers.json` | Single VM entry (`xerxes`) |
| `safe/` | PEM, address, prod `.env` |
| `static/gpg` | Docker Ubuntu GPG (Iran bootstrap) |
| `remote/` | On-VM / local-prod compose scripts |
| `xerxes-ctrl.bat` / `.sh` | CLI entry |

## Typical first deploy (SSH)

```bat
xerxes-ctrl.bat setup
xerxes-ctrl.bat pubkey
REM add VM pubkey to GitHub
xerxes-ctrl.bat clone
xerxes-ctrl.bat env
xerxes-ctrl.bat start
```

Day-2:

```bat
xerxes-ctrl.bat update
xerxes-ctrl.bat status
xerxes-ctrl.bat backup-acme
```

## Backend (Docker Desktop / host apps)

```bat
xerxes-ctrl.bat setup-local
xerxes-ctrl.bat backend run dev
xerxes-ctrl.bat backend run dev --slim
xerxes-ctrl.bat backend run prod            REM or: production
xerxes-ctrl.bat backend stop dev
xerxes-ctrl.bat backend down dev
xerxes-ctrl.bat backend purge dev --only infra
xerxes-ctrl.bat backend reset dev
xerxes-ctrl.bat backend logs dev api
```

The backend is the API and multiplayer series in one process today; `backend run` starts both. When multiplayer moves to its own container the same command will start both containers (see [__docs__/backend.md](../__docs__/backend.md)).

| Action | Infra (compose.dev.yml) | Apps (host) |
|--------|-------------------------|-------------|
| `run` / `start` | `up -d` db, redis (full), proxy, adminer | Rust (Axum) API :8000 (runs SQL migrations), Redis queue worker (full) |
| `stop` | `compose stop` — containers kept | kill host processes |
| `down` | `compose down` — volumes kept | kill host processes |
| `purge` | `compose down -v` — wipe data, stay down | kill host processes |
| `reset` | wipe then `run` | stop then run |

| `--only` (dev) | Notes |
|----------------|-------|
| `infra` | Docker only. SQL migrations run when the API starts |
| `apps` | host processes (needs infra already up) |
| (none) | run: infra→apps · stop/down/purge/reset: apps→infra |

`prod` / `production` runs `compose.yml` on this machine through the same scripts SSH uses (`remote/start-prod.*`, `stop-prod.*`, `reset-prod.*`); `purge prod` is `reset-prod` (wipes the data volumes and app images, keeps SSL).

Opens browser tabs for Adminer / Traefik / API docs after a successful run. The engine app and games are not part of the dev stack: start the engine with `engine run web`, a game with `game dev`.

**Runtime profiles:** `backend run dev` (full — includes Redis + worker) · `backend run dev --slim` (no Redis/worker); the same flag on `prod`. See [__docs__/runtime-profiles.md](../__docs__/runtime-profiles.md).

## Games (Xerxes engine)

Every target is a Dioxus + Bevy frontend, and all of them go through the same pipeline. Full description: [__docs__/games.md](../__docs__/games.md).

| Index | Target |
|-------|--------|
| `game list` | Games made with the engine (`games/<name>/`; `game new` games are their own Git repos, the Darius games live in the engine repo), A-Z from 1. The engine app (`engine/`) has its own `engine` commands |
| `template list` | Finished games kept as templates (`games/__templates__/<name>/`, committed with the engine) A-Z |

```bat
xerxes-ctrl.bat engine run web              REM engine app, http://localhost:5100
xerxes-ctrl.bat template dev darius windows    REM Bevy window
xerxes-ctrl.bat template run 1 android         REM same as `dev`, by index or name; APK on the adb device
xerxes-ctrl.bat template publish darius all    REM dist/darius/{web,windows,android}/
xerxes-ctrl.bat game new my-game            REM from the engine's built-in starter, git init
xerxes-ctrl.bat template dev <name>      REM a finished template game
```

| Command | Runs (in the target's folder) |
|---------|-------------------------------|
| `dev [t] web` (default) | `dx serve --platform web --port 5100`: Dioxus page, Bevy canvas, HUD on top |
| `dev [t] windows` | `cargo run`: a Bevy window (no Dioxus overlay yet) |
| `dev [t] android` | `cargo apk run --example android --features android --target <device ABI>` (cargo-apk, NativeActivity) |
| `publish [t] web` (default) | `dx bundle --platform web --release --out-dir dist/<name>/web` |
| `publish [t] windows` | `cargo build --release`, then the executable and `assets/` are copied to `dist/<name>/windows/` |
| `publish [t] android` | `cargo apk build --release --example android` → `dist/<name>/android/<name>.apk` |

Without a target the CLI shows the index and asks. Every command runs with `CARGO_TARGET_DIR=engine/target`, so Bevy is built once per feature set, and with `CARGO_BUILD_JOBS` = half the cores unless you set it, so Bevy doesn't exhaust memory. `dx` is pinned to `DX_VERSION` in `lib/runtimes.py` and installed with `cargo binstall`; web commands also add the `wasm32-unknown-unknown` target.

## Tests

```bat
xerxes-ctrl.bat test all
xerxes-ctrl.bat test backend
xerxes-ctrl.bat test engine
xerxes-ctrl.bat test templates
xerxes-ctrl.bat test games
xerxes-ctrl.bat test contract
```

Backend tests use in-memory fakes and need neither Postgres nor Redis. `test engine` runs `cargo test` in `engine/` (tests in `tests/engine/`), then `cargo check --features web --target wasm32-unknown-unknown`. `test templates` / `test games` do the same in each template / game. `test all` = backend + engine + templates + games. `test contract` checks a running API against the wire contract.

## Troubleshooting: ports on Windows

`dx serve` or `adb` failing with "an attempt was made to access a socket in a way forbidden by its access permissions" (error 10013) does not mean something else holds the port. Windows reserves ranges for Hyper-V, WinNAT, WSL and Docker, and a bind inside one fails: `netsh interface ipv4 show excludedportrange protocol=tcp` lists them (which move around, especially when the TCP dynamic range starts low: `netsh int ipv4 show dynamicport tcp`; the default start is 49152).

- The web dev servers (`engine run web` on 5100, `game dev <x> web` on 5200) pick the next bindable port themselves and print it (`http://localhost:<port>`; Traefik's `engine.localhost` / `game.localhost` then do not match). Force one with `XERXES_PORT_5100=<port>` / `XERXES_PORT_5200=<port>`.
- `adb` (default 5037): `set ANDROID_ADB_SERVER_PORT=5043` before running the Android commands.
- The permanent fix, in an admin shell, then reboot: `netsh int ipv4 set dynamicport tcp start=49152 num=16384` (same for `ipv6`).

## Troubleshooting: link error LNK1140 / "No space left"

A Windows link failing with `LNK1140: limit exceeded for program database` can simply mean the drive is full: Bevy builds take tens of GB (`engine/target` alone about 34 GB). Run `xerxes-ctrl cleanup`.

## Production on this machine

```bat
xerxes-ctrl.bat backend run prod
xerxes-ctrl.bat backend stop prod
xerxes-ctrl.bat backend reset prod
xerxes-ctrl.bat prod backup-acme
```

Same scripts SSH uses under `remote/`. Prefer SSH `start`/`stop` when operating the real VM from your laptop.

## Setup (ctrl tool itself)

```bat
python -m venv .venv
.venv\Scripts\pip install -r requirements.txt
```

On first `setup-local` / `backend run dev`, the ctrl entry installs system **Python 3.10+** (via winget / Homebrew / apt) if missing. `_setup_local` then installs **Rust** (rustup: winget `Rustlang.Rustup` on Windows, `sh.rustup.rs` elsewhere), **dx** (`cargo binstall dioxus-cli`), and the `wasm32-unknown-unknown` target, and runs `cargo fetch` for `backend/` and `engine/`. No Node.js is involved. When Rust cannot be installed but Docker is available, the backend still runs in a `rust:1` container. The engine and games always need host Rust. `setup-local --force` deletes `backend/target` and `engine/target` first.

Iran VMs (`iran_setup: true`) keep provider DNS, rewrite apt to Arvan `apt_mirror`, and use Arvan Docker `registry_mirror`. `clone` routes GitHub SSH via `ssh.github.com:443`.
