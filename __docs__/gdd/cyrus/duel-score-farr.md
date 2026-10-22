# Farr: Carry the glory

Cyrus (3D) · **Duel x Score** · Coming soon (empty scene)

| | |
|---|---|
| Name | The farr (khvarenah) is the divine glory that clings to a rightful king. |
| Mode | Duel x Score ([taxonomy](../../game-modes.md)), head-to-head score |
| Modifiers | Respawn 5 s, TimeLimit 10 min, MovingObjective (the farr relocates) |
| Camera | Third person |
| Players | 2 teams, 2v2 up to 4v4 (+ spectators) |
| Physics | Rapier 3D (character mover, sensors, pickups) |
| Screens | Local: split viewports, one per local player. Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/duel/farr.scene.rs` |
| Build rung | R8 3D core ([ladder](../README.md#the-build-ladder)) |
| Twin | [Sofreh](../darius/duel-score-sofreh.md) (Darius (2D), same mode) |

## Pitch

One glowing relic sits in the middle of the arena. Pick it up and your team scores every second you hold it. The carrier can't fight, so the team must guard them; everyone else sees a beam rising from the carrier and comes for it. The most time held when the clock ends wins.

## Why it is this mode

The number is how long a team held the one contested object, not kills: Duel x Score (the "Halo skull carry").

## Rules (the `Sim`)

- **State:** the farr (position, carrier, dropped timer), each player, the two teams' held time, the clock.
- **The carrier** is **unarmed** (hands full): they can sprint and dash, and shove, but cannot use a weapon. A beam marks them for everyone.
- **A hit that downs the carrier drops the farr.** After a second anyone may pick it up. If it lies untouched for 10 s it **relocates** to a new hill, so it cannot be camped.
- **Scoring:** one point per second held, the team's total.
- **Weapons** from pickups for everyone else (the Gun's data); the pad, as in Darius, plus a look pad on touch.
- **A round ends when:** a team reaches the held-time target, or the clock ends with the higher total.
- **Result:** ranked: the winning team.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD + mouse, Fire, E Interact / Switch, Shift Reload / Dash | Stick + look pad + the stack of three buttons |

## Screens

- **Local:** viewports, one per local player (2 side by side, up to 2 x 2).
- **Online:** one camera per player. Nothing is hidden.

## Offline, online, spectate

- **Offline:** you plus **bots**: a carrier bot runs to cover, escorts that stay near, hunters that head for the beam.
- **Online:** a quick queue, 2v2 to 4v4.
- **Spectate:** an overview, or follow the carrier; replay from the inputs.

## What it forces in the engine

- A single contested objective with a carrier state in the match rules (MovingObjective)
- Third-person controller, pickups, respawn
- Several viewports in a 3D scene

## Slices

1. One arena, the farr, two players, hold time scored
2. Weapons, the beam, respawn, relocation
3. Bots; 4v4
4. Online; spectate

## Open questions

- Can the carrier throw the farr to a teammate?
- Is the carrier slowed, or does the beam alone make them the target?
