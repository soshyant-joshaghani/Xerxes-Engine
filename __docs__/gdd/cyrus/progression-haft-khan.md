# Haft Khan: Seven labours

Cyrus (3D) · **Progression** · Coming soon (empty scene)

| | |
|---|---|
| Name | The Haft Khan are the seven labours of Rostam in the Shahnameh. |
| Mode | Progression ([taxonomy](../../game-modes.md)) |
| Modifiers | none |
| Camera | Third person |
| Players | 1 (+ spectators) |
| Physics | Rapier 3D |
| Scene | `games/__templates__/cyrus/assets/scenes/games/outside/haft_khan.scene.rs` |
| Build rung | R8 3D core ([ladder](../README.md#the-build-ladder)) |
| Twin | [Caravanserai](../darius/progression-caravanserai.md) (Darius (2D), same mode) |

## Pitch

Ride with Rostam and his horse Rakhsh through seven labours: the lion, the desert, the dragon, the witch, the warrior, the div army, the White Div. Gear, levels and Rakhsh's training carry over from run to run.

## Why it is this mode

Progressed, not won: levels, gear, unlocks. Progression.

## Rules (the `Sim`)

- **State:** Hero stats, gear, labours unlocked, Rakhsh's skills.
- **Inputs:** Move, attack, dodge, ride.
- **A round ends when:** Never (each labour ends).
- **Result:** Persistent progress.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD + mouse | Stick + buttons |

## Offline, online, spectate

- **Offline:** You; saved.
- **Online:** Cloud save.
- **Spectate:** Watch a run.

## What it forces in the engine

- Animation state machines
- Stats and gear on top of quests and save (3.6, 3.8)

## Slices

1. First labour, gear, save
2. Three labours
3. All seven

## Open questions

- Each labour a different mini-genre?
