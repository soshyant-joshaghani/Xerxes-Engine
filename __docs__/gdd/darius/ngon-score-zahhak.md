# Zahhak: Serpent arena

Darius (2D) · **N-gon x Score** · Slice 1 playable: you plus three bots

| | |
|---|---|
| Name | Zahhak is the Shahnameh tyrant with a serpent growing from each shoulder. |
| Mode | N-gon x Score ([taxonomy](../../game-modes.md)), multi-way |
| Modifiers | TimeLimit 3 min, Respawn (a short delay, back small) |
| Camera | Fixed top-down view of the round field |
| Players | Slice 1: 4 (you plus 3 bots); later: up to 16 (+ spectators) |
| Physics | none (continuous 2D motion, circle tests) |
| Screens | Local: split viewports (up to 2 x 2). Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/darius/assets/scenes/games/ngon/zahhak.scene.rs` |
| Build rung | R2 Participants and bots ([ladder](../README.md#the-build-ladder)) |
| Twin | [Kuchulu Cup](../cyrus/ngon-score-kuchulu-cup.md) (Cyrus (3D), same mode) |

## Pitch

Slither.io, in rounds. Everyone is a serpent on one open field, eating to grow. Steer, boost, and cut others off: hit another serpent's body and you burst into food for whoever is nearby. The round is three minutes. The longest serpent at the bell wins; die and you come back small, so the board never goes quiet.

## Why it is this mode

The number is length, ranked on a board, and nothing is won by being the last one alive: N-gon x Score. (Slither.io's own leaderboard is the model; a timer turns it into a ranked round.)

## Reference

[Slither.io](https://en.wikipedia.org/wiki/Slither.io) (and Snake battle royales in general): continuous steering, a speed boost that costs length, food from the dead, one open field.

## Rules (the `Sim`)

- **State:** the field (a round arena with a wall), serpents (a head, a body of segments, length, boost state), food pellets, each side's score, the clock.
- **Inputs:** a steering direction (the stick, or the pointer's direction); **Utility = boost** (faster, costs length, leaves pellets behind). No other buttons.
- **Steering:** a serpent moves at a constant speed along its heading and turns at a limited rate, so a long serpent is harder to place.
- **Food:** pellets spawn over the field; eating grows the serpent. A dead serpent's body becomes food, worth its length.
- **Death:** a serpent's head touching any other serpent's body, or the wall, kills it. Head against head kills the shorter one. The dead respawn after about 3 s at the starting length.
- **Score:** the length held when the clock ends. Dying late costs you, so the best play is to stay alive and eat.
- **A round ends when:** the clock reaches zero.
- **Result:** ranked by length (equal lengths share a rank).

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD / arrows or drag in the field steer; hold Shift to boost after growing | Drag in the field steers; the Boost button works after growing |

Your serpent is cyan with a white outline, and the HUD labels your length as **YOU**. The opening
spawns four serpents in separate lanes so the player has time to identify their worm and steer.

## Screens

- **Local:** the field scrolls, so up to four local players get viewports (2 side by side, up to 2 x 2). Nothing is hidden.
- **Online:** one camera per player.

## Offline, online, spectate

- **Offline:** you plus **bots** (greedy foragers; some try to circle a longer serpent) filling to 8 or more.
- **Online:** a quick queue, rooms of up to 16.
- **Spectate:** an overview, or follow the longest; replay from the inputs.

## What it forces in the engine

- N sides on one field, filled with bots (the first bot slice of stage 3.3)
- Many small entities (pellets, segments) and fast circle tests
- A deterministic fixed-step world for replay and later server authority
- Several viewports in one scene

## Slices

1. One field, you and 3 bots, steering, food, death and the length score (playable)
2. Boost that costs length, food from the dead, the timer and respawn
3. 16 serpents, local split screen
4. Online rooms; spectate

## Open questions

- Is the score the length at the bell (as written) or the peak length reached?
- Is the field one fixed size, or does it shrink late (a ShrinkingZone) to force contact?
