# Qanat: Water-channel puzzle

Darius (2D) · **Solo x Objective** · Playable (first slice); the timer and the boards are the next slice

| | |
|---|---|
| Name | A qanat is the Persian underground aqueduct that carries mountain water to a village. |
| Mode | Solo x Objective ([taxonomy](../../game-modes.md)) |
| Modifiers | Timer: a clock per room and a board. It ranks the solve and never fails a room |
| Camera | Side cut-away of a hillside, grid |
| Players | 1 (+ spectators) |
| Physics | none (grid, turn-based) |
| Scene | `games/__templates__/darius/assets/scenes/games/solo/qanat.scene.rs` |
| Build rung | R1 Rules without physics ([ladder](../README.md#the-build-ladder)) |
| Twin | [Arash's Arrow](../cyrus/solo-objective-arashs-arrow.md) (Cyrus (3D), same mode) |

## Pitch

A hillside cut open like a diorama: a spring high in the rock, a dry village below. Lay and rotate channel tiles, then open the sluice and watch the water run. Get it to the village to clear the room; fewer tiles earns bronze, silver or gold. A clock runs from your first tile until the water reaches the village, and the rooms are ranked on a board by that time, so a puzzle becomes a race against the room as well as against the tile count.

## Why it is this mode

Pass or fail a set goal, tiered instead of ranked: Solo x Objective (challenge clear). The clock adds a board without changing the goal: a solve that fails is not timed, and a solve that wins is.

## Rules (the `Sim`)

- **State:** A tile grid (rock, soil, channel pieces, sluice), the tile budget, flow state after the sluice opens, the room's clock.
- **Inputs:** Place / rotate / remove a tile, open the sluice, undo, restart. A move into rock is rejected.
- **Timer:** starts at the first placed tile and stops when the water reaches the village. Turn-based thinking counts: the clock is the room's record, not a rule the player is racing. It never ends a room; undo and restart keep running the clock.
- **A round ends when:** The water reaches the village (win) or settles without reaching it (try again).
- **Result:** Won with a tier (gold/silver/bronze by tiles used) and a time. The board ranks solves by time, then by tiles.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| Click a cell, or arrows + Space; Z undo, C clear, Enter open the sluice, R restart | Tap a cell, tap again to rotate |

## Offline, online, spectate

- **Offline:** Alone; your best time and tiles per room are kept on this device. A solver bot (BFS over placements) is the Hint button. Using the hint keeps the solve off the board.
- **Online:** a board per room (best time, then fewest tiles). A board entry is the solve's inputs and their timing, so the server checks the time by running the same rules.
- **Spectate:** Watch a live solve (a cheap step stream) or the replay of a solution.

## What it forces in the engine

- A step (turn-based) world in `xerxes_sim` next to the realtime one
- Undo as part of the rules, rejected inputs
- Tiered results
- A fixed-tick flood fill for the water
- A clock and a board per room: time is a record of the solve, checked by replay

## Slices

1. Done 2026-10-12: one hand-made room (9 x 7), lay/turn/undo/clear, water that never runs uphill, win with a tier; rules in `src/qanat/rules.rs`
2. Tiers and 10 rooms
3. Hint solver (a hinted solve is unranked)
4. Timer and the local board per room (time, then tiles)
5. Replays (M3) and the online board per room (M1, step 9)

## Open questions

- Hand-made rooms only, or a generator later?
- Should the clock pause while the player is not placing (a long think is slow, not wrong), or run as drawn?
- Is the board ranked by time first and tiles second (as drawn), or the tier first and time second?
- Once Arash's Arrow stops using the step rules, do they move back into the game (`src/qanat/`) until a second game needs them? (The rules moved into `xerxes_sim::qanat` for Gates of Babylon, 2026-10-14.)
