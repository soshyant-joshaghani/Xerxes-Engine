# Jamshid's Workshop: Physics toy table

Darius (2D) · **Sandbox** · Slice 1 playable (spawn, drag, gravity slider)

| | |
|---|---|
| Name | Jamshid is the mythic Persian king credited with inventing crafts and tools. |
| Mode | Sandbox ([taxonomy](../../game-modes.md)) |
| Modifiers | LowGravity (a slider) |
| Camera | Side view of a workbench |
| Players | 1 (shared room later) (+ spectators) |
| Physics | Rapier (heavy) |
| Scene | `games/__templates__/darius/assets/scenes/games/outside/jamshids_workshop.scene.rs` |
| Build rung | R1 Rules without physics ([ladder](../README.md#the-build-ladder)) |
| Twin | [Pairidaeza](../cyrus/sandbox-pairidaeza.md) (Cyrus (3D), same mode) |

## Pitch

A workbench full of toys: crates, balls, springs, ramps, ropes. Drop them, stack them, tilt the gravity, see what breaks. Beside it, **a weapon and enemy range**: spawn any [Ahriman's Horde](solo-survival-ahrimans-horde.md) weapon or enemy and tune numbers live. A sandbox is a practice range; this one is also how the shooter gets balanced. No goal. Every new physics feature of the engine is tried here first.

## Why it is this mode

No win condition at all: Sandbox.

## Rules (the `Sim`)

- **State:** Bodies, joints, the gravity setting; optional range (spawned weapons, enemies, live tunables).
- **Inputs:** Pick a toy, drop, drag, delete, gravity slider, reset; on the range: spawn a weapon or enemy, edit its data, fire the pad.
- **A round ends when:** Never.
- **Result:** None.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| Mouse | Tap and drag |

## Offline, online, spectate

- **Offline:** You.
- **Online:** A shared room, 1 to 4.
- **Spectate:** Yes.

## What it forces in the engine

- Free spawning
- LowGravity
- A lab scene for new engine features
- Live edit of weapon / enemy data (the Gun's balance desk)

## Slices

1. **Done.** Spawn and drag toys, gravity slider
2. Springs and joints
3. Weapon and enemy range (Ahriman's Horde data)
4. Shared room online

## Open questions

- Save a contraption?
