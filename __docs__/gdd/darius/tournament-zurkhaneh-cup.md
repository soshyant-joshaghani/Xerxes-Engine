# Zurkhaneh Cup: Duel bracket

Darius (2D) · **Tournament** · Coming soon (empty scene)

| | |
|---|---|
| Name | The zurkhaneh ('house of strength') is where the pahlevans train; the cup is its tournament. |
| Mode | Tournament ([taxonomy](../../game-modes.md)) |
| Modifiers | wraps Duel x Survival (Mobarezeh) |
| Camera | Bracket screen, then Mobarezeh matches |
| Players | 4 to 8 (+ spectators) |
| Physics | as Mobarezeh |
| Scene | `games/__templates__/darius/assets/scenes/games/outside/zurkhaneh_cup.scene.rs` |
| Build rung | R10 Scale and live ([ladder](../README.md#the-build-ladder)) |
| Twin | [Farr League](../cyrus/tournament-farr-league.md) (Cyrus (3D), same mode) |

## Pitch

Four or eight fighters (or teams), one bracket. Winners advance until one holds the cup. The same wrapper can hold any Duel or N-gon game.

## Why it is this mode

Not a mode itself: a wrapper that repeats a Duel cell over elimination rounds. Tournament.

## Rules (the `Sim`)

- **State:** The bracket, current matches, results.
- **Inputs:** Ready, then the wrapped game's inputs.
- **A round ends when:** The final is played.
- **Result:** Bracket placement.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| as Mobarezeh | as Mobarezeh |

## Offline, online, spectate

- **Offline:** You + bots in the bracket.
- **Online:** Registration, a bracket room, match rooms.
- **Spectate:** The bracket page and each live match.

## What it forces in the engine

- A wrapper over any Duel / N-gon cell
- Bracket state, scheduled starts (M5)

## Slices

1. Offline bracket with bots
2. Online registration and match rooms

## Open questions

- Single or double elimination?
