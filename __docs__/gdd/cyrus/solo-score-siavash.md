# Siavash: Endless run and attack

Cyrus (3D) · **Solo x Score** · Slice 1 playable (the run on three lanes, jump, roll, dive, enemies on the ground and in the air, distance and kills, crash and retry); the daily road, ghost and board come later

| | |
|---|---|
| Name | Siavash is the prince of the Shahnameh who walked through fire unharmed to prove he was innocent. |
| Mode | Solo x Score ([taxonomy](../../game-modes.md)), endless, high score |
| Modifiers | none |
| Camera | Third-person, behind the knight, fixed: it looks straight down the road and does not turn when he changes lane. **The run goes toward +Z** (a glTF model's own front axis), so the camera stands at -Z and the knight's model needs no turning; the player's right, and the screen's, is -X |
| Players | 1 (+ spectators) |
| Pacing | The run steps on a fixed 60 Hz clock through `Pace` (in the rules crate): one frame never adds more than 0.05 s, so a slow frame — the first ones of a level, a phone that stalls — cannot fast-forward the knight into something he never saw, and the run waits for the road to be laid |
| Physics | None in the rules: lanes, jump, roll, dive and attacks are a fixed step (the Bevy-free `rules/` crate, `cyrus_rules::siavash`), 3D only in the view |
| Scene | `games/__templates__/cyrus/assets/scenes/games/solo/siavash.scene.rs`, built from prefabs: `prefabs/games/solo/siavash/` (`knight` — the `.glb` character, `goon`, `flyer`, `road_tile`, `ground`) and the game's global `header` / `content` / `banner` |
| Build rung | R3 Side view and ghosts, as the 3D camera rig ([ladder](../README.md#the-build-ladder)) |
| Twin | [Chapar](../darius/solo-score-chapar.md) (Darius (2D), same mode) |

## Pitch

A road out of Ecbatana, three lanes, and a knight who keeps running. Enemies come at him in every lane: goons on the road and flyers in the air. You cannot just run into them. Switch into a goon's lane at the exact moment it is close and you strike it; roll (down on the ground) or dive (down in the air) onto a goon in your lane and you strike it; jump through a flyer's lane and you strike it. Run straight into a goon with no change of direction and the run is over. Roll under a flyer to pass it. The score is distance and the enemies you strike down.

## Reference: Blades of Brim

Blades of Brim (Sybo Games, the studio behind Subway Surfers) is the reference for the play: an endless three-lane runner where the hero jumps, rolls and wall-runs, and kills by **direction**. A sideways switch into an enemy's lane, a forward roll attack, and a jump onto flying enemies all count as attacks; a runner who keeps straight into a goon loses. Siavash takes that attack-by-direction loop with a **gun for the blade**: the attack is a strike at the moment of direction, and the gun is what it looks like. He does **not** take the melee combo chain, the dungeons, the bosses or the upgrades (later). Wall-running is not in slice 1.

Sources: [Blades of Brim on Touch Arcade](https://toucharcade.com/games/blades-of-brim), [a review on Vulcan Post](https://vulcanpost.com/345881/blades-of-brim-mobile-game-review/), [Adweek on the launch](https://www.adweek.com/socialtimes/subway-surfers-creator-launches-blades-of-brim-on-ios/621255).

## Why it is this mode

No finish line and no opponent: an endless score attack, like Chapar in 2D. The number grows with distance and with the enemies struck down, and the better you read the road the faster it grows. That is Solo x Score in 3D.

## Rules (the `Sim`)

- **State:** Knight (lane 0 to 2, height, vertical speed, rolling, diving), speed, the road ahead (enemies from the seed), kills, distance.
- **Inputs:** Lane (-1, 0, +1), jump, down. Down on the ground rolls; down in the air dives. Five values per tick, so a replay is a short line of text.
- **Lanes:** a switch is instant in the rules; the view moves the knight across the screen.
- **Jump:** 1.28 times the first version's height: an apex of 1.63 m on the 60 Hz clock (the first was 1.275 m). A jump pressed up to 0.12 s before landing is kept; one pressed up to 0.10 s after leaving the road still counts.
- **Goons:** stand on the road in one lane, 1.0 m tall. A knight whose feet are below its top meets it and the run ends; a jump above its top clears it (no strike, no crash).
  - **Roll:** a roll on the ground strikes a goon in your lane within 4 m ahead, for as long as the roll lasts. No jump is needed.
  - **Lane switch:** switching into a goon's lane while it is within 3 m ahead strikes it. Switching early (more than 3 m ahead) does not, and the goon is then hit.
  - **Dive:** diving (down in the air) while in its lane and within 4 m ahead strikes it.
- **Flyers:** hang over one lane, from 1.54 m up to 2.14 m (1.28 times the first version, and drawn up in the sky to match). A knight in the air when he passes its lane strikes it down; a knight on the road meets it standing and the run ends; a rolling one passes under.
- **Speed:** a function of the distance run (14 m/s rising to 26 m/s), not of a clock, so the generator keeps every enemy fair at the speed the knight will have there.
- **Road:** one enemy, or two in different lanes, per row; the next row comes after the time to react (a fixed minimum plus 0.9 s at the speed there). The first 30 m are clear.
- **A round ends when:** the knight runs into a goon in his lane without a strike, or is hit by a flyer he did not strike or roll under.
- **Result:** distance and kills; ranked by score (metres plus kills), the distance as the tiebreak.
- **Determinism:** the same seed and inputs give the same run on any device. A ghost, a replay and a daily board are inputs against a seed.

## Controls

| Keyboard / controller | Mouse or touch |
|---|---|
| A / D or Left / Right change lane; W jump; S / Down roll (on the ground) or dive (in the air); Space special ability (not built yet); R run again; the controller stick and buttons do the same | Drag on the content: sideways changes lane, up jumps, down rolls or dives. The header holds the distance, the hints and Back; there is no footer |

## Offline, online, spectate

- **Offline:** the daily seed (the same road for everyone today); your best run as a ghost (step 6).
- **Online:** a daily-seed board, the same as Chapar's.
- **Spectate:** follow a live run from the chase camera; replay a board entry from its inputs.

## What it forces in the engine

- A 3D camera rig that stays fixed behind the lane (the first camera rig in Cyrus)
- A lane model in a fixed step: the lanes, the jump arc, the roll and the dive are data, drawn in 3D
- Enemies streamed from a seed, laid ahead of the knight, and attacked by direction
- Kills and the distance as plain state (no physics)
- Ghosts: Chapar's recording moves into `xerxes_sim` at step 6 (second use)

## Slices

1. **Built:** three lanes; jump (1.28 times the first height), roll and dive; goons and flyers; attacks by direction (lane switch, dive, jump); distance and kills; crash and retry; a fixed chase camera
2. Coins, ammo pickups and the power-ups; the score with its multiplier (**wall-running moved to a later slice: the road has no walls yet, decided 2026-10-22**)
3. The gun as the strike's look, the combo chain for consecutive kills, the enemy kinds (shielded goons take two strikes)
4. Ghost of the best run (step 6)
5. Daily board online

## Open questions

- Three lanes or five? Three reads better on a phone; five is the Subway default.
- Does a strike need its own button on touch, or is a swipe toward the enemy enough (as drawn above)?
- Should a flyer also be rollable from its own lane, or only jumped and passed by a lane change?
- Does the run slow down after a strike, or keep the same speed (as drawn)?
