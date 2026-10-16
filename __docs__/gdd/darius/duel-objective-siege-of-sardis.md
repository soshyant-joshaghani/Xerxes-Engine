# Siege of Sardis: Lane MOBA

Darius (2D) · **Duel x Objective** · Coming soon (empty scene)

| | |
|---|---|
| Name | Cyrus took Sardis, the Lydian capital, in 546 BC; Darius later ruled it as a satrapy. |
| Mode | Duel x Objective ([taxonomy](../../game-modes.md)), base destruction |
| Modifiers | Respawn (grows with level), no time limit |
| Camera | Top-down, following your hero |
| Players | 2 teams, 1v1 up to 3v3 (5v5 later) (+ spectators) |
| Physics | none from Rapier: character mover and casts |
| Screens | Local: split viewports, one per local player. Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/darius/assets/scenes/games/duel/siege_of_sardis.scene.rs` |
| Build rung | R5 Crowds, paths and objectives ([ladder](../README.md#the-build-ladder)) |
| Twin | [Derafsh](../cyrus/duel-objective-derafsh.md) (Cyrus (3D), same mode) |

## Pitch

League of Legends and Dota 2 on a phone and a stick. Two citadels at opposite ends of a long map, lanes between them, waves of foot soldiers marching down each lane, towers along the way. You are a hero. Kill soldiers for darics, buy items at your keep, level up, win a lane, push the towers and take down the enemy keep. You can lose every fight and still win by the keep.

## Why it is this mode

The objective is the only thing that counts: Duel x Objective (base destruction, the MOBA nexus).

## Rules (the `Sim`)

- **State:** two keeps (HP), towers per lane, soldier waves (spawn on a timer, march a lane), neutral camps, heroes (level, xp, items, cooldowns, dead timer), darics, the match.
- **The pad:** Fire = basic attack (hold to aim and plant), Interact / Switch = the hero's ability (and, near the shop or a camp, interact), Reload / Dash = dash (or the ultimate from level 6). A hero has an attack, one ability, a dash.
- **Lanes:** three lanes in the full map; the first slice is one lane, 1v1.
- **Economy:** darics from soldiers and kills, spent at the keep's shop on items (data). Levels from lane experience.
- **Towers:** hit the nearest enemy unit; a hero in range of its own tower is protected; a tower dies before the keep can be hurt.
- **Respawn:** a downed hero returns at its keep after a delay that grows with level.
- **A round ends when:** a keep falls.
- **Result:** ranked: the team whose keep stands.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD, mouse aim, left button Fire, E ability, Shift dash | Stick + the stack of three buttons |

## Screens

- **Local:** viewports, one per local player (2 side by side, up to 2 x 2).
- **Online:** one camera per player.
- No hidden information in the first version (fog of war is a later modifier).

## Offline, online, spectate

- **Offline:** you plus **bots** that last-hit in their lane, retreat when low and push when ahead; bots fill both teams.
- **Online:** a queue, 1v1 to 3v3.
- **Spectate:** an overview of the map, or follow a hero; replay from the inputs.

## What it forces in the engine

- Wave director, minions and towers as AI with targeting rules
- Items, shops and heroes as data; levels and stats
- Many cheap units (instancing), one camera per player
- Objective HP and Respawn that scales

## Slices

1. One lane, 1v1, a hero with an attack and a dash, soldiers, one tower, the keep
2. Items and the shop; level-up; the ability
3. Three lanes, camps, 3v3, bots on both sides
4. Online; spectate

## Open questions

- How many abilities fit three buttons? (Assumed an attack, one ability, one dash, the ultimate replacing the dash.)
- Should a first version use a fixed hero roster, or let a team pick a class?
