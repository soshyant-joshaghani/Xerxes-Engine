# Derafsh: Heroes of the standard

Cyrus (3D) · **Duel x Objective** · Coming soon (empty scene)

| | |
|---|---|
| Name | Derafsh Kaviani was the great banner of Persian kings, raised by the smith Kaveh. |
| Mode | Duel x Objective ([taxonomy](../../game-modes.md)), base destruction |
| Modifiers | Respawn (grows with level) |
| Camera | Third person, over the shoulder |
| Players | 2 teams, 1v1 up to 3v3 (5v5 later) (+ spectators) |
| Physics | Rapier 3D (character mover, skill shots) |
| Screens | Local: split viewports (up to 2 x 2). Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/duel/derafsh.scene.rs` |
| Build rung | R8 3D core ([ladder](../README.md#the-build-ladder)) |
| Twin | [Siege of Sardis](../darius/duel-objective-siege-of-sardis.md) (Darius (2D), same mode) |

## Pitch

Smite on a stick and a look pad. Third-person heroes from the Shahnameh, a lane between two bases, waves of soldiers, towers, and at each end a great standard. Level up, buy items, aim your skill shots over the shoulder, and tear down the other team's Derafsh. **Pushing the enemy standard lights you up on every map**, so the final attack is a hunt as well as an objective.

## Why it is this mode

The objective is the only thing that counts: Duel x Objective (base destruction). You can lose every fight and win by the standard.

## Rules (the `Sim`)

- **State:** two standards (HP), towers, soldier waves, camps, heroes (level, items, cooldowns, dead timer, marked), darics, the match.
- **Heroes:** a class with a basic attack, abilities that are **aimed skill shots in 3D** and an ultimate. The pad's three buttons plus the look pad: attack, one ability, a dash; more abilities as a hero levels.
- **Map:** the first version is **one lane**, 3v3 (Smite's Joust); the three-lane map follows.
- **Economy, towers, respawn:** as in [Siege of Sardis](../darius/duel-objective-siege-of-sardis.md), the Darius twin.
- **Lit up:** any hero inside the enemy standard's attack range is marked on every ally and enemy minimap until they leave.
- **A round ends when:** a standard falls.
- **Result:** ranked: the team whose standard stands.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD + mouse, left button attack, E / Q / R abilities, Shift dash | Stick + look pad + the stack of three buttons |

## Screens

- **Local:** viewports, one per local player. **Online:** one camera per player. Nothing is hidden at first.

## Offline, online, spectate

- **Offline:** you plus **bots** (last-hit, retreat, push), filling both teams.
- **Online:** a queue, 1v1 to 3v3.
- **Spectate:** an overview or follow a hero; replay.

## What it forces in the engine

- Navmesh minions, towers, wave director in 3D
- Skill shots (projectiles and area casts) with hit detection
- Heroes, items and shops as data; one camera per player

## Slices

1. One lane, 1v1, a hero with an attack and a dash, soldiers, a standard
2. Skill shots, items, level-up
3. Three lanes, camps, 3v3, bots
4. Online; spectate

## Open questions

- Targeted or untargeted skills first? (Skill shots assumed, as Smite.)
- Fixed hero roster at first?
