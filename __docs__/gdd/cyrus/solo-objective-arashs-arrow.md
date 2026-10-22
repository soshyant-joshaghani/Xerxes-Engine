# Arash's Arrow: Speedrun with cards and demons

Cyrus (3D) · **Solo x Objective** · The scene today is a 3D clone of Qanat's first room (wrong for this game, to be reworked); the Neon White speedrun is slice 1 and comes at R9, with the first-person controller

| | |
|---|---|
| Name | Arash the archer shot an arrow from Mount Damavand to set the border between Iran and Turan; legend says it flew from dawn to noon. |
| Mode | Solo x Objective ([taxonomy](../../game-modes.md)), tiered by time: every demon must fall before the end point opens, then the time ranks the run |
| Modifiers | none (the clock ranks the run, it does not cut it) |
| Camera | First person, the bow in view, with an aim point (pointer-locked where the platform allows) |
| Players | 1 (+ spectators) |
| Physics | Rapier 3D: the first-person controller, ledges, arrows and card shots as casts (fixed step, deterministic) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/solo/arashs_arrow.scene.rs` |
| Build rung | R9 First person ([ladder](../README.md#the-build-ladder)), after the first-person controller of [Atashkadeh](duel-role-atashkadeh.md) |
| Twin | [Qanat](../darius/solo-objective-qanat.md) (Darius (2D), same mode) |

## Pitch

Short courses on Damavand: rooms, ledges and rope bridges, and demons (divs) standing between the archer and the **end point**. The end point stays shut until the last demon has fallen. Run, jump, and shoot.

Cards are the weapons and the abilities. Each card is a gun pickup on the course. Hold up to three. Use a card once, one of two ways:

- **Shoot** it: its gun fires once, down the aim.
- **Burn** it: its ability fires once (double jump, dash, cannon throw, smash down, and more).

The bow in hand is the base weapon: plain arrows, no ammo.

The clock starts at your first input and stops when you reach the open end point after the last demon. Falling resets the course at once and the clock keeps going, so every mistake is paid in time, not in a life. The medal is your time against the author's: Author, Gold, Silver, Bronze.

## Reference: Neon White

Neon White is the speedrun: short levels, a list of demons to clear before the exit, cards that are both guns and one-use abilities, a medal by time, an instant restart, and a leaderboard for every level. Arash's Arrow takes those rules as they are, with the Persian mountain and a bow for the body. The camera is first person, as Neon White's is, and the bow stands in for the base gun. It does **not** take the story. The first-person view is why this game moves from the 3D core (R8) to R9: it waits for the first-person controller that Atashkadeh builds (step 34 in [PROGRESS.md](../../../__plans__/PROGRESS.md)).

## Why it is this mode

A set goal (kill every demon, then reach the end point), tiered by time instead of pass or fail: Solo x Objective. The time is the score, and a board per level makes it a race against yourself and everyone else. The demons come first on purpose: the time only starts to count against you once the course is clear of them, so the question is always how fast the rest of the route goes.

## Rules (the `Sim`)

- **State:** Archer (position, velocity, grounded, air jumps left, dash charge, the hand of cards), the demons alive, the arrows and card shots in flight, the end point (open or shut), the level clock, the course (rooms, ledges, cards on the map).
- **Inputs:** Move, look, jump, shoot (the first card in hand, or the bow when the hand is empty), burn (the first card in hand), restart. The sim gets the same input values from a keyboard, a pad or touch.
- **The bow:** always in hand; a plain arrow every 0.4 s, no ammo.
- **The clock:** starts at the first input, stops when the archer crosses the end point. It is never a limit: no time runs out.
- **The end point:** shut while any demon is alive; open the tick the last one falls.
- **Cards:** picked up on the course, held in a hand of up to three; a card is spent by one use, shot or burned. A level's cards are fixed by the course, so a route is a choice.
- **Card set (first):** *double jump* (one more jump in the air), *dash* (a burst along the aim), *cannon throw* (launch along the aim), *smash down* (drop straight down and break what is under the archer, the demons in reach included). More cards come with the courses.
- **Falling:** the course resets to its start at once; the clock keeps running.
- **A round ends when:** the archer crosses the open end point (win, with the time and a medal) or restarts (the clock is reset, no record kept).
- **Result:** Finished, with a time and a medal; ranked by time on the level board.
- **Determinism:** the physics runs at a fixed step, so a recorded run (its inputs) replays to the same time. That is what a board entry and a ghost are.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD move; Space jump; mouse look; left click shoot; right click burn; R restart | Left stick move; right stick look; fire button; burn button; jump button; restart in the header |

## Offline, online, spectate

- **Offline:** the course, your medal, your best time, and a ghost of your best run.
- **Online:** a board per level (best time) and a total board (the sum of best levels). Submitted when the end point is crossed.
- **Spectate:** watch a live run from behind the archer; replay a board entry from its inputs.

## What it forces in the engine

- A first-person controller with pointer lock (Atashkadeh's controller, the Phase 5 FPS work)
- Arrows and card shots as cheap casts in a deterministic 3D world
- Cards as data: a card holds one gun and one ability, used once (the same shape as `Condition` / `Action` in stage 3.5)
- A level clock, an end point that opens on a rule, and medals shown in the HUD
- Per-level boards and replays (M5 and M3)

## Slices

1. **Rework (R9): replace the Qanat clone.** The scene built so far is a 3D copy of Qanat's first room, run on Qanat's rules, with an orbit camera and taps. That is Qanat's game, not this one. The rework removes Qanat's rules from this scene and builds the first course below in its place. Until then the copy is a placeholder and changes nothing else.
2. One course, first person: the archer, jump, the bow, three demons, the end point that opens on the last one, the clock, and a bronze and gold medal
3. Cards: shoot and burn; double jump, dash, cannon throw and smash down; five courses; all four medals (Author, Gold, Silver, Bronze)
4. Local level boards; the ghost of the best run
5. Online boards per level and the total (M5); spectate (M3)

## Open questions

- One mountain with a course per level, or a chain of shrines the archer climbs?
- Should a fall cost time (a penalty) or only the reset it already costs?
- Do cards come from the course only, or from a short deck the player brings?
- Are the medal times set by hand, or by a solver (as Qanat's hint is)?
- Does the touch layout keep the aim on the right stick, or does it need an aim-assist for the arrow?
