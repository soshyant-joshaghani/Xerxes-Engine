# Kashi: Tile glaze

Darius (2D) · **N-gon x Objective** · Coming soon (empty scene)

| | |
|---|---|
| Name | Kashi is the glazed tilework of Persian mosques and palaces. |
| Mode | N-gon x Objective ([taxonomy](../../game-modes.md)), multi-side control |
| Modifiers | TimeLimit 3 min, MovingObjective (a golden medallion tile) |
| Camera | Top-down, one courtyard on one screen |
| Players | 3 or 4 teams of 1 or 2 (+ spectators) |
| Physics | none (a tile grid and casts) |
| Screens | Local: one shared screen for up to four players, nothing hidden. Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/darius/assets/scenes/games/ngon/kashi.scene.rs` |
| Build rung | R5 Crowds, paths and objectives ([ladder](../README.md#the-build-ladder)) |
| Twin | [Arzhang](../cyrus/ngon-objective-arzhang.md) (Cyrus (3D), same mode) |

## Pitch

A courtyard of bare tiles and three or four teams with a pot of glaze each. Splat tiles your colour, glaze over everyone else's, dive into your own glaze to move fast and refill. When the three minutes are up, the team that holds the most tiles wins. A golden medallion tile that wanders the courtyard counts triple, so whoever leads becomes the target.

## Why it is this mode

Several sides fight over one shared area, and the leader is ganged up on, so alliances shift: N-gon x Objective (multi-side control).

## References

- **Splatoon** (Splatoon 3's Tricolor Turf War has three teams): painting territory with ink, swimming in your own ink to refill and move.
- **Crash Bash's Pogo Pandemonium:** a grid of squares painted your colour on a small one-screen arena for up to four players. Its maps are the model for our arena layouts; we draw our own.
- **Paper.io:** the free-for-all feel.

## Rules (the `Sim`)

- **State:** the tile grid and each tile's owner, each fighter (position, planted, tank, dive, alive), the medallion's tile, the clock.
- **The pad:** the stick moves. **Fire held** plants you and **splats a 3 x 3 area** a few tiles ahead along the aim; it costs tank. **Utility** **dives** into your own glaze: faster, nearly unseen, and it **refills the tank**; you can neither splat nor be splatted on foot while diving, and you can only dive on your own colour.
- **Walking** glazes your own tile thinly (it holds, but a splat covers it).
- **Enemy glaze** slows you and cannot be dived in.
- **A direct splat hit** on a fighter sends them back to their base for a short delay (Respawn).
- **The medallion** tile wanders; the team holding it scores it **three times**.
- **A round ends when:** the clock reaches zero.
- **Result:** ranked by tiles held (equal counts share a rank).

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD, mouse aim, hold left button Fire, Shift dive | Stick + the stack of three buttons (Context is unused) |

## Screens

- **Local:** the courtyard fits one screen for up to four players. Nothing is hidden.
- **Online:** one camera per player.

## Offline, online, spectate

- **Offline:** you plus **bots**: they head for unglazed or enemy tiles, dive to refill, and chase the medallion.
- **Online:** a lobby of 3 or 4 teams.
- **Spectate:** the courtyard with tile counts; replay from the inputs.

## What it forces in the engine

- Many small state changes (tile deltas) in one deterministic world
- MovingObjective
- A dive state (visibility, speed, refill) and a tank resource
- Four arena layouts as data
- Greedy bots

## Slices

1. One courtyard, 3 local players or bots, walking glaze, splat, the clock
2. Tank, dive, enemy-glaze slowdown, splat hits
3. The medallion; three more layouts
4. Online lobby; spectate

## Open questions

- Is a splat hit lethal for a delay, or only a stun?
- How many tiles ahead does a splat land, and can its range be held longer?
