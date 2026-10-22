# Kuchulu Cup: Tiny racers, giant world

Cyrus (3D) · **N-gon x Score** · Coming soon (empty scene)

| | |
|---|---|
| Name | *Kuchulu* is Persian for tiny, the affectionate "little one". (Name provisional.) |
| Mode | N-gon x Score ([taxonomy](../../game-modes.md)), multi-way race |
| Modifiers | none |
| Camera | Third-person chase, low and close (the world is huge) |
| Players | 3 to 8 (bots fill the grid) (+ spectators) |
| Physics | Rapier 3D (an arcade vehicle: a raycast body, sensors, drifting) |
| Screens | Local: split viewports (2 or up to 2 x 2). Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/ngon/kuchulu_cup.scene.rs` |
| Build rung | R8 3D core ([ladder](../README.md#the-build-ladder)) |
| Twin | [Zahhak](../darius/ngon-score-zahhak.md) (Darius (2D), same mode) |

## Pitch

A kids' kart race in a Persian house the size of a city. You drive a chapar courier's cart or a war chariot, a few centimetres long, over a giant sofreh, along the rim of a samovar, through the fringe of a carpet and across a table of tea glasses. Drift-hop for a boost, grab crates, throw camel spit at the leader. Points by finishing place over a cup of four tracks.

## Why it is this mode

Points scale by finishing rank across several races, so second place still matters: N-gon x Score (multi-way race, the kart-racing cell).

## References

- **Crash Team Racing Nitro-Fueled:** the drift-hop turbo, item crates, fruit that raises top speed, Grand Prix cups, cartoon slapstick.
- **RC Cars:** the scale joke, tiny cars racing through a house and a garden that were built for people.
- **Theme:** Iranian, with the chapar (royal courier) and the war chariot as the two vehicle families.

## Rules (the `Sim`)

- **State:** racers (position, velocity, lap, checkpoint, item, boost, fruit count), items on the track, the clock, each cup's points.
- **Handling:** an arcade vehicle with a **hop-and-drift**: hop into a turn, hold the drift, and release for a boost whose strength depends on how long you held it.
- **Vehicles:** the chapar cart (light, quick to turn) and the war chariot (heavy, fast on straights); each with a few characters.
- **Pickups:** *pomegranates* on the track (up to ten) raise top speed; crates give an item.
- **Items:** a pomegranate bomb, a camel-spit puddle (slows), a flying slipper (homing), a stinky kebab on a skewer (drop it behind you), an evil eye (shield). The jokes are crude and silly, never cruel.
- **A race ends when:** every racer has finished, or a cut-off passes after the winner.
- **Points:** by finishing place (first the most, down to last).
- **A cup is:** four tracks; **the result is the total points**, ranked.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD / arrows steer and accelerate, Shift hop and drift, Space use item, C horn | Tilt or a stick to steer, buttons: item, hop / drift (accelerate is automatic) |

On the pad: the stick steers, **Fire uses the item**, **Utility is the hop and drift**, Context is the horn (a camel's groan).

## Screens

- **Local:** classic split screen (2 or up to 2 x 2). Nothing is hidden.
- **Online:** one camera per player.

## Offline, online, spectate

- **Offline:** you plus **bots** that follow a racing line, use items and take drifts.
- **Online:** a lobby that fills with bots (3 to 8), then a countdown.
- **Spectate:** follow any racer or the leader; replay from the inputs.

## What it forces in the engine

- Arcade vehicle handling and a drift-boost model in 3D
- Checkpoints, laps, a spline racing line for bots
- Items as data; a lobby that fills with bots
- Several viewports in a 3D scene

## Slices

1. One track, one cart, laps and checkpoints
2. Drift-hop and boost; pomegranates
3. Items and bots; three more tracks and the cup
4. Online lobby; spectate

## Open questions

- Is the camera locked behind the vehicle, or free so the scale of the world reads?
- Does a Battle mode (CTR's arena) belong in this game later?
