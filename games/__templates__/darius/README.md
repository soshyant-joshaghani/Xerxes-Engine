# Darius

A 2D game on the [Xerxes engine](../../../README.md) with a small mini-game for every game mode of the taxonomy, each playable offline, online and spectated.

It is the engine's working game while the engine grows its tools (Phase 3), so it lives here in `games/__templates__/` (committed with the engine) and is developed in place. Its 3D sibling is [Cyrus](../cyrus/README.md). The plan is in [`__docs__/darius.md`](../../../__docs__/darius.md), and every game's design (a GDD each) and the build order in [`__docs__/gdd/`](../../../__docs__/gdd/README.md).

**Where it is today:** the menu is the game mode taxonomy as a grid, with a tile for each of the 21 games. Four games play:

- **Ahriman's Horde** (Solo · Survival), slice 1: a walled courtyard with a pistol, the Fire / Switch / Reload-Dash pad and one rusher; kill it and step into the warp. Waves are the next slice ([GDD](../../../__docs__/gdd/darius/solo-survival-ahrimans-horde.md)).
- **Qanat** (Solo · Objective), its first room: lay and turn channel tiles in a cut-away hillside, open the sluice, and the water runs (never uphill) to the village for a gold, silver or bronze tier. Its rules are in the engine's `xerxes_sim::qanat`, shared with Cyrus. The timer and the boards are the next slice.
- **Chapar** (Solo · Score), slices 1 to 4: the day's road (seeded from the UTC date), jump (hold for height) and duck, blocks, hanging banners and pits, relay stations whose token a held jump takes for a sprint, distance as the score, and your best run of the day as a ghost. Its run is Bevy-free in `src/chapar/rules.rs`.
- **Jamshid's Workshop** (Sandbox), slice 1: spawn and drag toys, tilt gravity with the slider. No win condition.

The other 17 games are empty scenes (title, mode, camera, *Coming soon*, Back to menu), wired as real levels with their game modes, built in the order of the [build ladder](../../../__docs__/gdd/README.md#the-build-ladder).

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
| `src/ahrimans_horde/` | Ahriman's Horde in Bevy: `combat.rs` (pistol, pad, rusher, warp), `controls.rs` (the pad), `world.rs` (player steering, darics as sensors, score), `hud.rs` (bevy_ui HUD), plugged in by `src/logic.rs` |
| `src/jamshids_workshop/` | Jamshid's Workshop in Bevy: `play.rs` (spawn and drag toys, gravity slider), plugged in by `src/logic.rs` |
| `src/qanat/` | Qanat: `play.rs` (input, water clock, drawing, HUD), plugged in by `src/logic.rs`. Its rules are the engine's `xerxes_sim::qanat` |
| `src/chapar/` | Chapar: `rules.rs` (the Bevy-free, seeded, deterministic run with stations, with tests; moves to `rules/` at step 5, when Chapar's daily board runs in a room), `daily.rs` (the day's seed from the date), `ghost.rs` (recording and replaying inputs), `play.rs` (input per fixed tick, scrolling view, HUD, the match report), plugged in by `src/logic.rs` |
| `assets/darius.project.rs` | Settings and flow: the menu scene and the 21 levels, each with its `GameMode` |
| `assets/scenes/menu.scene.rs` | The menu: the taxonomy grid of game tiles |
| `assets/scenes/games/<family>/<game>.scene.rs` | One scene per game, in `solo`, `coop`, `duel`, `ngon`, `outside` |
| `src/logic.rs` | Plugs a game's code in (`ahrimans_horde`, `chapar`, `jamshids_workshop`, `qanat`; logic modules are project-wide, so each game's systems only act on its own scene's markers) |

Bevy features: the `2d` and `3d` collections (scenes are 2D or 3D, not games; every game builds both).
