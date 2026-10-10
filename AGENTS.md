# AGENTS.md — AI development contract

Read this file before architectural changes in Xerxes.

Xerxes is a fullstack game engine built on Bevy: the engine and editor, plus an API and a Rust multiplayer server that is a port of Colyseus (rooms, matchmaking, state sync; see `__docs__/multiplayer.md`) that grow alongside them, so developers are not left to build the backend. Games are Bevy only (world and UI in bevy_ui). The engine app, the editor where games are created, adds a Dioxus UI over its Bevy viewport, from one source on every platform. It grows bottom-up from small real games. It started from the Rust-Dioxus kit and keeps that kit's Axum backend.

## Start here

| Question | Answer |
|----------|--------|
| What to read first? | This file → [`__plans__/PROGRESS.md`](__plans__/PROGRESS.md) (Phase 3 is the current plan) → [__docs__/engine.md](__docs__/engine.md) → [__docs__/architecture.md](__docs__/architecture.md). For Phase 3 also [darius](__docs__/darius.md), [coverage](__docs__/coverage.md), [warp](__docs__/warp.md), [dialogue-quest](__docs__/dialogue-quest.md), [physics](__docs__/physics.md), [multiplayer](__docs__/multiplayer.md) |
| Where is the engine? | `engine/` (crate `xerxes_engine`), modules in `engine/src/modules/` (`runtime`, `scene`): Bevy only |
| Where is the engine app? | `engine/src/main.rs` + `engine/src/editor/` (feature `editor`, default): the editor where games will be created. Dioxus UI (`panel`, `catalog`) in one host page (`host/`), rendered by Dioxus web on the web and by Blitz over the Bevy window natively |
| Where do games go? | `games/<name>/`: its own crate, depending on `../../engine` (`default-features = false`), with its own `assets/`. A game made with `game new` is its own Git repo and is gitignored here (`games/` is gitignored). `games/__templates__/<name>/` is committed with the engine: finished games, and the Darius games, which are developed in place there together with the engine |
| Templates? | `games/__templates__/<name>/`: finished games with clear names, and the two games, Darius and Cyrus, (`darius`, `cyrus`) while they are developed; committed with the engine, **read-only in the editor and backend**: to work on one in the engine, clone it into `games/` through the editor ("Use as template"). The Darius content itself is developed directly on disk in `games/__templates__/` by the AI and the authors, not through the editor. New projects do not start from one: the engine creates its own empty `starter` project (editor New Project, `game new`, `xerxes_build::scaffold::starter_files`, files in `engine/starter/`) |
| What is the reference game? | `games/__templates__/darius` (Darius), in progress; run it with `template dev darius [web|windows|android]` |
| Where does multiplayer go? | `backend/src/modules/multiplayer/` (the Colyseus port), the Bevy-free rules and physics crate `xerxes_sim` (`engine/sim/`: physics world done in 3.1, match rules next) and the engine's client plugin (`modules/net`). It is the **M track inside Phase 3** ([PROGRESS.md](__plans__/PROGRESS.md)), built stage by stage as darius's modes need it, not before its dependencies |
| How to run it? | `__ctrl__\xerxes-ctrl.bat backend run dev [--slim]` (infra + backend; `backend run prod|production` for the production compose), `engine run web` (engine app at `engine.localhost`, Play published games from it), `game list`, then `game dev <index|name> [web|windows|android]`; `game publish` for release builds ([__docs__/games.md](__docs__/games.md)) |
| How to test it? | `__ctrl__\xerxes-ctrl.bat test all` (backend, engine, templates, games) |

## Engine rules

- **Xerxes is not a new engine.** It is the editor and tools for making Bevy games: Bevy is the runtime, used directly (no wrapper API). The engine library only adds concepts Bevy lacks (project settings, scenes, prefabs, metas, bundles, input actions). Optional gameplay grows as copyable `.rs` add-ons in `engine/addons/`, never as strict engine components. A game owns its copies; what a game proves worth keeping and is not needed by every game goes back into the library.
- Three logic layers: **engine** (`engine/src/modules`: `assets`, `flow`, `project`; flow splash → main menu → levels, game modes, scenes, prefabs, metas, bundles, input actions; Bevy used directly), **add-ons** (`engine/addons`: optional logic over engine + Bevy, compiled into `xerxes_build` so the editor, backend and CLI all offer it), **game** (`games/<name>/assets`: settings, levels, prefabs, configured copies). Normal users only touch the game layer. Every level has a game mode typed on __docs__/game-modes.md.
- Every web build (dev and publish, engine and games) ships with a Dockerfile (written by `__ctrl__`) so it runs on another server.

- **Bottom-up.** Games come first. Code used once stays in the game. Move it into `engine/src/modules/` only after it repeats and is clearly generic. Never put gameplay or game assets in `engine/`. The exception is decided, not assumed: the tools listed in Phase 3 (match rules, Warp, save, quests, dialogue, text, UI kit, multiplayer) are engine modules from the start because every game needs them; each is still only `done` once a darius scene uses it.
- **One stage at a time**, in the order and with the dependencies PROGRESS.md gives (Phase 3: stages 3.x and the M track). Do not build a stage before the stages it depends on, and do not build the node graph before Phase 10. There is no scripting language in the plan for now (see "Not scheduled yet" in PROGRESS.md): game logic is Rust, and dialogue, quests and Warp use typed `Condition` / `Action` data. Reference material in `__temp__/` (the Unity dialogue/quest package) is local, gitignored and design-only: never commit it, never copy its code, text or art.
- No top-level `crates/` or `examples/` (the engine's own Android entry is `engine/platform/android/main.rs`, declared as a Cargo example for cargo-apk), no `frontend/` dashboard, no `foxg_*` names. Engine library code lives in `engine/src/modules/`, the engine app in `engine/src/editor/`.
- Every target (engine app, templates, games) runs through the same pipeline from day one: `engine build|run|publish`, `game build|dev|publish` (and `template ...`). Keep it working when changing the engine.
- A game in progress lives in `games/<name>` (its own repo, gitignored). Finished games go into `games/__templates__/`. The Darius games are the exception: they are developed in place in `games/__templates__/` (edited as files, by the AI) because they grow with the engine, and they use the `template` commands (`template dev darius`). To open one in the editor, clone it to `games/` first.
- Cross-platform is a hard rule: the engine app (Dioxus + Bevy + engine) and every game (Bevy + engine) must dev and publish on web, windows and android (ios later). Every target builds for web, windows and android: code in the `game` lib, `main.rs` and `android.rs` only call `run()`. Bevy features are spelled out with `android-native-activity` (never the `2d`/`3d` collections, which pull GameActivity).
- Bevy is the runtime (ECS, render, input, assets, scheduling) and the games' UI (bevy_ui: HUD, menus, settings). Dioxus is the engine editor's UI only, in `engine/src/editor/`. A game, and every game export, never depends on Dioxus.
- One source of truth for every target. The editor UI is the same Dioxus components and host page on web, windows and android; per-platform code is limited to renderer glue (`editor/host/web.rs`, `editor/host/native.rs`). Never fork a component, stylesheet or pipeline per platform. Blitz is not a browser: check new CSS natively.
- The editor UI and the stage talk only through the bridge (`editor/bridge`, never depends on Bevy or Dioxus). In a game, the HUD and the world share resources and messages.
- **Scenes are 2D or 3D, games are not.** A game can mix both (a 2D menu level, a 3D world). Each scene declares `dimension: Dimension::D2 | Dimension::D3` in its `.rs`; every game builds both renderers (the `2d` and `3d` Bevy collections). The editor shows a 2D scene flat (orthographic, XY, pixels) and a 3D scene in perspective.
- The engine takes the Bevy features its scene defs need (mesh, pbr, sprite). A game spells out its Bevy features (the `2d` and `3d` collections, with `ui_api`/`ui_bevy_render`), and its `web` feature enables `xerxes_engine/web`.
- `engine/winit` is winit 0.30.13 with one patch (real mouse input on Android: cursor, buttons, wheel; marked "Xerxes patch"). The engine and every game use it through `[patch.crates-io]`; the scaffold points new games at it. When Bevy moves to a winit with Android mouse support, drop the vendored copy.
- **Stable paths.** `engine/` (the crate), `engine/build/` and `engine/winit/` are referenced by relative path from every game, the scaffold and the backend: never move or rename them (see `engine/README.md`). The add-ons live in feature folders in `engine/addons/` (`camera-ctrl`, `chr-ctrl`, `hud`, `game`: conditional logic only; what every game has is engine code); read its README before adding one. Terms: **starter** = the engine's empty new project (`engine/starter/`), **template games** = finished games in `games/__templates__/` (read-only), **add-ons** = `engine/addons/`. Docs are in `__docs__/`.
- Target Bevy 0.18 and Dioxus 0.7.10 (`DX_VERSION` in `__ctrl__/lib/runtimes.py`) until there is a concrete reason to move.
- Authored content is `.rs` data in a game's `assets/` (see __docs__/engine.md#assets-everything-authored-is-a-rs-file): known assets are named `Name.Type.rs`: the one project settings file `<name>.project.rs` (title, start scene, preload bundles, input actions, platform options, icon), scenes (`*.scene.rs`), prefabs (`*.prefab.rs`), logic modules (`logic/*.rs`), and beside every external file its typed meta (`crate.png` + `crate.image.rs`, `hero.glb` + `hero.mesh.rs`, `theme.ogg` + `theme.audio.rs`, `intro.mp4` + `intro.video.rs`), which holds its import settings and bundle. The Project browser shows each asset once (metas hidden, like Unity). The naming rule lives in one place, `xerxes_build::types`. Never move or rename an external file without its meta.
- Art ships only as bundles (`public/bundles/<id>.zip`, built by `build.rs`), downloaded and extracted before use on every platform. Never ship loose art or `.rs` source.
- **The engine app is standalone, like Godot or Unity.** The exe and APK work on any device without the repo or the backend. The editor reaches projects only through its project store (`engine/src/editor/project/store.rs`), one interface with two adapters: natively the device's disk (inside the Xerxes repo: `games/` and `games/__templates__/`; anywhere else: a projects folder, `Documents/Xerxes Projects` or the app's storage on Android), on the web the local backend's engine services (a page cannot touch the disk).
- **One way to create a project:** the shared scaffold `xerxes_build::scaffold` (copy a template game, rename package, binary, APK, Android id, titles, README, guarantee the `.gitignore`, point the engine dependency: `../../engine` inside the repo, the engine's Git repository elsewhere). The editor (template games packed into it by `engine/build.rs`), the backend (`POST /engine/projects`, for the web editor) and `xerxes-ctrl game new` (binary `xerxes-new`) all call it. Never copy a template another way.
- Each game's `<name>.project.rs` names its backend (`backend: BackendSettings { dev, publish }`); the game reads the URL for its build from the `GameBackend` resource.
- Assets: GLB/glTF from Blender, normalized at export. No per-game rotation or scale hacks unless an asset truly needs them.

## Plan loop

1. Take the first eligible `pending` stage in `__plans__/PROGRESS.md`, or the stage the user named.
2. Mark it `in_progress`.
3. Implement it.
4. Run `__ctrl__\xerxes-ctrl.bat test all` (or `test engine` / `test backend` for the touched side).
5. Mark it `done` only when tests pass.
6. End the day green and leave a trail: `test all` passing, the **Next step** line at the top of PROGRESS.md rewritten, and a few dated lines added to `__plans__/LOG.md`. Prefer a thin working slice over a wide unfinished one (see `__plans__/EXECUTION.md`).

Do not gitignore `__plans__/`.

## Backend

Route (Axum handler) → Service → Repository (trait, SQLx) → PostgreSQL

| Layer | Responsibility |
|-------|----------------|
| Route | HTTP only: parse, call a service, shape the response |
| Service | Rules, authorization, cache, enqueue |
| Repository | Queries and writes only |
| Core | Config, database, cache, jobs, security, errors, docs pages |

- The backend is two series of modules in one process, built to be cut apart later: `backend/src/modules/api/` (HTTP API: `base`, `system`, `engine`, `editor`, one folder per module) and `backend/src/modules/multiplayer/` (rooms and realtime, the M track of Phase 3). They share only `core`: **neither imports the other** (`tests/backend/modules.rs` enforces it), and `SERVICES=all|api|multiplayer` picks what a process serves. Multiplayer is its own container later, not a rewrite.
- New API modules go in `backend/src/modules/api/<name>/` (router, service, repository, schemas), declared and merged in `api/mod.rs`.
- **API docs are generated, never written by hand.** Routers are `aide::axum::ApiRouter`s; register routes with `core::api::{get, post, put, patch, delete}` and take input through its typed extractors (`Body<T>`, `Form<T>`, `Query<T>`, `Path<T>`, `RawBody`, `CurrentUser`, `Superuser`) with `T: JsonSchema`. Swagger (`/docs`) and Scalar (`/sdoc`) read the generated document: summary, id and tag come from the handler's name and module, bodies, params and responses from its types, the bearer requirement from the auth extractor. Name handlers for their summary (`read_users` → "Read Users").
- Errors are `ApiError` in `backend/src/core/error.rs`. The response is always `{"detail": "..."}` with the status from [CONTRACT.md](../../CONTRACT.md).
- PostgreSQL is the system of record. New tables go in `backend/migrations/NNNN_name.sql` with `IF NOT EXISTS`. Migrations apply when the API starts.
- Redis cache helpers no-op when Redis is down. Do not cache auth. Jobs use the Redis list protocol in the contract.

## Definition of done

- Code in the right place (engine module vs game vs backend)
- Tests for behaviour that can fail: `tests/engine/`, `tests/backend/`, or the game's own tests
- `cargo fmt`; `test all` green
- Docs updated (`__docs__/engine.md` for engine changes); `__plans__/PROGRESS.md` updated when a stage was in progress
- Do not add a dependency without a reason
