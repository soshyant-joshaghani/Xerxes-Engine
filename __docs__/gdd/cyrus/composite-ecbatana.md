# Ecbatana: The seven-walled city hub

Cyrus (3D) · **Composite** · Coming soon (empty scene)

| | |
|---|---|
| Name | Herodotus describes Ecbatana as ringed by seven walls, each a different colour. |
| Mode | Composite ([taxonomy](../../game-modes.md)) |
| Modifiers | per place |
| Camera | Third person, walkable |
| Players | 1 to a party (+ spectators) |
| Physics | Rapier 3D |
| Scene | `games/__templates__/cyrus/assets/scenes/games/outside/ecbatana.scene.rs` |
| Build rung | R8 3D core ([ladder](../README.md#the-build-ladder)) |
| Twin | [Persepolis](../darius/composite-persepolis.md) (Darius (2D), same mode) |

## Pitch

A city of seven coloured rings. Each ring holds a family of games; the palace in the centre holds the stories. Portals lead into online rooms.

## Why it is this mode

Free roam, missions, progression and online portals stacked: Composite.

## Rules (the `Sim`)

- **State:** Place, open rings, party state.
- **Inputs:** Walk, enter.
- **A round ends when:** Never.
- **Result:** Per part.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD + mouse | Stick + look pad |

## Offline, online, spectate

- **Offline:** You.
- **Online:** A party walks together.
- **Spectate:** Per place.

## What it forces in the engine

- Warp in 3D
- Streaming places

## Slices

1. Hub with three portals
2. All rings

## Open questions

- Rings unlock by playing, or all open?
