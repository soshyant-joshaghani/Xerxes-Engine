# Arzhang: Ink war

Cyrus (3D) · **N-gon x Objective** · Coming soon (empty scene)

| | |
|---|---|
| Name | The *Arzhang* was the painted book of Mani, the Persian prophet and painter. (Name provisional.) |
| Mode | N-gon x Objective ([taxonomy](../../game-modes.md)), multi-side control |
| Modifiers | TimeLimit 4 min, MovingObjective (a golden mark) |
| Camera | Third person |
| Players | 3 or 4 teams of 1 or 2 (+ spectators) |
| Physics | Rapier 3D (character mover); the ink is a layer of paintable cells on floors and walls, not a physics body |
| Screens | Local: split viewports (2 or up to 2 x 2). Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/ngon/arzhang.scene.rs` |
| Build rung | R8 3D core ([ladder](../README.md#the-build-ladder)) |
| Twin | [Kashi](../darius/ngon-objective-kashi.md) (Darius (2D), same mode) |

## Pitch

Splatoon on the walls of a Persian palace. Three or four teams paint floors, walls and ceilings with ink; dive into your own ink to swim, climb walls and refill; the team that covers the most when the clock ends wins. The arenas are the same four layouts as Kashi, built in 3D.

## Why it is this mode

Several teams fight over one shared area and the leader is ganged up on: N-gon x Objective (multi-side control). It is the same game as Kashi in 3D, which is the point of a twin.

## References

- **Splatoon (Tricolor Turf War):** the ink, the swim, the wall climbing, three teams.
- **Crash Bash's Pogo Pandemonium:** the arena layouts it shares with Kashi.

## Rules (the `Sim`)

- **State:** the paintable cells and their owners, each fighter (position, tank, swimming, alive), the golden mark's cell, the clock.
- **Inputs (the Cyrus pad):** stick and look pad; **Fire** shoots ink (hold to keep firing, free to move); **Utility** swims in your own ink (fast, refills the tank, climbs ink-covered walls); **Context** uses a special when its meter is full (a burst of ink).
- **Enemy ink** slows you and cannot be swum in; a direct hit sends a fighter back to base for a short delay.
- **The golden mark** wanders over the paintable cells; the team holding it counts it three times.
- **A round ends when:** the clock reaches zero.
- **Result:** ranked by cells held.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD + mouse, left button Fire, Shift swim, E special | Stick + look pad + the stack of three buttons |

## Screens

- **Local:** split viewports, 2 or up to 2 x 2. **Online:** one camera per player. Nothing is hidden.

## Offline, online, spectate

- **Offline:** you plus **bots** that paint, swim to refill and chase the mark.
- **Online:** a lobby of 3 or 4 teams.
- **Spectate:** an overview with the cell counts; replay.

## What it forces in the engine

- Painting a 3D surface as a grid of owned cells (with a cheap mesh or decal view)
- Swim and wall-climb states in the third-person controller
- Several viewports in a 3D scene
- Four 3D arena layouts as data

## Slices

1. One arena, the floor only, two local players, ink and the clock
2. Swim, refill, walls
3. The special, the golden mark, more layouts
4. Bots; online lobby; spectate

## Open questions

- How many cells per surface (so the painting reads as ink and stays cheap)?
- Is wall climbing in the first release?
