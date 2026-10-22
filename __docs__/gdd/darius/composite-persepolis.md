# Persepolis: The city hub

Darius (2D) · **Composite** · Coming soon (empty scene)

| | |
|---|---|
| Name | Persepolis was the ceremonial capital Darius I began. |
| Mode | Composite ([taxonomy](../../game-modes.md)) |
| Modifiers | per place |
| Camera | Top-down walkable city |
| Players | 1 to a party (+ spectators) |
| Physics | kinematic |
| Scene | `games/__templates__/darius/assets/scenes/games/outside/persepolis.scene.rs` |
| Build rung | R7 Tools (Warp, save, quests, dialogue) ([ladder](../README.md#the-build-ladder)) |
| Twin | [Ecbatana](../cyrus/composite-ecbatana.md) (Cyrus (3D), same mode) |

## Pitch

Walk the terraces of Persepolis. **Every gate is a contract board**: a Qanat room, an Ahriman's Horde courtyard, a Chapar relay, a teahouse tale, a portal to Maydan. Currency and gear persist. The bazaar holds Caravanserai, the teahouse the Naqqal, the workshop Jamshid. It is the walk-in version of the menu, and the Warp tool at full size.

## Why it is this mode

Several kinds stacked: free roam, missions, progression and online portals. Composite.

## Rules (the `Sim`)

- **State:** The party's place, open gates (locks), contract boards per gate, party-wide currency and gear.
- **Inputs:** Walk, enter a gate, take a contract.
- **A round ends when:** Never (its parts end).
- **Result:** Per part; gear and darics persist.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD, E enter | Stick + button |

## Offline, online, spectate

- **Offline:** You.
- **Online:** A party walks together; gates open rooms.
- **Spectate:** Per place.

## What it forces in the engine

- Warp at full size: places, ports, links, locks, params (stage 3.7)
- Mixing modes in one game
- Online Warp (M4)

## Slices

1. Hub with three gates; each gate shows a contract into an existing scene
2. Locks from quests; currency and gear persist
3. Party online; portal to Maydan

## Open questions

- Should the hub replace the grid menu, or sit beside it?
