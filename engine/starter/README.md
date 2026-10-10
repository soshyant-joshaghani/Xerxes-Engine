# Starter

An empty game made with the [Xerxes engine](../../../README.md) (`../../../engine`). It is plain Bevy, with a bevy_ui UI and no Dioxus in the game or its exports, and it is made of `.rs` assets: project settings, and one 3D scene (a camera, a sun, a ground plane). Scenes are 2D or 3D; add 2D scenes from the editor too.

Open it in the Xerxes editor (Project Manager), or work from the CLI:

```bat
__ctrl__\xerxes-ctrl.bat game dev <name> web         REM http://game.localhost (:5200)
__ctrl__\xerxes-ctrl.bat game dev <name> windows     REM Bevy window
__ctrl__\xerxes-ctrl.bat game publish <name>         REM release builds into dist/<name>/ (web with a Dockerfile)
```

## Layout

| Path | Holds |
|------|-------|
| `assets/<name>.project.rs` | Title, flow (menu, levels), preload bundles, input actions, window |
| `assets/scenes/main.scene.rs` | The start scene |
| `assets/logic/*.rs` | Logic modules (components + systems): copy them from the engine templates when the game needs them (camera and character controllers, HUD, pause) |
| `assets/art/` | External files and their typed metas (`crate.png` + `crate.image.rs`: import settings, bundle id) |
| `build.rs` | `xerxes_build::generate()`: `assets/` → Rust modules + `public/bundles/<id>.zip` |
| `public/` | Generated, gitignored: the bundles, the only files that ship |
| `src/lib.rs`, `main.rs`, `android.rs` | `run()` and the two entry points; they only call it |
