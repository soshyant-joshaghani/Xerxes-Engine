# Nowruz Feast: Coop kitchen

Cyrus (3D) · **Coop x Score** · Coming soon (empty scene)

| | |
|---|---|
| Name | Nowruz is the Persian New Year, celebrated with a great family feast. |
| Mode | Coop x Score ([taxonomy](../../game-modes.md)) |
| Modifiers | TimeLimit 4 min |
| Camera | Fixed angled camera over a kitchen |
| Players | 2 to 4 (+ spectators) |
| Physics | Rapier 3D (carryables) |
| Screens | Local: one shared kitchen camera. Online: same shared camera per device ([Screens](#screens)) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/coop/nowruz_feast.scene.rs` |
| Build rung | R8 3D core ([ladder](../README.md#the-build-ladder)) |
| Twin | [Tombak Duo](../darius/coop-score-tombak-duo.md) (Darius (2D), same mode) |

## Pitch

It is Nowruz and the guests are hungry. Chop herbs, cook rice, fry fish and plate the feast before each order expires. A **bonfire in the middle of the floor (Chaharshanbe Suri)** burns anything dropped near it. Passing a dish to a teammate across the counter earns a bonus.

## Why it is this mode

One shared score, rewarding plays that need two: Coop x Score.

## Rules (the `Sim`)

- **State:** Stations, ingredients, plates, orders, the bonfire (burn radius), shared score.
- **Inputs:** Move, pick up / put down, use a station, throw.
- **Bonfire:** anything dropped or thrown into its radius is destroyed (ingredient, plate, or a teammate's carry). It is the hazard that makes the middle of the floor matter.
- **Hand-off:** passing a dish across the counter (not across the fire) earns a score bonus.
- **A round ends when:** The time limit.
- **Result:** Finished, with the shared score.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| P1 WASD + Space/E, P2 arrows + Enter/Shift | Stick + two buttons |

## Screens

- **Local:** one shared kitchen camera (Overcooked-style).
- **Online:** the same shared framing on each device.

## Offline, online, spectate

- **Offline:** You + bot cooks (follow the order list), or couch play.
- **Online:** Party code, 2 to 4.
- **Spectate:** The kitchen camera.

## What it forces in the engine

- Pick up / carry / interact in 3D
- A fixed camera framing
- Couch coop in 3D
- A hazard volume that destroys carryables

## Slices

1. One kitchen, one recipe, the bonfire
2. Orders, hand-off bonus
3. Online

## Open questions

- Moving kitchens (a boat, a cart) as later stages?
