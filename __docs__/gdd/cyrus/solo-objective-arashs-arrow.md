# Arash's Arrow: Timed speedrun

Cyrus (3D) · **Solo x Objective** · Slice 1 (a 3D diorama of the first room) is built; the speedrun comes at R8

| | |
|---|---|
| Name | Arash the archer shot an arrow from Mount Damavand to set the border between Iran and Turan; legend says it flew from dawn to noon. |
| Mode | Solo x Objective ([taxonomy](../../game-modes.md)), tiered by time |
| Modifiers | none (the clock ranks the run, it does not cut it) |
| Camera | Third person, close behind the archer, with an aim line |
| Players | 1 (+ spectators) |
| Physics | Rapier 3D: the character controller, ledges, arrows as casts (fixed step, deterministic) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/solo/arashs_arrow.scene.rs` |
| Build rung | R8 3D core ([ladder](../README.md#the-build-ladder)) |
| Twin | [Qanat](../darius/solo-objective-qanat.md) (Darius (2D), same mode) |

## Pitch

A short course on the mountain: a room or two of rock and rope bridges, and divs (demons) that must all fall before the clock stops. Run, jump, aim and loose the arrow. Pick up cards on the way, each one an ability for one use (a second jump, a hook to a ring, an arrow that bounces, an arrow that bursts). Clear the last div and the clock stops; the medal is your time against the author's: Author, Gold, Silver, Bronze. Falling resets the course at once and the clock keeps going, so every mistake is paid in time, not in a life.

## Reference: Neon White

Neon White is the speedrun game: short levels, a target list (the demons), cards that are abilities used once, a medal by time, an instant restart, and a leaderboard for every level. Arash's Arrow takes all of that, with the Persian mountain and a bow for the body. It does **not** take the first-person view or the story: the third person keeps the archer readable beside Croesus' Gold and the 3D core.

## Why it is this mode

A set goal (clear every div), tiered by time instead of pass or fail: Solo x Objective. The time is the score, and a board per level makes it a race against yourself and everyone else.

## Rules (the `Sim`)

- **State:** Archer (position, velocity, grounded, cards in hand), the divs alive, the arrow in flight (if any), the level clock, the course (rooms, ledges, cards on the map).
- **Inputs:** Move, jump, aim, loose an arrow along the aim, use the card in hand, restart. The sim gets the same input values from a keyboard, a pad or touch.
- **The clock:** starts at the first input, stops when the last div falls. It is never a limit: no time runs out.
- **Cards:** picked up on the course, held in a short row; each use spends it. A level's cards are fixed by the course, so a route is a choice.
- **Falling:** the course resets to its start at once; the clock keeps running.
- **A round ends when:** the last div falls (win, with the time and a medal) or the player restarts (the clock is reset, no record kept).
- **Result:** Finished, with a time and a medal; ranked by time on the level board.
- **Determinism:** the physics runs at a fixed step, so a recorded run (its inputs) replays to the same time. That is what a board entry and a ghost are.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD move; Space jump; mouse aim; left click loose the arrow; E use the card; R restart | Left stick move; right stick aim; fire button; card button; jump button; restart in the header |

## Offline, online, spectate

- **Offline:** the course, your medal, your best time, and a ghost of your best run.
- **Online:** a board per level (best time) and a total board (the sum of best levels). Submitted at the end of a level.
- **Spectate:** watch a live run from behind the archer; replay a board entry from its inputs.

## What it forces in the engine

- A 3D character controller with a third-person camera (the controller add-on, second use after Croesus' Gold)
- Arrows as cheap casts in a deterministic 3D world
- Cards as data: an ability is a `Condition` / `Action`-shaped record, used once
- A level clock and medals as rules, shown in the HUD
- Per-level boards and replays (M5 and M3)

## Slices

1. **Built (diorama, not yet this game):** the first room as a 3D diorama, from the Qanat rules; the orbit camera and taps. Kept until the speedrun replaces it at R8.
2. One course: the archer, jump, the arrow, three divs, the clock and a bronze and gold medal
3. Cards (second jump, hook, bounce) and five courses; all four medals (Author, Gold, Silver, Bronze)
4. Local level boards; the ghost of the best run
5. Online boards per level and the total (M5); spectate (M3)

## Open questions

- One mountain with a course per level, or a chain of shrines the archer climbs?
- Should a fall cost time (a penalty) or only the reset it already costs?
- Do cards come from the course only, or from a short deck the player brings?
- Are the medal times set by hand, or by a solver (as Qanat's hint is)?
- First person later (Atashkadeh's controller), or third person for good?
