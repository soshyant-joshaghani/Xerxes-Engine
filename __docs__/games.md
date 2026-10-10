# Engine and games: build, run, publish

Every target (the engine app and every game) builds from one source and goes through the same steps on each platform. Games are Bevy only, with a bevy_ui HUD. The engine app adds the Dioxus editor UI, the same components on every platform ([engine.md](engine.md#the-editor-one-source-every-platform)):

| Platform | What runs | Game UI (bevy_ui) | Engine editor UI (Dioxus) |
|----------|-----------|-------------------|---------------------------|
| `web` | One page with Bevy's canvas | yes | Dioxus renders the page (DOM) |
| `windows` (`win`) | A Bevy window (macOS and Linux behave the same) | yes | Blitz paints it over the Bevy window |
| `android` | A NativeActivity APK, built with cargo-apk (no Gradle) | yes | Blitz paints it over the Bevy window, touch forwarded |
| `ios` | Needs macOS with Xcode. Refused on other hosts; not wired on macOS yet | — | — |

| Step | What it does |
|------|--------------|
| `build` | Compiles the dev build, so the next run only links and starts |
| `run` / `dev` | Live: hot reload on the web, `cargo run` on windows, install and start on Android |
| `run --built` | Runs the published build from `dist/` (engine) |
| `publish` | Release build into `dist/<name>/<platform>/` |

## Where they live

| Path | What | Git |
|------|------|-----|
| `engine/` | The engine app (`xerxes-engine`), where games will be created, plus the `xerxes_engine` library | engine repo |
| `games/<name>/` | Games made with the engine, finished or in progress | `game new` games: each its own repo, ignored by the engine repo. `darius` and `cyrus`: committed with the engine (`games/.gitignore`) |
| `games/__templates__/<name>/` | Finished games kept as templates, with clear names (`darius`). `game new` starts from the engine's built-in `starter`, not from one of these | committed with the engine |
| `dist/<name>/<platform>/` | Release builds (`dist/engine/...` for the engine app) | ignored |

A game of your own lives in `games/` (its own repo) and moves to `games/__templates__/` once it is finished. The Darius games are developed in `games/__templates__/` from the start (run with `template dev darius`).

## The engine app

```bat
__ctrl__\xerxes-ctrl.bat engine build all            REM web + windows + android dev builds
__ctrl__\xerxes-ctrl.bat engine run web              REM live, hot reload: http://localhost:5100 (http://engine.localhost with infra up)
__ctrl__\xerxes-ctrl.bat engine run win              REM Bevy window
__ctrl__\xerxes-ctrl.bat engine run android          REM APK on the adb device
__ctrl__\xerxes-ctrl.bat engine publish web          REM dist/engine/web/
__ctrl__\xerxes-ctrl.bat engine publish android      REM dist/engine/android/engine.apk
__ctrl__\xerxes-ctrl.bat engine run web --built      REM serve dist/engine/web on :5100 (API proxied to :8000)
```

The engine's web panel lists the games (from the backend, so run `backend run dev` first). A game with a published web build gets a **Play** button that opens `http://<name>.play.localhost`, where the backend serves `dist/<name>/web/public`. Games without a build show the `game publish` command.

## Games and templates

```text
> xerxes-ctrl game list            > xerxes-ctrl template list
   (none)                              1  darius          template
```

Games are indexed from 1 in alphabetical order; templates have their own index. Every command accepts an index or a name; if you leave it out, the CLI asks.

```bat
__ctrl__\xerxes-ctrl.bat game build 1 all                REM compile web + windows + android
__ctrl__\xerxes-ctrl.bat template dev darius                REM web, hot reload, http://game.localhost (:5200)
__ctrl__\xerxes-ctrl.bat template dev darius win            REM Bevy window
__ctrl__\xerxes-ctrl.bat template dev darius android        REM build, install and start on the adb device
__ctrl__\xerxes-ctrl.bat template run 1 android            REM `run` = `dev`, by index or name (also `game run`; --built = published build)
__ctrl__\xerxes-ctrl.bat game publish 1                  REM web build into dist/darius/web/ (Play in the engine)
__ctrl__\xerxes-ctrl.bat template publish darius all        REM web + windows + android
__ctrl__\xerxes-ctrl.bat game new my-game                REM the engine's built-in starter (one scene) into games/my-game, git init
__ctrl__\xerxes-ctrl.bat game new my-game --from <template>
__ctrl__\xerxes-ctrl.bat template dev <name>             REM play a finished template game
__ctrl__\xerxes-ctrl.bat cleanup                         REM delete every target/ and dist/ (asks first; --dry-run lists sizes)
```

| Step | web | windows | android |
|------|-----|---------|---------|
| build | `dx build --platform web` | `cargo build` | `cargo apk build --example android --features android` |
| dev / run | `dx serve --platform web --port 5100` | `cargo run` | `cargo apk run --example android --features android --target <device ABI>` |
| publish | `dx bundle --platform web --release` → `dist/<name>/web/` (static site) | `cargo build --release` → `dist/<name>/windows/` (executable + `assets/`) | `cargo apk build --release --example android --features android` → `dist/<name>/android/<name>.apk` |

Android needs the SDK (`ANDROID_HOME`), the NDK and a JDK. The CLI finds the NDK under the SDK when `ANDROID_NDK_ROOT` isn't set, installs `cargo-apk` on first use, and picks the Rust target from the device `adb` sees (phone: arm64, emulator: x86_64; arm64 when nothing is connected). Release APKs are signed with the debug key until you add a release keystore (`[package.metadata.android.signing.release]` in the target's Cargo.toml).

## How a target is laid out

```text
src/lib.rs       the game (`[lib] name = "game"`): scenes + `pub fn run()`
src/main.rs      windows / macOS / Linux / web: `game::run()`
src/android.rs   Android: `#[bevy_main] fn main() { game::run() }`, the `android` example (cdylib)
Cargo.toml       Bevy features spelled out (the `2d` and `3d` collections: scenes are 2D or 3D, not games; with `android-native-activity`),
                 `[package.metadata.android]` for cargo-apk
Dioxus.toml      app name and page title for the web
```

The engine app follows the same layout: `engine/src/main.rs` and `engine/platform/android/main.rs` both launch `xerxes_engine::editor::EditorScene`.

`game new <name>` runs the shared scaffold (`xerxes_build::scaffold`, binary `xerxes-new`; the editor's New Project and the backend run the same code): it copies a template and sets the Cargo package, bin and APK names, the Android package id and label, the Dioxus app name and title, the scene title and the engine path (`../../engine`), writes a README for the game (keeping the template's layout section) and a `.gitignore` (`/target/`, `/dist/`, `/public/`), then runs `git init`. `engine` and template names are reserved, because `dist/<name>/` and `<name>.play.localhost` are shared.

All builds share `engine/target` (`CARGO_TARGET_DIR`), so each Bevy feature set and platform compiles once. `CARGO_BUILD_JOBS` defaults to half the cores, because Bevy at full parallelism can exhaust memory (on Windows: "paging file is too small"). `cleanup` frees all of it.

## Tests

```bat
__ctrl__\xerxes-ctrl.bat test engine       REM engine lib + app tests, engine web check
__ctrl__\xerxes-ctrl.bat test templates    REM each template: cargo test + wasm32 web check
__ctrl__\xerxes-ctrl.bat test games        REM each game in games/: cargo test + wasm32 web check
```

`test all` runs the backend, engine, templates and games.
