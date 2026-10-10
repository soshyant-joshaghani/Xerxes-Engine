# Darius

A 2D game on the [Xerxes engine](../../../README.md) with a small mini-game for every game mode of the taxonomy, each playable offline, online and spectated.

It is the engine's working game while the engine grows its tools (Phase 3), so it lives here in `games/__templates__/` (committed with the engine) and is developed in place. Its 3D sibling, Cyrus, comes later. The plan, with every mini-game and the stage that unlocks it, is in [`__docs__/darius.md`](../../../__docs__/darius.md).

**Where it is today:** on the asset model (stage 3.0): a project file, a menu scene (title and Play) and one walled arena scene, with a player square and coins whose collision is the engine's physics module (stage 3.1, over the `xerxes_sim` crate). The mode mini-games ([ideas](../../../__docs__/darius-games.md)) come next.

## Run it

```bat
__ctrl__\xerxes-ctrl.bat template dev darius web        REM dx serve with hot reload (port 5200, or the next free one)
__ctrl__\xerxes-ctrl.bat template dev darius windows    REM Bevy window
__ctrl__\xerxes-ctrl.bat template dev darius android    REM APK on the adb device
__ctrl__\xerxes-ctrl.bat template publish darius        REM release build into dist/darius/
```

A template game is **read-only in the editor**. To open Darius in the editor, clone it into your own project: Project Manager, "Use as template". The game itself is edited as files here.

## Layout

| Path | Holds |
|------|-------|
| `src/lib.rs` | The `game` lib: `pub fn run()` calls `launch_project(assets::project())`; `assets` is generated from `assets/` by `build.rs` |
| `src/main.rs`, `src/android.rs` | Platform entries (desktop/web, Android `#[bevy_main]`); both call `game::run()` |
| `src/top_down/` | The arena's rules in Bevy: `world.rs` (player steering, coins as sensors, score), `hud.rs` (bevy_ui HUD), plugged in by `assets/logic/top_down.rs` |
| `assets/` | `darius.project.rs` (settings, flow), `scenes/menu.scene.rs`, `scenes/arena.scene.rs`, `logic/top_down.rs` |

Bevy features: the `2d` and `3d` collections (scenes are 2D or 3D, not games; every game builds both).
