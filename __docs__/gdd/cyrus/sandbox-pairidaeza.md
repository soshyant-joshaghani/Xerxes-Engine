# Pairidaeza: Garden builder (and 3D physics lab)

Cyrus (3D) · **Sandbox** · Slice 0 playable (physics lab: spawn, drag, gravity)

| | |
|---|---|
| Name | Pairidaeza, the walled Persian garden, is the root of the word 'paradise'. |
| Mode | Sandbox ([taxonomy](../../game-modes.md)) |
| Modifiers | none (gravity slider in the lab) |
| Camera | Free builder camera (slice 0: fixed look at the plot) |
| Players | 1 (shared room later) (+ spectators) |
| Physics | Rapier 3D (`xerxes_sim::Physics3d`) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/outside/pairidaeza.scene.rs` |
| Build rung | R8 3D core pulled forward ([ladder](../README.md#the-build-ladder)); lab before Atash |
| Twin | [Jamshid's Workshop](../darius/sandbox-jamshids-workshop.md) (Darius (2D), same mode) |

## Pitch

An empty walled plot. **Slice 0** is the 3D physics lab twin of Jamshid's Workshop: drop cubes and spheres, drag them, tilt gravity. Later slices turn the plot into a garden (paths, water, trees, pavilions). No goal; the 3D lab where new engine features are tried first.

## Why it is this mode

No win condition: Sandbox.

## Rules (the `Sim`)

### Slice 0 (physics lab)

- **State:** Bodies, the gravity setting.
- **Inputs:** Pick a toy, drag, spawn cube/ball, gravity slider, reset.
- **A round ends when:** Never.
- **Result:** None.

### Later (garden)

- **State:** Placed pieces, water network.
- **Inputs:** Pick, place, rotate, delete, play water.
- **A round ends when:** Never.
- **Result:** None.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| Mouse drag toys; HUD buttons | Tap and drag |

## Offline, online, spectate

- **Offline:** You.
- **Online:** A shared garden, 1 to 4.
- **Spectate:** Yes.

## What it forces in the engine

- `Physics3d` / `Body3d` (pulled forward from Atash's line step)
- Free spawning in 3D
- Later: in-game 3D placement tools, saving a creation

## Slices

0. Physics lab: spawn and drag cubes/spheres, gravity slider (**done**)
1. Place and rotate garden pieces
2. Water
3. Save, share

## Open questions

- Shared gardens visitable by others (needs moderation)?
