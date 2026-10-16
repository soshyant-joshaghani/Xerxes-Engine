# Cyrus

A 3D game on the [Xerxes engine](../../../README.md) with a small mini-game for every game mode of the taxonomy, each playable offline, online and spectated: the 3D sibling of [Darius](../darius/README.md).

Like Darius it is developed in place here, in `games/__templates__/` (committed with the engine), while the engine grows. The plan is in [`__docs__/cyrus.md`](../../../__docs__/cyrus.md), and every game's design (a GDD each) and the build order in [`__docs__/gdd/`](../../../__docs__/gdd/README.md).

**Where it is today:** scaffolded. The menu is the game mode taxonomy as a grid with a tile for each of the 21 games; every game is a real level with its game mode and an empty 3D scene (camera, sun, ground, and a card with its title, mode and camera, *Coming soon* and Back to menu). The first games to be built are Rakhsh (rung R3, a lane runner) and Arash's Arrow (its diorama slice is in; the speedrun waits for the 3D core, R8).

## Run it

```bat
__ctrl__\xerxes-ctrl.bat template dev cyrus web        REM dx serve with hot reload (port 5200, or the next free one)
__ctrl__\xerxes-ctrl.bat template dev cyrus windows    REM Bevy window
__ctrl__\xerxes-ctrl.bat template dev cyrus android    REM APK on the adb device
__ctrl__\xerxes-ctrl.bat template publish cyrus        REM release build into dist/cyrus/
```

A template game is **read-only in the editor**. To open Cyrus in the editor, clone it into your own project: Project Manager, "Use as template". The game itself is edited as files here.

## Layout

| Path | Holds |
|------|-------|
| `src/lib.rs` | The `game` lib: `pub fn run()` calls `launch_project(assets::project())`; `assets` is generated from `assets/` by `build.rs`. Each game gets its own `src/<game>/` when it is built |
| `src/main.rs`, `src/android.rs` | Platform entries (desktop/web, Android `#[bevy_main]`); both call `game::run()` |
| `assets/cyrus.project.rs` | Settings and flow: the menu scene and the 21 levels, each with its `GameMode` |
| `assets/scenes/menu.scene.rs` | The menu: the taxonomy grid of game tiles (a 2D UI scene) |
| `assets/scenes/games/<family>/<game>.scene.rs` | One 3D scene per game, in `solo`, `coop`, `duel`, `ngon`, `outside` |

Bevy features: the `2d` and `3d` collections (scenes are 2D or 3D, not games; every game builds both). The launcher icons in `res/` are Darius's for now.
