# Atashkadeh: Plant and defuse

Cyrus (3D) · **Duel x Role** · Coming soon (empty scene, still named `div_e_sepid.scene.rs`; renamed in the batch fix)

| | |
|---|---|
| Name | An atashkadeh is a fire temple, where the sacred flame is kept. |
| Mode | Duel x Role ([taxonomy](../../game-modes.md)), attacker vs defender |
| Modifiers | Permadeath (one life per round), TimeLimit (the round and the fuse) |
| Camera | First person |
| Players | 2 teams, 2v2 up to 5v5 (+ spectators) |
| Physics | Rapier 3D (hitscan and projectile casts) |
| Screens | Local: split viewports (up to 2 x 2). Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/duel/atashkadeh.scene.rs` |
| Build rung | R9 First person ([ladder](../README.md#the-build-ladder)) |
| Twin | [Karvan](../darius/duel-role-karvan.md) (Darius (2D), same mode) |

## Pitch

Counter-Strike's search and destroy, in a fire temple. One team are the **raiders**: carry a jar of naphtha to one of two altars and set it alight. The other team are the **guardians**: keep them away, or douse the fire before it takes. One life each round, buy your weapons between rounds, swap sides at half time.

## Why it is this mode

Two teams with different jobs and different win terms (plant and hold, or stop and defuse), sides swapped to keep it fair: Duel x Role (attacker vs defender, search and destroy).

## Rules (the `Sim`)

- **State:** the map (two altar sites, spawns, cover), each player (hp, weapons, darics, alive), the jar and its carrier, the lit fuse, the round clock, the score.
- **The round:** a buy phase, then about 115 seconds of play. **One life**.
- **Raiders:** carry the jar (one player has it, it can be dropped and picked up), reach an altar and **plant** it (hold Interact about 4 s, vulnerable). The fire then burns on a fuse of about 35 s.
- **Guardians:** kill the raiders, or **douse** the lit altar (hold Interact about 5 s; faster with a kit bought).
- **A round ends when:** the altar burns (raiders win), the fire is doused (guardians win), all raiders are down before planting (guardians), all guardians are down (raiders, and a fire planted may still be doused), or the clock ends with no plant (guardians).
- **Economy:** darics earned by rounds won, kills and the plant; spent in the buy phase on weapons and kits.
- **A match is:** first to 7 rounds, sides swap after 6.
- **Result:** ranked: the team that wins the match.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD + mouse, left button Fire, E Interact (plant, douse, buy), R reload, Shift dash, Q switch | Stick + look pad + the stack of three buttons |

## Screens

- **Local:** viewports, one per local player; a shared team HUD for the economy.
- **Online:** one camera per player.
- **Spectators** are delayed so a spectator cannot feed positions to a player.

## Offline, online, spectate

- **Offline:** you plus **bots** that use the navmesh, hold angles, plant and defuse.
- **Online:** a queue, 2v2 to 5v5.
- **Spectate:** delayed live view, then a full replay.

## What it forces in the engine

- The first-person controller, hitscan and lag compensation (Phase 5)
- Round state machine, economy, buy phase, plant and defuse objects
- Sound as information (footsteps) and line of sight
- Bots on a navmesh

## Slices

1. One map, two altars, two local players, plant and defuse
2. Rounds, economy, the buy phase, side swap
3. Bots
4. Online 2v2 to 5v5; delayed spectating

## Open questions

- Footsteps as audible information, or visual only?
- Is the first version 2v2 so a match is small enough for one phone?
