# Pasargadae Depths: Explorer and guide

Cyrus (3D) · **Coop x Role** · Coming soon (empty scene)

| | |
|---|---|
| Name | Pasargadae was Cyrus's capital, where his tomb still stands. |
| Mode | Coop x Role ([taxonomy](../../game-modes.md)) |
| Modifiers | FogOfWar (the explorer sees only the torch) |
| Camera | First person for one, a 2D map for the other |
| Players | 2 (+ spectators) |
| Physics | Rapier 3D (explorer) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/coop/pasargadae_depths.scene.rs` |
| Build rung | R9 First person ([ladder](../README.md#the-build-ladder)) |
| Twin | [Behistun](../darius/coop-role-behistun.md) (Darius (2D), same mode) |

## Pitch

Below Pasargadae runs a labyrinth. The Explorer walks it in first person with only a torch. The Guide sees the whole maze as a 2D map with the traps, but not the Explorer's view. Talk them through.

## Why it is this mode

Different views, different jobs, neither can do it alone: Coop x Role.

## Rules (the `Sim`)

- **State:** The maze, traps, the explorer's position, torch fuel.
- **Inputs:** Explorer: walk, look, use. Guide: mark the map (pings).
- **A round ends when:** The Explorer reaches the chamber (win) or the torch dies.
- **Result:** Won or Lost, with the time.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| Explorer: WASD + mouse; Guide: mouse | Explorer: stick + look pad; Guide: tap |

## Offline, online, spectate

- **Offline:** You + a bot guide (reads out turns) or a bot explorer.
- **Online:** Party code, 2.
- **Spectate:** Live: the map only, or neither. Replay: both.

## What it forces in the engine

- A 3D view and a 2D view of the same match, one per player
- First-person camera and pointer lock
- Private views in 3D

## Slices

1. Maze, explorer, guide map, hot-seat split screen
2. Traps and pings
3. Online

## Open questions

- Does the Guide see the Explorer's position live or only last ping?
