# Cyrus

A 3D game on the [Xerxes engine](../../../README.md) with a small mini-game for every game mode of the taxonomy, each playable offline, online and spectated: the 3D sibling of [Darius](../darius/README.md).

Like Darius it is developed in place here, in `games/__templates__/` (committed with the engine), while the engine grows. The plan is in [`__docs__/game-cyrus.md`](../../../__docs__/game-cyrus.md), and every game's design (a GDD each) and the build order in [`__docs__/gdd/`](../../../__docs__/gdd/README.md).

**Where it is today:** scaffolded. The menu is the game mode taxonomy as a grid with a tile for each of the 21 games; every game is a real level with its game mode. Three have code beyond the card: **Siavash** (Solo · Score, slice 1: the knight runs on three lanes and attacks by direction, as in Blades of Brim: switch into a goon's lane at the right moment or dive on it, jump through a flyer, roll under it; running straight into a goon ends the run; R runs again; its rules are the `rules/` crate), **Pairidaeza** (Sandbox, slice 0: a 3D physics lab, spawn, drag and gravity) and **Arash's Arrow** (a 3D clone of Qanat's first room, the wrong idea for this game; it is reworked into the Neon White speedrun at R9, after the first-person controller). The other 18 are empty 3D scenes (camera, sun, ground, and a card with its title, mode and camera, *Coming soon* and Back to menu).

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
| `src/lib.rs` | The `game` lib: `pub fn run()` calls `launch_project(assets::project())`; `assets` is generated from `assets/` by `build.rs` |
| `src/main.rs`, `src/android.rs` | Platform entries (desktop/web, Android `#[bevy_main]`); both call `game::run()` |
| `src/logic.rs` | Plugs `arashs_arrow` and `pairidaeza` into the project's `LOGIC` list |
| `src/arashs_arrow/` | Arash's Arrow: today the 3D diorama (`play.rs`), to be reworked into the Neon White speedrun |
| `src/pairidaeza/` | Pairidaeza: the 3D physics lab, slice 0 (`play.rs`) |
| `assets/cyrus.project.rs` | Settings and flow: the menu scene and the 21 levels, each with its `GameMode` |
| `assets/scenes/menu.scene.rs` | The menu: the taxonomy grid of game tiles (a 2D UI scene) |
| `assets/scenes/games/<family>/<game>.scene.rs` | One 3D scene per game, in `solo`, `coop`, `duel`, `ngon`, `outside` |

Bevy features: the `2d` and `3d` collections (scenes are 2D or 3D, not games; every game builds both). The launcher icons in `res/` are Darius's for now.
