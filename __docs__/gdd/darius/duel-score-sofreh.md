# Sofreh: Kitchen cook-off

Darius (2D) · **Duel x Score** · Coming soon (empty scene, still named `pomegranate_volley.scene.rs`; renamed in the batch fix)

| | |
|---|---|
| Name | The sofreh is the cloth spread on the floor for a meal, the heart of a Persian feast. |
| Mode | Duel x Score ([taxonomy](../../game-modes.md)), head-to-head score |
| Modifiers | TimeLimit 4 min |
| Camera | Top-down, one shared kitchen on one screen |
| Players | 2 teams, 1v1 up to 3v3 (+ spectators) |
| Physics | none (kinematic characters, carried items) |
| Screens | Local: one shared screen, nothing hidden. Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/darius/assets/scenes/games/duel/sofreh.scene.rs` |
| Build rung | R2 Participants and bots ([ladder](../README.md#the-build-ladder)) |
| Twin | [Farr](../cyrus/duel-score-farr.md) (Cyrus (3D), same mode) |

## Pitch

Two teams cook in **one kitchen**. Each team has its own order rail and its own serving hatch; the stoves, the pot rack and the cutting boards are shared. Cook the dishes on your orders and deliver them before the other team does. Grab the free grill, steal a dish off a counter, knock the plate out of a rival's hands. The team that delivered more dishes when the time runs out wins.

## Why it is this mode

The number is dishes delivered, the two teams contest the same stations, and the lead at the timer decides: Duel x Score (head-to-head score from a contested shared mechanic, not from hurting anyone).

## Rules (the `Sim`)

- **State:** the kitchen (stations, counters, items on them), each cook (position, held item, stun), each team's order rail, delivered dishes, the clock.
- **Inputs (each player):** stick; **Interact** (pick up, put down, chop, load a pot, deliver; hold to chop); **Fire held** (plant and throw the held item along the aim: a hit stuns a rival for 1.5 s and knocks their item loose); **Utility** (dash, short cooldown).
- **Dishes (data):** kebab (skewer, grill, flatbread, plate), ash (chop herbs, pot, bowl), tahdig (rice pot, crust, plate). Each is a graph of stations and times.
- **Orders:** every team has its own queue of tickets with a patience timer; a late dish scores less.
- **Contest:** one grill bank and one pot rack are shared, so stations are won by being there. A finished dish left on a counter can be picked up by anyone and delivered at their own hatch. An unattended grill burns the dish.
- **A round ends when:** the clock reaches zero. The higher number of dishes (with tips as the tiebreak) wins; a tie is played out to the first dish.
- **Result:** ranked: the winning team, with both scores.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD, E Interact, hold left button throws (aim with the mouse), Shift dash; further local players on arrows + numpad or gamepads | Stick + the stack of three buttons |

## Screens

- **Local:** the kitchen fits one screen, so it is one shared screen for up to six cooks. Nothing is hidden.
- **Online:** one camera per player, on that player.

## Offline, online, spectate

- **Offline:** you plus **bot cooks** that take a role (chopper, runner, deliverer) and fill the other team too.
- **Online:** a quick queue, 1v1 to 3v3.
- **Spectate:** the whole kitchen with both rails shown; replay from the orders' seed and the inputs.

## What it forces in the engine

- Carryable items and stations as data; a recipe graph; an order queue
- Many Local players on one device, in teams
- A contested shared object under server authority
- A bot that plays a role by a script

## Slices

1. One kitchen, one dish, two local cooks, delivery and the clock
2. Three dishes, order rails, burning, throw and stun
3. Bot cooks; a second kitchen layout
4. Online 1v1 to 3v3; spectate

## Open questions

- Is the grill shared (as written) or does each team own one and only the pot rack is shared?
- Can a cook be slowed by a heavy item (a full pot)?
