# Farr League: Team bracket

Cyrus (3D) · **Tournament** · Coming soon (empty scene)

| | |
|---|---|
| Name | A league of Farr, the carry-the-glory team game. (Name provisional.) |
| Mode | Tournament ([taxonomy](../../game-modes.md)) |
| Modifiers | wraps Duel x Score (Farr) |
| Camera | Bracket, then Farr matches |
| Players | 4 to 8 teams (+ spectators) |
| Physics | as Farr |
| Scene | `games/__templates__/cyrus/assets/scenes/games/outside/farr_league.scene.rs` |
| Build rung | R10 Scale and live ([ladder](../README.md#the-build-ladder)) |
| Twin | [Zurkhaneh Cup](../darius/tournament-zurkhaneh-cup.md) (Darius (2D), same mode) |

## Pitch

Teams register, a bracket fills, winners play on. The wrapper of Zurkhaneh Cup, over a team game in 3D.

## Why it is this mode

A wrapper over a Duel cell: Tournament.

## Rules (the `Sim`)

- **State:** Bracket, matches.
- **Inputs:** Ready, then Farr.
- **A round ends when:** The final.
- **Result:** Placement.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| as Farr | as Farr |

## Offline, online, spectate

- **Offline:** You + bot teams.
- **Online:** Team registration, bracket and match rooms.
- **Spectate:** Bracket and live matches.

## What it forces in the engine

- The tournament wrapper with teams

## Slices

1. Offline bracket
2. Online teams

## Open questions

- Seasons?
- Which game does the league wrap in the end (Farr or Atashkadeh)?
