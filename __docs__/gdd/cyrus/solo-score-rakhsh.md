# Rakhsh: Endless lane run

Cyrus (3D) · **Solo x Score** · Coming soon (empty scene)

| | |
|---|---|
| Name | Rakhsh is Rostam's horse in the Shahnameh, the one who carries the hero through every trial. |
| Mode | Solo x Score ([taxonomy](../../game-modes.md)), endless, high score |
| Modifiers | none |
| Camera | Third-person chase, behind the horse, three lanes |
| Players | 1 (+ spectators) |
| Physics | None in the rules: lanes and a jump / slide arc are a fixed step (`src/rakhsh/rules.rs`), 3D only in the view |
| Scene | `games/__templates__/cyrus/assets/scenes/games/solo/rakhsh.scene.rs` |
| Build rung | R3 Side view and ghosts, as the 3D camera rig ([ladder](../README.md#the-build-ladder)) |
| Twin | [Chapar](../darius/solo-score-chapar.md) (Darius (2D), same mode) |

## Pitch

A road out of Ecbatana, three lanes, and a horse that never stops. Switch lanes, jump the gates, slide under the banners, and slash the divs that step into your lane with your blade. Coins and power-ups line the road; the score is distance, coins and the slash combo. Crash and you start again at once.

## Reference: Subway Surfers and Blades of Brim

Subway Surfers is the lane runner: swipe between three lanes, jump and roll, collect coins, take power-ups, the run ends on the first crash. Blades of Brim adds the blade: the runner slashes what is in the lane, and a chain of slashes is a combo. Rakhsh takes both, with the Persian road and a horse for the body. It does **not** take the on-foot run (the stride, the wall-runs) or the fully authored city: a horse with three lanes is the whole game.

## Why it is this mode

No finish line and no opponent: an endless score attack, like Chapar in 2D. The number grows with distance, coins and the combo, and the better you read the road the faster it grows. That is Solo x Score in 3D.

## Rules (the `Sim`)

- **State:** Rider (lane 0 to 2, height above the road, vertical speed, sliding), speed, the road ahead (lanes, from the seed), the blade's combo and window, distance, coins, power-up timers.
- **Inputs:** Lane (-1 left, 0 none, +1 right), jump, slide, slash: five values per tick, so a replay is a short line of text.
- **Lanes:** the lane changes at once in the rules; the view glides over about 0.12 s. A lane change in the air is allowed.
- **Speed:** a function of the distance run (it builds with distance, a cap at the top), not of a clock, so the generator can keep every obstacle fair at the speed the rider will have there.
- **Obstacles:** a gate (jump over it), a banner (slide under it), a fallen column (change lane). A div in a lane is an enemy: it must be slashed before it reaches you, or you must leave its lane. The first 300 m are gates and columns only. After every obstacle there is more than 0.75 s of road to read.
- **Slash:** a swing that clears the nearest div in the lane ahead, within reach. Kills in one window (0.8 s) chain a combo, which multiplies the score; a missed swing costs the window.
- **Power-ups:** a **shield** (the Simurgh's feather: one crash forgiven), a **magnet** (coins come to you for a few seconds), a **boost** (faster speed, wider reach for the slash).
- **Feel:** a jump pressed up to 0.12 s before landing is kept; a jump up to 0.10 s after leaving the road still counts. Slide and jump input are buffered the same way.
- **A round ends when:** the rider hits a gate, a column, a banner, or a div it did not slash (and no shield is left).
- **Result:** Finished, ranked by score (distance and coins shown beside it, the combo as the tiebreak).
- **Determinism:** the same seed and inputs give the same run on any device. A ghost, a replay and a daily board are inputs against a seed.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| A / D or Left / Right change lane; W / Space jump; S / Down slide; J or left click slash | Swipe left / right to change lane, swipe up to jump, swipe down to slide, tap to slash |

## Offline, online, spectate

- **Offline:** the daily seed (the same road for everyone today); your best run as a ghost (step 6).
- **Online:** a daily-seed board, the same as Chapar's.
- **Spectate:** follow a live run from the chase camera; replay a board entry from its inputs.

## What it forces in the engine

- A 3D chase camera rig with lane smoothing (the first camera rig in Cyrus)
- A lane model in a fixed step: the lanes, the jump arc and the slide are data, drawn in 3D
- Obstacles and enemies streamed from a seed, laid ahead of the rider as in Chapar
- A combo and a power-up timer as plain state (no physics)
- Ghosts: Chapar's recording moves into `xerxes_sim` at step 6 (second use)

## Slices

1. Three lanes, the rider, jump and slide, gates and columns, distance, crash and retry; a chase camera
2. Coins and the three power-ups; the score with its multiplier
3. Divs and the blade: the slash, the combo window, the score
4. Ghost of the best run (step 6)
5. Daily board online

## Open questions

- Three lanes or five? Three reads better on a phone; five is the Subway default.
- Does the slash need its own button on touch, or is a tap enough (as drawn above)?
- Does the horse tire, as Chapar's open question asked? For now the speed only builds.
- Free running between lanes (as a horse on a plain) or lanes only (as drawn)?
