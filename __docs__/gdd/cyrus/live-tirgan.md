# Tirgan: 24/7 archery arena

Cyrus (3D) · **Live session** · Coming soon (empty scene)

| | |
|---|---|
| Name | Tirgan is the festival of Arash the Archer, whose arrow set Iran's border. |
| Mode | Live session ([taxonomy](../../game-modes.md)) |
| Modifiers | Respawn |
| Camera | First person |
| Players | drop-in, up to 16 (+ spectators) |
| Physics | Rapier 3D (arrows) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/outside/tirgan.scene.rs` |
| Build rung | R10 Scale and live ([ladder](../README.md#the-build-ladder)) |
| Twin | [Maydan](../darius/live-maydan.md) (Darius (2D), same mode) |

## Pitch

An always-open archery ground. Drop in, loose arrows, drop out; you are ranked by Glicko-2 rating. Its archery is its own (the first user of a first-person bow).

## Why it is this mode

N-gon Score as a 24/7 room: Live session.

## Rules (the `Sim`)

- **State:** Archers, arrows, ratings.
- **Inputs:** Move, look, draw and loose, dash.
- **A round ends when:** Never.
- **Result:** Rating per kill.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD + mouse | Stick + look pad + fire |

## Offline, online, spectate

- **Offline:** Practice vs bots.
- **Online:** The always-on room.
- **Spectate:** Yes.

## What it forces in the engine

- Persistent 3D rooms
- Archery (draw, flight time, drop) as an add-on of its own

## Slices

1. Practice
2. Always-on room
3. Ratings

## Open questions

- Same ratings pool as the other first-person games, or its own?
