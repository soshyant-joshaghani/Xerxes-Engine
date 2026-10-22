# Xerxes engine

The engine is extracted from small real games, not designed ahead of them. Something used once stays in its game. Something used twice is a pattern to watch. Something reused three times and clearly generic moves into the engine. This document describes only what exists.

## What Xerxes is

Bevy has no editor. Xerxes is the editor and the tools for making Bevy games, not a new engine on top of Bevy:

- **Bevy is the runtime** (rendering, audio, input, ECS). Games and templates use the Bevy API directly; Xerxes adds no wrapper API over it.
- **A few engine concepts Bevy does not have:** project settings, scenes, prefabs, metas, bundles, and named input actions (`engine/src/modules/`: `assets`, `flow`, `project`) and, from Phase 3, optional tools every game may want: `physics` (2D and 3D) and `matches` (the level's match: modes, scoring, rounds) and `participants` (who plays each seat: local, bot, remote, spectator), over the Bevy-free `xerxes_sim` crate in `engine/sim/`. Nothing else goes into the engine library.
- **Add-ons are how it grows:** ready-to-use `.rs` logic (camera controllers, character controllers, HUD, pause) in `engine/addons/`. Adding one to a game copies it into the game's `assets/`, where the game owns it and changes it freely. Anything a game proves worth keeping goes back into the templates.
- **The Dioxus editor** creates and edits a game's `.rs` files (scenes, prefabs, metas, settings) and runs the build pipeline.

## Three logic layers

```text
games/<name>/assets/         GAME      project settings, levels (scene + game mode), prefabs, configured logic copies
engine/addons/               ADD-ONS   optional logic over engine + Bevy: controllers, HUD, pause (later inventory, guns, melee)
engine/src/modules/          ENGINE    game flow (splash, main menu or a menu scene, levels), game modes, scenes, prefabs, metas, bundles, input actions, physics (2D and 3D, over xerxes_sim)
bevy                         RUNTIME   rendering, lights, meshes, cameras, animation, audio, UI, ECS
```

- **Engine:** solid, general game-making concepts in Rust, plus Bevy used directly for everything it already does. Normal users never change it.
- **Templates:** starter logic that uses the engine and Bevy, copied into a game and configured there. Normal users never change the originals.
- **Game:** what a user makes. Its own project settings, levels, prefabs and its copies of templates.

### Game flow (engine)

Every game runs `Preload → Splash → MainMenu → Level #`. Project settings declare the splash, the main menu and the ordered levels. Each level is a scene plus its **game mode** ([game-modes.md](game-modes.md): sides × win condition, outside-grid kinds, modifiers). Templates and games move the flow with messages (`LoadLevel`, `LevelComplete`, `ExitToMenu`); the engine owns the transitions. A scene sends them with buttons written as data: `LevelButton(n)` starts level `n` (a menu scene's tiles) and `ExitToMenuButton` goes back to the menu (a level's Back button); both act in the frame they are pressed.

## Where things live

| Part | Path |
|------|------|
| Engine library | `engine/`: crate `xerxes_engine`, modules in `engine/src/modules/`. Bevy only |
| Engine app (editor) | `engine/src/main.rs` + `engine/src/editor/` (bin `xerxes-engine`, feature `editor` = Bevy rendering + Dioxus, on by default): the Dioxus editor UI over a Bevy stage, where game creation tools will grow |
| Games | `games/<name>/`: each its own crate, depending on `../../engine` with `default-features = false`, with its own `assets/`. `game new` games are their own Git repositories; the Darius games live in the engine repository |
| Templates | `games/__templates__/<name>/`: finished games with clear names, committed with the engine. New projects start from the engine's built-in `starter` (`engine/starter/`), not from a template |
| Multiplayer (track M, Phase 3) | `backend/.../multiplayer/`: the **room runtime** only. Game rules stay in the game's `rules/` crate (`games/<name>/rules`); `xerxes_sim` is shared tools. See [backend-multiplayer.md](backend-multiplayer.md) |
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
    scenes/games/solo/...     scenes in folders, as deep as the game wants
    prefabs/header.prefab.rs  PrefabAsset: objects, nested prefabs, overridden config
    prefabs/games/solo/...    one game's own prefabs, mirroring its scenes
    src/logic.rs              the logic plugins: `pub fn mover(app)` per module, listed in LOGIC
    art/crate.png             an external file (image, mesh, audio, video)
    art/crate.image.rs        its meta: import settings, bundle id (`Name.<type>.rs`)
  public/                     generated by build.rs (gitignored): what ships
  build.rs                    xerxes_build::generate()
```

- **ECS, component-driven.** An object is an entity with components. Add an empty object, attach logic modules (components whose systems the game registers), and it runs. Built-in components cover transform, mesh, material, sprite, camera and light.
- **A level starts whole.** The flow is `Preload → Splash → MainMenu → Loading → Level`: the `Loading` step reads what the next scene's objects load (`scene_needs`, which walks prefabs too), fetches the bundles those files ship in, waits for the asset server behind a progress bar, and only then spawns the scene — so a heavy model never pops into a level that has already begun. It passes straight through when everything is in memory already (a second visit, a prefetch, a scene with no art). `PrefetchLevel(n)` fetches a level ahead of time; the menu does it for the button under the pointer. The screen says which step the wait is on (`Downloading … 7.5 MB`, then `Preparing … 0 / 1`) and the log says how long it took, so a slow level is visible without a profiler. Keep `preload_bundles` to what the menu itself needs and give a scene's heavy art its own bundle: that is what makes a web build download only what is played ([web-progressive.md](web-progressive.md)).
- **A level starts whole, and then waits a beat.** A level opens with the player looking at it for the first time: where they stand, what is coming at them, which way is forward. A game that starts the instant the scene spawns takes that away, and where there is no second chance — a runner meeting its first obstacle — it takes the run with it. So the flow gives every level a beat (`LevelReady`, 3 s by default, `app.insert_resource(LevelReady::of(…))` to change it, `LevelReady::of(0.0)` for a level that should just start), and a game's own systems go in the engine's **`Play`** set:

  ```rust
  app.add_systems(Update, (steer, step, collide).in_set(Play));
  ```

  The engine runs `Play` (in `Update` and in `FixedUpdate`) while a level is on **and** the beat is over, so a game gets both gates from one name and never writes `run_if(in_state(Flow::Level))` for its play again. What must keep running *during* the beat — building the level's world, the HUD, a countdown, a restart button — simply stays out of the set: that is the whole distinction it draws, advancing the game against showing it. A game that restarts a run inside the level (Siavash's R and its Restart button) calls `LevelReady::restart` so the retry gets the beat too, and reads `LevelReady::left()` to count it down on screen.

  The beat is counted in **frames the player could watch**, not in wall-clock seconds: a level's first frames are the slowest it will ever have, and at most `LevelReady::MOST_PER_FRAME` (0.1 s) of the beat is spent per frame, so one two-second frame cannot swallow it.

- **Art is a component too.** `MeshDef` makes a primitive (cube, cuboid, sphere, plane), `SpriteDef` a 2D sprite, and **`ModelDef` loads a `.glb` / `.gltf` from its bundle** and spawns the whole glTF scene (meshes, materials, textures) under the object. A glTF model faces **+Z** with +Y up, so an object with no rotation of its own faces the way a game's forward axis points; keep a game's forward axis +Z and nothing has to be turned around. The Scene view stands a box where a model is until the editor loads the model itself.
- **Typed names.** `Name.Type.rs` with `Type` one of `project`, `scene`, `prefab`, and (metas of external files) `image`, `mesh`, `audio`, `video`. The rule lives in `xerxes_build::types`, shared by the build step, the backend and the editor.

  | File | Type | Meta of |
  |------|------|---------|
  | `my-game.project.rs` | project settings | |
  | `main.scene.rs`, `coin.prefab.rs` | scene, prefab | |
  | `crate.image.rs` | image config | `crate.png`, `.jpg`, `.webp`, ... |
  | `hero.mesh.rs` | mesh config | `hero.glb`, `.gltf` |
  | `theme.audio.rs` | audio config | `theme.ogg`, `.wav`, `.mp3`, `.flac` |
  | `intro.video.rs` | video config | `intro.mp4`, `.webm` |

  Logic is plain Rust in `src/`; the project file's `LOGIC` lists each module's `plugin`. An external file of another type (a font) needs no meta and ships in `core`.
- **One entry per asset.** The Project browser shows `crate.png` once, with its type (Image) and a small preview (an image's thumbnail, a scene's 2D/3D, a type badge); `crate.image.rs` is hidden like Unity's `.meta`. Renaming, moving or deleting the file moves or deletes its meta (the editor and the backend both do it). Two files of one type and name (`crate.png`, `crate.jpg`) would share a meta, which the build refuses.
- **A file's constants are part of its document.** The `const PANEL: Color = ..;` items of a scene or a prefab are the palette and the sizes it names; the editor reads them (a colour written `BackgroundColor(PANEL)` resolves through them), shows them in the Properties with nothing selected, and writes them back on Save — add, rename (every use in the document follows), change and remove. They stay plain Rust: what is typed is what the file holds. A `pub const` is the file's interface (the project file's `LOGIC`), not its palette, and is never touched.
- **Project settings** (`<name>.project.rs`) hold the game's top-level config, like Unity's or Unreal's Project Settings. Gameplay reads input through named actions (`actions.vector("move")`, `actions.just_pressed("jump")`), never raw keys: the project file's `input` map binds each action to the keyboard, mouse, gamepad and on-screen touch controls per game, on every platform (see [input](input.md)).
- **Scenes are 2D or 3D, games are not.** A game can mix both (a 2D menu level, a 3D world). Each scene declares `dimension: Dimension::D2 | Dimension::D3` in its `.rs`; every game builds both renderers (the `2d` and `3d` Bevy collections). The editor shows a 2D scene flat (orthographic, XY, pixels) and a 3D scene in perspective.
- **Scenes** list their objects (plain objects or prefab instances with overrides).
- **Prefabs are the modules a scene is built from.** A prefab (`*.prefab.rs`) is one object tree, and it may hold instances of other prefabs. The rules:

  - **An instance replaces components by type and appends children.** `Object::prefab(p).named("Pause Button").with(HudButton(Action::TogglePause))` takes the prefab's frame and puts its own action on it; `.child(..)` adds to what the prefab already holds (the prefab's own children come first). An instance **cannot** reach inside the prefab's children, so the rule when authoring is **the prefab holds the invariant frame and the scene adds what varies** (a title, a marker on a label).
  - **Folders say who owns a prefab.** The root of `assets/prefabs/` is the game's global set — the screen frame (`header`, `content`, `footer`, `banner`), the HUD widgets, the main menu's cells — and `prefabs/games/<family>/<game>/` holds one game's own, mirroring `scenes/games/<family>/`.
  - **A variant is a prefab made of a prefab.** `PrefabAsset { root: Object::prefab(base).with(..) }` is Unity's Prefab Variant in plain Rust: the base, with changes of its own. Three layers can meet on one object — the base, the variant, the scene's instance — and the last one wins, by component type, while children add up.
  - **An override is a component of the same type**, on an instance or on a variant's root. The editor marks it, and **Revert** drops it (the prefab's value comes back) while **Apply** writes it into the prefab, so every scene that uses it follows.
  - **A prefab is a module path.** `prefabs/games/solo/qanat/cell.prefab.rs` is written as `crate::assets::prefabs::games::solo::qanat::cell_prefab::prefab`, by the one rule in `xerxes_build::module_path` (the editor's `project::prefab_ref` uses it too, so the editor and the game always agree).
  - **The editor enters a prefab** from any instance of it and comes back out (`editor-layout.md`, prefab mode); a prefab has no `dimension` of its own, so the editor reads 2D or 3D off what it holds.
  - **A game spawns its prefabs while it plays**, so there is one place that says what a thing looks like: `spawn(&mut commands, Object::prefab(p).with(Transform::..), Some(parent))` returns the entity at once and applies the prefab when the command runs (Siavash's goons, flyers and road chunks are made this way).
- `build.rs` turns `assets/` into Rust modules plus a manifest, and packs the shipped files. It runs inside every build (cargo, dx, cargo-apk), so every platform gets the same result.

## Add-ons (`engine/addons/`)

Add-ons are **conditional game logic**: things a game may or may not have. It is compiled into the `xerxes_build` crate, so the editor, the backend (`GET /engine/addons`, `POST /engine/projects/{name}/addons`) and the CLI offer exactly the same set, with or without the repo. What every game has (scenes, prefabs, bundles, flow, input, project settings) is the engine, in `engine/src/modules/`. One folder per feature (rules in `engine/addons/README.md`); adding one copies it into the project's `src/addons/`, with what it needs, wires it into `src/lib.rs`, `src/addons/mod.rs` and the project file's `LOGIC`, and never overwrites a file the game already has:

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

## Progressive web loading (planned)

Today a game web build is one WASM with every scene and every system, plus art bundles. The plan is a **core** web feature: a slim boot package, then **code and art packages keyed to the scene flow**, so entering a level downloads what that level needs. Games stay Bevy-only (no Dioxus). The loader is engine-owned and stays thin — no new heavy dependencies. Design: [web-progressive.md](web-progressive.md). Stage **2j** in [`__plans__/PROGRESS.md`](../__plans__/PROGRESS.md). Not built yet.

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

UI is a Canvas of elements, Unity's model: see [ui.md](ui.md).

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
engine patches winit's Android input handler: `engine/vendors/winit` (winit 0.30.13, changes
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

## Safe area (Android insets)

Android apps draw edge-to-edge: the game and the editor fill the whole window, into the display
cutout (a camera notch) and under the navigation bar. The APK asks for this with a custom theme,
`@style/Theme.Xerxes` (`res/values/styles.xml`, `windowLayoutInDisplayCutoutMode = shortEdges`,
set as the activity theme in `Cargo.toml`), so a cutout no longer letterboxes the app with a black
band. Every target ships it: the engine app (`engine/platform/android/res`), the starter and so
every new game (`engine/starter/res`, carried by `xerxes_build::scaffold`), and the Darius games.

Drawing edge-to-edge means content must keep its interactive UI out from under the reserved edges.
The engine exposes them as a **`SafeArea { left, top, right, bottom }` resource** (logical px, `0`
on web and desktop), in `engine/src/modules/safe_area.rs`. On Android it is refreshed every frame
from `AndroidApp::content_rect()` (via `bevy::android::ANDROID_APP`, the handle `bevy_winit` uses)
compared to the window's physical size; it updates with the cutout and bars (orientation, gesture
vs 3-button nav). `SafeAreaPlugin` is installed by `UiPlugin`, so every game (`launch_project`) and
the editor's stage get the resource.

- **Games** pad their HUD or shift their world by `SafeArea`. The engine's bands
  (`modules::ui::bands`, see [ui.md](ui.md)) inset the header, content and footer and the camera
  gap by the safe area, so the HUD buttons never fall under the nav bar or cutout in either
  landscape.
- **The editor** reads `SafeArea` into its layout insets (`theme::Insets`, computed in
  `editor::stage::camera`), from which both the Dioxus panels and the Scene view cameras are placed
  inside the reserved edges — one computation, shared.

## Wall clock and the day (`modules::clock`)

A game that wants today's date (a daily seed, a daily reward) must not call
`std::time::SystemTime`: it panics in a browser. `engine/src/modules/clock.rs` reads the device's
clock (`Date.now()` through `js-sys` on the web, `SystemTime` elsewhere) and turns it into UTC days
and dates, so every device on the same day gets the same answer:

| Function | What it gives |
|---|---|
| `unix_seconds()` | seconds since 1970-01-01 UTC (0 if the clock cannot be read) |
| `day_number(secs)`, `today()` | whole days since 1970-01-01 UTC |
| `daily_seed(day, salt)` | a seed for a day; the game picks `salt` so two games do not share "day N" |
| `Date::from_day(day)` | a calendar date; `Display` is `YYYY-MM-DD` |

They are plain functions (no Bevy resource), so Bevy-free rules and a server take the day number
as an argument. A game logs the day with its result so a replay or a board entry is for a road.
First user: Chapar's daily road (`games/__templates__/darius/src/chapar/daily.rs`). Found missing
while building Chapar's slice 2 and added to the engine then, not kept in the game.

## Persistent storage (`modules::storage`)

Small persistent text for games, the same on every platform: `load(key)`, `save(key, text)`,
`remove(key)`. The web keeps it in local storage (`xerxes.data.<key>`), desktop in a file in the
user's data folder (`%APPDATA%/Xerxes/data`, `$XDG_DATA_HOME/xerxes` or `~/.local/share/xerxes`;
`XERXES_DATA` overrides it for tests), Android in the app's internal storage. Keys are the game's
own, `"<game>.<what>"`. A failed write never fails the game, and a write still reads back for the
rest of the session.

It is only the platform piece of the save stage (3.6): no schema, serializer or cloud hook yet; a
game encodes its own text (Chapar's ghost is one line). The editor's preferences are a separate
per-user store (`editor/prefs.rs`). First user: Chapar's ghost, added to the engine when the game
needed a best run to survive a reload.

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

nginx serves the wasm with its MIME type, gzips wasm/js/zip, and falls back to `index.html` for unknown paths. When progressive web loading (stage 2j) lands, the same image will serve boot + scene packages the same way; see [web-progressive.md](web-progressive.md).

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
