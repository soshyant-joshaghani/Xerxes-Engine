# The Immortals: The line never breaks

Cyrus (3D) · **Coop x Survival** · Coming soon (empty scene)

| | |
|---|---|
| Name | The Immortals were the elite royal guard of the Achaemenid kings, always ten thousand strong. |
| Mode | Coop x Survival ([taxonomy](../../game-modes.md)) |
| Modifiers | Respawn (downed soldiers refill from the reserve) |
| Camera | Third person |
| Players | 2 to 4 captains (+ spectators) |
| Physics | Rapier 3D |
| Screens | Local: split viewports. Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/coop/the_immortals.scene.rs` |
| Build rung | R8 3D core ([ladder](../README.md#the-build-ladder)) |
| Twin | [Naft](../darius/coop-survival-naft.md) (Darius (2D), same mode) |

## Pitch

A mountain pass held by a formation of NPC Immortals. **The line never breaks until the reserve runs out:** downed soldiers refill from the reserve, which is the health bar. Players are captains who revive, re-form and call the line. It is command, not build-and-shoot (that is Naft). Ranked by the wave the line held.

## Why it is this mode

The team survives or wipes together: Coop x Survival (team endurance). The reserve is how long you outlast the raiders.

## Rules (the `Sim`)

- **State:** Formation of NPC soldiers, reserve count, captains, raiders on a navmesh, wave.
- **Inputs:** Move, attack, revive / re-form a gap, call (hold / advance).
- **The line:** soldiers hold a front. A gap opens when one falls; a captain can pull from the reserve to refill it. At reserve 0, gaps stay open and the line can be broken.
- **A round ends when:** The line breaks (raiders pour through) or the reserve and living soldiers are gone.
- **Result:** Finished, ranked by wave.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD, mouse attack, E revive / re-form, Q call | Stick + buttons |

## Screens

- **Local:** split viewports. **Online:** one camera per player.

## Offline, online, spectate

- **Offline:** You + bot captains.
- **Online:** Party code, 2 to 4.
- **Spectate:** Follow a captain or the overview of the line.

## What it forces in the engine

- Navmesh pathfinding in 3D
- Formation AI (hold a line, refill from a pool)
- Wave director
- Revive / re-form as a command, not a build tool

## Slices

1. Pass, a fixed formation, one enemy type, reserve as HP
2. Captains: revive, re-form, call advance/hold
3. Online

## Open questions

- Classes (spear, bow, shield) for captains, or one kit?
- Can a captain fight in the line, or only command?
