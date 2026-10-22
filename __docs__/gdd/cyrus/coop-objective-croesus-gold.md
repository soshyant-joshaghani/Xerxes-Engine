# Croesus' Gold: Two-person litter

Cyrus (3D) · **Coop x Objective** · Coming soon (empty scene)

| | |
|---|---|
| Name | Croesus of Lydia, proverbially rich, lost Sardis to Cyrus. |
| Mode | Coop x Objective ([taxonomy](../../game-modes.md)) |
| Modifiers | Escort (the litter) |
| Camera | Third person |
| Players | 2 to 4 (+ spectators); at least two to carry |
| Physics | Rapier 3D |
| Screens | Local: split viewports. Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/coop/croesus_gold.scene.rs` |
| Build rung | R8 3D core ([ladder](../README.md#the-build-ladder)) |
| Twin | [Bound Bros](../darius/coop-objective-bound-bros.md) (Darius (2D), same mode) |

## Pitch

Get into the treasury of Sardis, take the gold and get out. **The gold is carried on a litter by two people**, who have to move in step; the rest of the crew clears the way. The litter is slower, loud and hard to climb with. Clear time ranks the crew.

## Why it is this mode

Succeed or fail as one unit, ranked by clear time: Coop x Objective (shared clear). No single player can carry alone.

## Rules (the `Sim`)

- **State:** Crew, guards, gates, the litter (two carriers, sync, noise), the gold on it.
- **Inputs:** Move, jump, interact, grab / release a litter pole.
- **Litter:** needs two grabbers. If the carriers' move inputs diverge past a threshold, the litter stalls or tips. Climbing is blocked while carrying. Noise alerts nearby guards.
- **Crew:** non-carriers open gates, distract guards and clear the path.
- **A round ends when:** The litter reaches the exit (win) or the whole crew is caught / the litter is abandoned too long.
- **Result:** Won (time) or Lost.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD, Space, E grab / interact | Stick + buttons |

## Screens

- **Local:** split viewports, one per local player.
- **Online:** one camera per player.

## Offline, online, spectate

- **Offline:** You + bot crew, or local split-screen (bots fill a litter pole if needed).
- **Online:** Party code, 2 to 4.
- **Spectate:** Follow any crew member or the litter.

## What it forces in the engine

- Third-person character controller and camera
- A two-player carryable with sync and noise
- Local split-screen

## Slices

1. One vault room, two on the litter, exit
2. Guards, gates, a third/fourth clearer
3. Online

## Open questions

- Stealth (guards see you) or a pure route puzzle?
- Can bots hold a pole well enough for offline solo practice?
