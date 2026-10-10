# Xerxes engine

The engine is extracted from small real games, not designed ahead of them. Something used once stays in its game. Something used twice is a pattern to watch. Something reused three times and clearly generic moves into the engine. This document describes only what exists.

## What Xerxes is

Bevy has no editor. Xerxes is the editor and the tools for making Bevy games, not a new engine on top of Bevy:

- **Bevy is the runtime** (rendering, audio, input, ECS). Games and templates use the Bevy API directly; Xerxes adds no wrapper API over it.
- **A few engine concepts Bevy does not have:** project settings, scenes, prefabs, metas, bundles, and named input actions (`engine/src/modules/`: `assets`, `flow`, `project`) and, from Phase 3, optional tools every game may want: `physics` (2D) and `matches` (the level's match: modes, scoring, rounds), both over the Bevy-free `xerxes_sim` crate in `engine/sim/`. Nothing else goes into the engine library.
- **Add-ons are how it grows:** ready-to-use `.rs` logic (camera controllers, character controllers, HUD, pause) in `engine/addons/`. Adding one to a game copies it into the game's `assets/`, where the game owns it and changes it freely. Anything a game proves worth keeping goes back into the templates.
- **The Dioxus editor** creates and edits a game's `.rs` files (scenes, prefabs, metas, settings) and runs the build pipeline.

## Three logic layers

```text
games/<name>/assets/         GAME      project settings, levels (scene + game mode), prefabs, configured logic copies
engine/addons/               ADD-ONS   optional logic over engine + Bevy: controllers, HUD, pause (later inventory, guns, melee)
engine/src/modules/          ENGINE    game flow (splash, main menu or a menu scene, levels), game modes, scenes, prefabs, metas, bundles, input actions, 2D physics (xerxes_sim)
bevy                         RUNTIME   rendering, lights, meshes, cameras, animation, audio, UI, ECS
```

- **Engine:** solid, general game-making concepts in Rust, plus Bevy used directly for everything it already does. Normal users never change it.
- **Templates:** starter logic that uses the engine and Bevy, copied into a game and configured there. Normal users never change the originals.
- **Game:** what a user makes. Its own project settings, levels, prefabs and its copies of templates.

### Game flow (engine)

Every game runs `Preload → Splash → MainMenu → Level #`. Project settings declare the splash, the main menu and the ordered levels. Each level is a scene plus its **game mode** ([game-modes.md](game-modes.md): sides × win condition, outside-grid kinds, modifiers). Templates and games move the flow with messages (`LoadLevel`, `LevelComplete`, `ExitToMenu`); the engine owns the transitions.

## Where things live

| Part | Path |
|------|------|
| Engine library | `engine/`: crate `xerxes_engine`, modules in `engine/src/modules/`. Bevy only |
| Engine app (editor) | `engine/src/main.rs` + `engine/src/editor/` (bin `xerxes-engine`, feature `editor` = Bevy rendering + Dioxus, on by default): the Dioxus editor UI over a Bevy stage, where game creation tools will grow |
| Games | `games/<name>/`: each its own crate, depending on `../../engine` with `default-features = false`, with its own `assets/`. `game new` games are their own Git repositories; the Darius games live in the engine repository |
| Templates | `games/__templates__/<name>/`: finished games with clear names, committed with the engine. New projects start from the engine's built-in `starter` (`engine/starter/`), not from a template |
| Multiplayer (track M, Phase 3) | `backend/`: authoritative rooms and state sync next to the Axum API, and `xerxes_sim` shared with the game. Nothing yet ([multiplayer.md](multiplayer.md)) |
| Engine tests | `tests/engine/` |

## Every target, every platform

Cross-platform is a hard rule. The engine app and every game build from one source and go through the same pipeline (`build`, `dev`, `publish`) on every platform:

| Target | Made of | web | windows (macOS, Linux) | android | ios |
|--------|---------|-----|------------------------|---------|-----|
| Engine app (editor) | Dioxus + Bevy + engine | dev + publish | dev + publish | dev + publish | later (needs macOS) |
| Games and templates | Bevy + engine (no Dioxus) | dev + publish | dev + publish | dev + publish | later (needs macOS) |

A platform difference is allowed only in renderer or platform glue (`editor/host/web.rs`, `editor/host/native.rs`, the Android entry). Never in components, assets, scenes or pipelines.

## Assets: everything authored is a `.rs` file

The editor scaffolds and edits typed Rust data, like Unity's assets. The game compiles that data as normal Rust, and the editor parses and regenerates it. Known assets are named **`Name.Type.rs`**, so the type is in the name.

```text
games/<name>/
  assets/                     authored (the editor's Asset Browser; Unity's "Assets")
    my-game.project.rs        ProjectSettings (exactly one): title, start scene, preload bundles, input manager
                              (named actions → keys/buttons/axes), platform options (window, web, android), icon
    scenes/main.scene.rs      SceneAsset: the objects in the scene
    prefabs/coin.prefab.rs    PrefabAsset: objects, nested prefabs, overridden config
    logic/mover.rs            a logic module: a component + its systems (`pub fn plugin(app)`)
    art/crate.png             an external file (image, mesh, audio, video)
    art/crate.image.rs        its meta: import settings, bundle id (`Name.<type>.rs`)
  public/                     generated by build.rs (gitignored): what ships
  build.rs                    xerxes_build::generate()
```

- **ECS, component-driven.** An object is an entity with components. Add an empty object, attach logic modules (components whose systems the game registers), and it runs. Built-in components cover transform, mesh, material, sprite, camera and light.
- **Typed names.** `Name.Type.rs` with `Type` one of `project`, `scene`, `prefab`, and (metas of external files) `image`, `mesh`, `audio`, `video`. The rule lives in `xerxes_build::types`, shared by the build step, the backend and the editor.

  | File | Type | Meta of |
  |------|------|---------|
  | `my-game.project.rs` | project settings | |
  | `main.scene.rs`, `coin.prefab.rs` | scene, prefab | |
  | `crate.image.rs` | image config | `crate.png`, `.jpg`, `.webp`, ... |
  | `hero.mesh.rs` | mesh config | `hero.glb`, `.gltf` |
  | `theme.audio.rs` | audio config | `theme.ogg`, `.wav`, `.mp3`, `.flac` |
  | `intro.video.rs` | video config | `intro.mp4`, `.webm` |

  `logic/*.rs` stay plain modules. An external file of another type (a font) needs no meta and ships in `core`.
- **One entry per asset.** The Project browser shows `crate.png` once, with its type (Image) and a small preview (an image's thumbnail, a scene's 2D/3D, a type badge); `crate.image.rs` is hidden like Unity's `.meta`. Renaming, moving or deleting the file moves or deletes its meta (the editor and the backend both do it). Two files of one type and name (`crate.png`, `crate.jpg`) would share a meta, which the build refuses.
- **Project settings** (`<name>.project.rs`) hold the game's top-level config, like Unity's or Unreal's Project Settings. Gameplay reads input through named actions (`actions.pressed("jump")`), never raw keys, so remapping is one place.
- **Scenes are 2D or 3D, games are not.** A game can mix both (a 2D menu level, a 3D world). Each scene declares `dimension: Dimension::D2 | Dimension::D3` in its `.rs`; every game builds both renderers (the `2d` and `3d` Bevy collections). The editor shows a 2D scene flat (orthographic, XY, pixels) and a 3D scene in perspective.
- **Prefabs** contain objects and other prefabs. An instance in a scene stores only its overridden components (a component of the same type replaces the prefab's).
- **Scenes** list their objects (plain objects or prefab instances with overrides).
- `build.rs` turns `assets/` into Rust modules plus a manifest, and packs the shipped files. It runs inside every build (cargo, dx, cargo-apk), so every platform gets the same result.

## Add-ons (`engine/addons/`)

Add-ons are **conditional game logic**: things a game may or may not have. It is compiled into the `xerxes_build` crate, so the editor, the backend (`GET /engine/addons`, `POST /engine/projects/{name}/addons`) and the CLI offer exactly the same set, with or without the repo. What every game has (scenes, prefabs, bundles, flow, input, project settings) is the engine, in `engine/src/modules/`. One folder per feature (rules in `engine/addons/README.md`); adding one copies it into the project's `logic/`, with what it needs, never overwriting a file the game already has:

| Logic | What it does | Needs |
|----------|--------------|-------|
| `game/pause.rs` | `Paused` resource toggled by the `pause` action | |
| `hud/hud.rs` | bevy_ui panel: title, hint, Pause/Resume, PAUSED banner | pause |
| `chr-ctrl/character_controller_3d.rs` / `_2d.rs` | Move with `move_*` actions, `respawn` action | pause |
| `camera-ctrl/camera_controller_3d.rs` / `_2d.rs` | Follow the object marked `CameraTarget3d` / `CameraTarget2d` | |

- The editor's starter scenes (`2d.scene.rs`, `3d.scene.rs`, for New Scene) are in `engine/src/editor/project/starters/`. `BundleTexture` (texture an object from an on-demand bundle) is part of the engine (`modules/bundles/texture.rs`), installed by `launch_project`.
- A module's first doc line says what it does; `//! needs: <name>` names the logic it uses by file name (added together).

## Bundles: art ships as zips, downloaded before use

- Each external file's meta (`Name.image|mesh|audio|video.rs`) names a **bundle** (`.bundle("core")`, default `core`). Files with the same id are zipped together at build time into `public/bundles/<id>.zip`. No loose art and no `.rs` source ship.
- At runtime a bundle is fetched through Bevy's `AssetServer` (HTTP on the web, a file on desktop, the APK on Android) and **extracted in memory** into the `bundle://` asset source. Scenes refer to assets by their authored path (`art/crate.png`), and the engine resolves it inside the bundles.
- The project's preload bundles are downloaded and extracted behind a loading screen **before the first scene spawns**, the same on every platform. Other bundles load on demand (later levels, DLC).

## Edit mode and Play

`.rs` assets are compiled into the game, so the editor never runs game code itself (no dylibs on web or Android):

- **Edit mode:** the editor parses the scene and shows its built-in components in its own Bevy viewport. Logic modules appear in the Inspector with their fields.
- **Play / Build / Publish:** the editor asks the local backend to run the normal pipeline step (`game dev|build|publish <name> <platform>`), on machines that have the toolchain.

## Projects: a standalone editor

The engine app (exe, APK) is standalone, like Godot or Unity: it runs on any device without the repo or the backend.

- **Project store** (`engine/src/editor/project/store.rs`): the editor's one interface for listing, creating, reading and writing projects. Natively it is the device's disk; on the web (a page cannot touch the disk) it is the local backend's engine services.
- **Where projects are (native):** inside the Xerxes repo (found from the working directory or the exe, or `XERXES_ROOT`), the repo's `games/` and `games/__templates__/`. Anywhere else, a projects folder: `XERXES_PROJECTS`, else `Documents/Xerxes Projects`, or the app's private storage on Android.
- **New Project** uses the template games packed into the editor binary (`engine/build.rs` → `templates.zip`) and the shared scaffold `xerxes_build::scaffold`, the same code as the backend (`POST /engine/projects`, web editor) and `xerxes-ctrl game new` (binary `xerxes-new`). Inside the repo a new game depends on `../../engine`; elsewhere on the engine's Git repository (`branch = "main"`), so it builds on any machine with Rust. Every new game gets a `.gitignore` with `/target/`, `/dist/`, `/public/` and, on desktop, its own Git repository.
- **New Scene** (Project panel) asks for a name and 2D or 3D and starts from `engine/src/editor/project/starters/{2d,3d}.scene.rs`. The Hierarchy's 2D/3D switch changes an open scene's `dimension`.
- **Game backend:** the `<name>.project.rs` has `backend: BackendSettings { dev, publish }` (dev: `http://127.0.0.1:8000`, which a phone reaches through `adb reverse`; publish: the deployed URL, empty for offline). The game reads the URL for its build from the `GameBackend` resource.

## Two UIs, one per side

| | UI | Ships in |
|-|----|----------|
| Engine editor | **Dioxus** (`engine/src/editor/`) | the engine app only |
| Games | **bevy_ui**, built in `Scene::build` like any other system | the game and its exports |

A game never depends on Dioxus. Its native and web dependency trees contain no Dioxus or Blitz crates (`cargo tree` in a game shows none).

## Games: Bevy only

```text
game     main.rs, android.rs → game::run() → xerxes_engine::launch::<MainScene>()
engine   scene::launch → Bevy App (DefaultPlugins + primary_window + Scene::build) → run
         windows / android: a Bevy window · web: a Bevy canvas
```

| Module | Holds |
|--------|-------|
| `runtime` | `primary_window`: one window setup for every platform (on the web, an optional host canvas selector) |
| `scene` | `Scene` trait, `app::<S>()` and `launch::<S>()` |

```rust
impl Scene for MainScene {
    const TITLE: &'static str = "My Game";
    fn build(app: &mut App) { app.add_plugins((world::MainScenePlugin, hud::HudPlugin)); }
}
```

The reference game splits a scene into `world.rs` (systems, the state the HUD reads, and an `Action` message the HUD writes) and `hud.rs` (bevy_ui nodes, `Button` + `Interaction` → `Action`).

## The editor: one source, every platform

The editor's UI is one set of Dioxus components (`editor/panel.rs`, `editor/catalog.rs`) inside one host page (`editor/host/mod.rs`: one stylesheet, a stage layer, a HUD layer). Only the renderer differs per platform:

```text
                    host::page(viewport, Root(UiPort))   ⇄  bridge<EditorCommand, EditorSnapshot>  ⇄  Bevy stage
web                 Dioxus web: the DOM; viewport = Bevy's <canvas>
windows / android   Dioxus native's DOM (DioxusDocument), laid out by Blitz, painted on the CPU (vello_cpu)
                    into an image a full-window bevy_ui node shows over the world; viewport = empty
```

| Module | Holds |
|--------|-------|
| `editor/bridge` | UI → runtime command queue; runtime → UI snapshot (latest + push). No Bevy, no Dioxus. Holds the editor's `protocol` and `view` too |
| `editor/stage/runtime` | Bevy end: `UiBridgePlugin`, `UiBridge` (resource), `UiCommand<C>` (message) (re-exported as `editor::runtime`) |
| `editor/host` | `launch`, the shared host `page`, `use_runtime_snapshot`; `web` (Dioxus web, `GameViewport`, `run_once`), `native` (`DioxusOverlayPlugin`) |
| `editor/stage`, `panel`, `catalog` | the Bevy stage, the Dioxus panel, the game catalog |

Rules:

- One source of truth. The panel and the host page are the same code on every target; per-platform code is limited to the renderer glue in `host/web.rs` and `host/native.rs` and the catalog's HTTP call. Never fork a component or a stylesheet per platform.
- Spell out what the UI relies on instead of leaving it to renderer defaults: Blitz is not a browser. Blitz 0.2 does not implement `text-transform`, so capitalised labels are written in capitals. Check new CSS natively before relying on it.
- `bridge` depends on neither Bevy nor Dioxus. The UI never reaches into the ECS and the world never touches the UI. `publish` drops a snapshot equal to the previous one, so the stage can publish every frame. Keep snapshots to what the UI shows.
- The HUD layer ignores the pointer except on buttons, links, inputs and `[data-interactive]` (web), so the world keeps the mouse.

### Web

Bevy and Dioxus share one thread and one wasm module. `host::launch` starts Dioxus; `GameViewport` mounts and focuses the canvas, then Bevy's `App::run()` hands its loop to the browser and returns. Bevy's `LogPlugin` is disabled there because Dioxus already installs the global logger. Bevy can start only once per page, so `run_once` guards it.

### Windows, macOS, Linux, Android

Bevy owns the only window and event loop (NativeActivity allows just one), and Dioxus runs inside the Bevy app:

- `DioxusOverlayPlugin` keeps a `DioxusDocument` as a main-thread (`NonSend`) resource and polls its VirtualDom every frame (bridge snapshots, clicks, finished fetches).
- Mouse, wheel and touch (first finger = left button) are forwarded to the document. Keys go to the document while one of its inputs has focus (`UiFocus`); otherwise they belong to the Scene view.
- When something changed (a poll did work, a click, hover crossed elements, a resize), Blitz resolves style and layout and paints into a premultiplied buffer that is unpremultiplied into an sRGB image. It repaints nothing otherwise.
- Links open in the system browser (Windows, macOS, Linux); not wired on Android yet.
- Projects are on the device's disk (the project store), so the native editor needs no backend.
- Android: a UI text input with focus opens the soft keyboard (`Window::ime_enabled`). The app runs fullscreen (no status bar over the toolbar) and handles every configuration change in-app (`config_changes`): NativeActivity cannot be recreated in the same process (winit: "RecreationAttempt"), so a mouse or keyboard plugged in mid-run would otherwise crash or hang it.
- Android logs: `game dev <x> android` and `engine run android` follow only the app's own output (Rust log, panics, crashes) after clearing the old device log; `XERXES_ALL_LOGS=1` shows the whole device log.
- Android launcher icons: `res/mipmap-*/ic_launcher.png` in each game (`resources = "res"`; the engine app keeps its own in `engine/platform/android/res`), `icon = "@mipmap/ic_launcher"`); new games inherit the template's.
- Windows icon: `engine/platform/windows/xerxes.ico` is compiled into the editor exe (`platform/windows/xerxes.rc`, `embed-resource` in `build.rs`), so Explorer shows it; the window and taskbar take the same resource through winit.

## Scene view input

| Input | Action |
|-------|--------|
| W / E / R | move / rotate / scale tool (anywhere except while typing in a field) |
| X | gizmo axes: Global (world) or Local (the object's); scale always uses the object's axes |
| RMB + mouse, RMB + W/A/S/D/Q/E | look around, fly |
| MMB, Alt + LMB / RMB / MMB, wheel | pan; orbit / dolly / pan; dolly |
| F | frame the selection |
| tap | select |
| one-finger drag | orbit (3D), pan (2D); on a gizmo handle it drags the handle |
| two fingers | pan; pinch to zoom |

Touch works on every platform (`editor/stage/touch.rs`): the first finger is the pointer.
Gizmo handles have a least grab radius in screen pixels (larger for a finger), so they stay
easy to hit on a phone.

A mouse on Android works like on a desktop (RMB look + WASD fly, MMB pan, wheel, Alt combos,
hover). Upstream winit 0.30 turns an Android mouse into touches and drops its buttons, so the
engine patches winit's Android input handler: `engine/winit` (winit 0.30.13, changes
marked "Xerxes patch"), used through `[patch.crates-io]` by the engine and every game.

Panel sizes follow the window (`theme::LEFT`, `RIGHT`, `BOTTOM`: a share of the usable window,
kept between a minimum and a maximum). The stage computes the frame each frame (`theme::Layout`:
the system's insets, such as Android's navigation bar or a cutout, and the panel sizes), places the
Scene view with it and sends it to the UI in the snapshot, which sizes the panels with the same
numbers: they always line up, on every platform. On phones the native editor also uses a denser
scale when the window is narrower than 1100 logical pixels (`phone_density`), and the app follows
the phone in either landscape direction (`sensorLandscape`).

The native UI is painted on the CPU (vello_cpu, multithreaded) only when it changes. Dependencies
build without debug assertions even in dev (`[profile.dev.package."*"]`): with them one repaint
took ~0.8 s instead of ~15 ms.

## Web builds ship with a Dockerfile

Every web build gets a `Dockerfile` (nginx) beside the site, so it can run on another server:

| Command | Site + Dockerfile |
|---------|-------------------|
| `game build <x> web` (dev) | `dist/<x>/web-dev/` |
| `game publish <x> web` | `dist/<x>/web/` |
| `engine build web` / `engine publish web` | `dist/engine/web-dev/` / `dist/engine/web/` (proxies `/api` to `XERXES_API`, default `http://host.docker.internal:8000`) |

```bat
docker build -t my-game-web dist\my-game\web
docker run -p 8080:80 my-game-web
```

nginx serves the wasm with its MIME type, gzips wasm/js/zip, and falls back to `index.html` for unknown paths.

## Commands

The dev and publish pipeline (index, `game dev`, `game publish`, `game new`, templates) is in [games.md](games.md).

Directly, inside `engine/` or a game folder (set `CARGO_TARGET_DIR` to the engine's `target` to share builds):

```bat
cargo run
dx serve --platform web --port 5100
```

## Build notes

- Games optimize dependencies in dev (`[profile.dev.package."*"] opt-level = 3`) because Bevy is unusably slow without it. Expect a long first build. The editor's native build also compiles Blitz and Stylo once.
- `wasm-dev` builds carry no debug info. With it, a Bevy dev wasm is over 1 GB and wasm-bindgen runs out of memory.
- The engine library takes the Bevy features its own code needs (window, log, assets, UI, state, and the scene defs: mesh, pbr, sprite). Rendering and the rest arrive through the game's features (the `2d` and `3d` collections, with `ui_api`/`ui_bevy_render`), because Cargo unifies features. Games depend on the engine with `default-features = false` so they don't pull in the editor (Dioxus). A game's `web` feature turns on `xerxes_engine/web` (Bevy's web support only).
- Editor native renderer versions follow `dioxus-native` 0.7.10: `dioxus-native-dom` 0.7.10, `blitz-dom` 0.2.4, `blitz-paint` 0.2.1, `anyrender` 0.6, `anyrender_vello_cpu` 0.8. Painting is on the CPU so Blitz's wgpu version never has to match Bevy's.
