# Atash: Keep the flame

Cyrus (3D) · **Solo x Survival** · Coming soon (empty scene)

| | |
|---|---|
| Name | Atash is fire, kept burning in Zoroastrian fire temples. |
| Mode | Solo x Survival ([taxonomy](../../game-modes.md)) |
| Modifiers | none |
| Camera | Isometric 3D, night |
| Players | 1 (+ spectators) |
| Physics | Rapier 3D (light); bullets are cheap casts |
| Scene | `games/__templates__/cyrus/assets/scenes/games/solo/atash.scene.rs` |
| Build rung | R8 3D core ([ladder](../README.md#the-build-ladder)) |
| Twin | [Ahriman's Horde](../darius/solo-survival-ahrimans-horde.md) (Darius (2D), same mode) |

## Pitch

Night falls on a hill fire temple and the divs come out of the dark. Fight them off with the **Gun in 3D** (the first proof the weapon is not 2D-only) and feed the sacred flame. **Light is your health:** you only take damage in the dark, and the flame shrinks as it starves. The run is ranked by how long the flame burned.

## Why it is this mode

Survive the game itself, ranked by time: Solo x Survival. The flame is the clock and the HP bar at once.

## Rules (the `Sim`)

- **State:** The flame's strength and light radius, the hero (position, aim, planted, inventory), divs, fuel pickups, upgrades.
- **Inputs:** Move, Fire (hold = plant and shoot), Interact / Switch, Reload / Dash, carry fuel to the flame.
- **Damage:** only outside the flame's light. Standing in light is safe; the dark drains you. Feeding the flame grows the radius; starving shrinks it.
- **A round ends when:** The flame goes out, or your health in the dark reaches 0.
- **Result:** Finished, ranked by time survived.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD, mouse aim, left button Fire, E Context, Shift / R Utility | Stick + the stack of three (same pad as Ahriman's Horde) |

## Offline, online, spectate

- **Offline:** Seeded nights; daily seed.
- **Online:** Daily board.
- **Spectate:** Follow a run; replay.

## What it forces in the engine

- The Gun in 3D (extraction candidate with Naft / Maydan)
- Instanced crowds in 3D
- Night lighting with a real light source as gameplay
- Isometric camera

## Slices

1. Temple, flame (light = safe), pistol, one div type, time
2. Fuel, more weapons, upgrades between nights
3. Daily board

## Open questions

- One hero, or pick from three?
- Does the dark drain slowly, or is one step outside a hit?
