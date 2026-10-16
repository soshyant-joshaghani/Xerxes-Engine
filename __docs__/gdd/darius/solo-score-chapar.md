# Chapar: Royal courier runner

Darius (2D) · **Solo x Score** · Slices 1 to 4 playable (obstacles, distance, relay stations, daily road, ghost, score: darics / multiplier / grazes)

| | |
|---|---|
| Name | The chapar was the Achaemenid royal courier relay; Herodotus praised its riders. |
| Mode | Solo x Score ([taxonomy](../../game-modes.md)), time trial / high score, endless |
| Modifiers | none |
| Camera | Side-scrolling, auto-run |
| Players | 1 (+ spectators) |
| Physics | None from `modules/physics`: the run is its own Bevy-free, deterministic step (`src/chapar/rules.rs`) |
| Scene | `games/__templates__/darius/assets/scenes/games/solo/chapar.scene.rs` |
| Build rung | R3 Side view and ghosts ([ladder](../README.md#the-build-ladder)) |
| Twin | [Rakhsh](../cyrus/solo-score-rakhsh.md) (Cyrus (3D), same mode) |

## Pitch

Carry the king's letter along the relay road. The rider runs by himself; you only jump, duck and grab fresh horses at relay stations. The road is generated from the day's seed and never ends; how far you got is the score of the day.

## Why it is this mode

No opponent and no finish line: an endless score attack. The number is accumulated (distance, a chain of darics, grazes, a multiplier) and the better you play the more it grows. That is Solo x Score. It was Survival in the first design; an endless runner ranks execution, not endurance against a hunter, so it moved when Ahriman's Horde became the endurance game.

## Reference: Vector, and what we did not take from it

[Vector](https://vector.game/) (Nekki's parkour runner) is the closest reference for the feel: one finger, a body that runs by itself, a run that ends on a mistake. Chapar takes the **auto-run, one-input-per-obstacle, speed that builds, instant retry** from it. It does **not** take parkour (vaults, wall-runs, tricks, ragdoll): a horse and a parkour body do not fit together, and animation work is out of scope until the game is fun with coloured boxes. Obstacles are therefore three clear questions, each with one answer: jump a block, duck a banner, jump a pit.

## Rules (the `Sim`)

- **State:** Rider (height above the road, vertical speed, grounded, ducking), speed, the obstacles ahead (from the seed), distance.
- **Inputs:** Jump (hold for higher, let go for a hop), duck (on the road; in the air it is a fast fall).
- **Feel:** a jump pressed up to 0.12 s before landing is kept; a jump up to 0.10 s after leaving the road still counts (coyote time); the hit box is narrower than the drawn rider, so a graze is forgiven.
- **Speed:** a function of the distance run (380 px/s, up to 760), not of a clock, so the generator knows how fast the rider will be at every obstacle and can keep each one fair.
- **Obstacles:** a block (jump), a hanging banner (duck; it cannot be jumped), a pit (jump across). The first 1800 px are blocks only. After each obstacle there is always more than 0.75 s of road to land and read the next.
- **Relay stations:** every 3 to 4 thousand pixels a station stands on the road with a fresh horse (a token) held high over it. A held jump reaches it; a hop does not. Taking it makes the next 1100 px a sprint at 1.5 x speed, which is more distance per second and tighter reading. The road after a station is made for the sprint speed in advance, so it is fair taken or missed (a pit is never wider than the road's own speed clears). Nothing stands inside a station.
- **The daily road:** the seed is the UTC date's day number, so everyone who rides today rides the same road. A retry reads the clock again (a run begun after midnight is on the new road). The header shows the date.
- **The ghost:** the best run of the day is ridden again beside you, faint, with its distance in the header. It is not a stored path: it is the run's recorded inputs (one byte per tick, run-length coded, a few KB of text) fed to a second run on the same road, so it cannot drift from what the rules do. Beating the day's best makes the new run the ghost. One slot is kept, the latest day played, so a new day starts with none. It survives a reload (engine `modules::storage`).
- **A round ends when:** a crash into a block or banner, or a fall into a pit: the run is out (a `Down` report to the engine's match).
- **Score (to add):** distance in metres (20 px each) plus darics collected, times a multiplier. A line of darics collected without a miss grows the multiplier; a miss resets it. A graze (a near-miss on a block or banner) is worth points; a relay station gives a multiplier burst on top of its sprint.
- **Result:** Finished, ranked by score; distance is shown beside it.
- **Determinism:** the same seed and the same inputs give the same run, on any device. A ghost, a replay and a daily board are all just inputs against a seed.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| Space / Up / W jump, S / Down duck; R or jump (after a moment) retries | Footer buttons: right half Jump (hold for height), left half Duck |

## Offline, online, spectate

- **Offline:** The daily seed; your best run as a ghost (done).
- **Online:** A daily-seed leaderboard.
- **Spectate:** Follow a live run; replay a board entry from its inputs.

## What it forces in the engine

- A follow / auto-scroll 2D view (done by scrolling the world past a fixed rider, because the shared layout module owns the game camera)
- A platformer controller (jump, coyote time, buffered jump). It is in the game now; extract it when Mobarezeh (the next side view) needs the same
- Obstacles streamed from a seed
- Input recording for a ghost: done in the game (`src/chapar/ghost.rs`); persistent storage was missing and is now the engine's `modules::storage`
- Daily-seed leaderboard

## Slices

1. **Done.** Flat road, jump, duck, obstacles, distance, crash and retry; seeded road laid ahead of the rider
2. **Done.** Relay stations (token, sprint), the daily seed from the date
3. **Done.** Ghost of the best run (recorded inputs, kept in the engine's storage)
4. **Done.** Score: darics in lines, the chain multiplier, grazes (rules in `games/.../chapar/`, not the backend)
5. Daily board online

## Open questions

- Horse and rider as one body (as built), or does the horse tire? Resolved for slice 1 as one body; revisit with relay stations.
- Does the sprint feel like a reward or a trap at the speed cap? Its length (1100 px) and factor (1.5) are first guesses.
- Should a missed token cost anything (it does not today: the station is simply a breather)? Or should stations also refill something (a stamina idea) so skipping one hurts?
- Should the ghost be one slot per day or the best of the last few days, and should a friend's or a leaderboard entry be ridable too (the recording is already a portable line of text)?
- Is a pit's width right near the speed cap (the generator keeps it under what a held jump clears)? Needs play-testing on a phone.
- Seeded *chunks* (hand-authored patterns picked by the seed) versus today's single obstacles laid by rule: add chunks only if single obstacles feel samey.
