# Sandstorm: Desert battle royale

Cyrus (3D) · **N-gon x Survival** · Coming soon (empty scene)

| | |
|---|---|
| Name | The storms of the Dasht-e Kavir, Iran's great salt desert. |
| Mode | N-gon x Survival ([taxonomy](../../game-modes.md)), last one standing |
| Modifiers | ShrinkingZone (the storm), Permadeath |
| Camera | Third person |
| Players | 16 to 64 (in solos, duos or squads; bots fill) (+ spectators) |
| Physics | Rapier 3D |
| Screens | Local: split viewports (up to 2 x 2, as a squad). Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/ngon/sandstorm.scene.rs` |
| Build rung | R10 Scale and live ([ladder](../README.md#the-build-ladder)) |
| Twin | [Palang](../darius/ngon-survival-palang.md) (Darius (2D), same mode) |

## Pitch

A pure battle royale in third person. A flying carpet carries everyone over a desert of ruins, oases and towns; drop where you like, loot, and fight while the sandstorm closes in. Solos, duos or squads. The last side standing wins.

## Why it is this mode

Elimination is permanent for the match and the last side standing wins: N-gon x Survival (last one standing).

## References

- **Fortnite:** the drop from a flying vehicle, the closing storm circle in phases, loot, squads, healing and shield, third person. Its building is in (see the art rule).
- **Call of Duty: Warzone / Black Ops battle royale:** loadouts and weapon tiers, contracts, a second chance for downed players, squads.
- Its 2D twin is [Palang](../darius/ngon-survival-palang.md), after Super Animal Royale.

## The art rule

The game keeps the core features of the battle-royale genre (the drop, loot, the closing circle, building, vehicles, buy-back). Its art is adapted so these make sense in its world, and it does **not** respect history: items are fictional, and anachronisms are welcome. The balloon is the example: Warzone's buy-back station, redrawn as a wood-and-fabric airship.

## Rules (the `Sim`)

- **State:** the map (terrain, towns, cover), players (position, hp, shield, weapons, ammo, alive), loot, the storm (circle and phases), the carpet and the drop, who fell when.
- **The drop:** the flying carpet crosses the map; players leave it when they choose and glide.
- **Loot:** weapons in tiers with limited ammo, shields and healing, ammo, throwables.
- **The storm:** closes in phases and hurts more each time; players outside it lose health.
- **Building (Fortnite-style):** gather materials and place walls, ramps and floors in a moment. The art justifies it: stacked mud-brick, planks and stretched fabric panels.
- **Vehicles:** a few, as in the genre: a war chariot, a camel cart, a flying carpet. They are fictional; they need not be historical.
- **Buy-back (Warzone-style):** a **giant balloon made of wood and fabric** is anchored at a few places on the map. Spend darics at it to bring back a fallen teammate, who is lowered from it by rope.
- **Teams:** solos, duos, squads; a downed teammate can be revived on the spot or bought back at a balloon (limited).
- **A round ends when:** one side is left.
- **Result:** ranked by elimination order.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD + mouse, left button Fire, E Interact, R reload, Shift dash | Stick + look pad + the stack of three buttons |

## Screens

- **Local:** squad viewports, up to 2 x 2. **Online:** one camera per player. Spectators are delayed.

## Offline, online, spectate

- **Offline:** you plus **bots**.
- **Online:** a lobby that fills, up to 64.
- **Spectate:** follow anyone; the final two; replay.

## What it forces in the engine

- Large terrain and streaming
- Interest management (send each client only what is near)
- Many clients in one room, squads, revive
- Loot tables and zone phases shared with Palang

## Slices

1. Small map, 8 bots, storm
2. Loot, shields, the drop
3. Squads, revive and the buy-back balloon
4. Building and vehicles
5. Online 16, then 64

## Open questions

- **Decided:** building, vehicles and buy-back are all in, with art that justifies them (above).
- How many vehicles in the first release, and how are they driven with the pad?
- How many balloons on the map, and how many buy-backs per player per match?
