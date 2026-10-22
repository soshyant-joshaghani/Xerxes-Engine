# Palang: Animal battle royale

Darius (2D) · **N-gon x Survival** · Not started: built **only when the multiplayer backend is ready**, and **pure online from day one**

| | |
|---|---|
| Name | *Palang* is Persian for the leopard; Iran's wildlife (the Persian leopard, the Asiatic cheetah, the fallow deer) are the fighters. (Name provisional.) |
| Mode | N-gon x Survival ([taxonomy](../../game-modes.md)), last one standing |
| Modifiers | ShrinkingZone (the closing gas), Permadeath |
| Camera | Top-down, following you |
| Players | 16 to 32 to start (more later), in solos, duos or squads; bots fill (+ spectators) |
| Physics | none from Rapier: character mover and casts; bullets are cheap casts |
| Screens | Local: split viewports (up to 2 x 2, as a squad). Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/darius/assets/scenes/games/ngon/palang.scene.rs` |
| Build rung | R10 Scale and live. It starts after M1 (server-authoritative state sync), with no offline-first version ([ladder](../README.md#the-build-ladder)) |
| Twin | [Sandstorm](../cyrus/ngon-survival-sandstorm.md) (Cyrus (3D), same mode) |

## Pitch

A pure battle royale in two dimensions. Dozens of animals drop onto one island, find guns and armour, and fight while a poisonous haze closes in. The last animal or team standing wins; the rest are ranked by when they fell.

## Why it is this mode

Elimination is permanent for the match and the last side standing wins: N-gon x Survival (last one standing).

## Reference

[Super Animal Royale](https://animalroyale.com/): a top-down 2D battle royale. The things we take: the drop from a flying bird, the closing gas, found guns and armour, healing, a rolling ball for fast travel, solos and squads, and last one standing. Fortnite and Call of Duty's battle royale are the general model for loot, zone phases and squads.

## Rules (the `Sim`)

- **State:** the island (terrain, buildings, cover), players (position, hp, armour, weapons, ammo, alive), loot on the ground, the gas (a circle and its phases), the flying bird and the drop, who fell when.
- **The drop:** a great bird (the *homa*) flies a line over the island; each player picks the moment to leave it and glides down.
- **The pad (as Ahriman's Horde):** Fire held = plant and shoot along the stick; Interact / Switch = pick up and use (loot, a door, a healing item) or switch weapons; Reload / Dash = dash when the weapon is loaded, reload otherwise.
- **Loot:** guns with limited ammo (the Gun add-on's data), armour tiers, healing items, ammo, a few utilities.
- **The gas:** closes in phases; standing in it costs health at a rate that rises each phase (ShrinkingZone).
- **A round ends when:** one side (a solo, a duo or a squad) is left.
- **Result:** ranked by elimination order; the winner takes first.
- **Teams:** solos, duos or squads; a downed squad member can be revived by a teammate's Interact for a short window.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD, mouse aim, hold left button Fire, E Interact / Switch, Shift or R Reload / Dash | Stick + the stack of three buttons |

## Screens

- **Local:** squad play on one device with viewports (2 side by side, up to 2 x 2). Nothing is hidden.
- **Online:** one camera per player.
- **Spectators:** a delay, so a spectator cannot feed positions to a player.

## Offline, online, spectate

- **Offline:** none first. The game is **pure online from day one**: the server runs the sim, and bots only fill empty slots in a room (they drop, loot, move with the gas and fight). A local squad on one device is a later slice, as clients of a local room.
- **Online:** a lobby that fills (with bots) to 16 to 32, then the drop.
- **Spectate:** follow anyone or the last two; replay from the seed and inputs.

## What it forces in the engine

- A large top-down map with streaming tiles
- **Interest management**: each client gets only what is near (the M5 to M6 work)
- Loot tables from a seed, the zone phases, the drop
- The Gun add-on's weapons and inventory (a third user after Ahriman's Horde and Naft)
- **The multiplayer backend itself** (rooms, state sync, a lobby that fills, interest management): this game cannot be built before it
- Many clients in one room, squads, revive
- Several viewports for local squads (later)

## Slices

1. A server room (needs M1): a small island, 8 players (people or server bots), loot, the gas
2. The drop, armour, healing; duos and revive; the lobby that fills (M2)
3. 16 to 32 players, a bigger map, interest management (M5)
4. Spectate (M3); the rolling ball for travel; local squads on one device

## Open questions

- **Decided:** it is started only when the multiplayer backend is ready, and it is online from the first slice (no offline version to port later).
- Is the first version solos only, with duos after?
- How big is the first island (and so the first player count)?
- Is the fast-travel ball in the first release?
