# engine (`xerxes_engine`)

Dioxus hosts Bevy. A game implements `Scene` (a Bevy world and a Dioxus HUD over one bridge) and calls `xerxes_engine::launch::<MyScene>()`. This crate is both the library games depend on and the engine app.

```text
src/modules/        the library games use (Bevy only): assets (scenes, prefabs, metas), bundles,
                    flow (Preload → Splash → MainMenu → Level, game modes), project (settings,
                    input actions, launch_project), runtime (window), scene (hand-written games)
src/editor/         the engine app (feature `editor`, default): bridge (UI ↔ stage, no Bevy/Dioxus),
                    host (DOM / Blitz), project (store, codec, browser model), stage (Bevy Scene
                    view), ui (Dioxus panels), theme
src/main.rs         engine app on windows / macOS / Linux / web
platform/android/main.rs  engine app on Android (the cargo-apk `android` example)
index.html          web page template: loading splash while the wasm compiles
```

## What is in this folder

| Path | Role | Movable? |
|------|------|----------|
| `src/`, `Cargo.toml`, `build.rs` | The `xerxes_engine` crate: library and engine app | **No.** Every game depends on `../../engine` |
| `build/` | The games' build crate `xerxes_build`: the build step (`build.rs` of every game), shared asset rules and the project scaffold. Also used by the backend | **No.** A stable path: games, the scaffold and the backend point at `engine/build` |
| `starter/` | The built-in empty project every New Project starts from (files only; compiled into the scaffold, and a generated copy is what builds) | Yes (the scaffold and `backend/Dockerfile` name it) |
| `winit/` | winit 0.30.13 with one patch (see `winit/XERXES.md`) | **No.** Every game's `[patch.crates-io]` points at `engine/winit` |
| `addons/` | The add-ons: optional gameplay logic a game copies (camera and character controllers, HUD, pause; see its README). Compiled into `build/` | Layout inside is free, the folder stays (`build/` and `backend/Dockerfile` name it) |
| `platform/` | Per-platform entry and resources of the engine app: `android/` (`main.rs`, the cargo-apk example, and `res/` launcher icons), `windows/` (exe icon) | Yes (referenced by `Cargo.toml` and `build.rs`) |
| `index.html`, `Dioxus.toml` | The web page and `dx` config of the engine app | |

Never move or rename `build/` or `winit/`: games created before the move would stop building. A move needs a breaking release first, where games depend on `xerxes_build` from the engine's Git repository instead of a relative path.

```bat
__ctrl__\xerxes-ctrl.bat engine run web          REM live, hot reload: http://localhost:5100
__ctrl__\xerxes-ctrl.bat engine publish android  REM dist/engine/android/engine.apk
cargo test                                       REM in engine/: tests in ../tests/engine
```

Games depend on this crate with `default-features = false`, so the engine app (`editor`) isn't built into them. No gameplay lives here. Games are in [`../games/`](../games/), the architecture is in [`../__docs__/engine.md`](../__docs__/engine.md), and the pipeline is in [`../__docs__/games.md`](../__docs__/games.md).
