# Naft: Gel and fire

Darius (2D) · **Coop x Survival** · Coming soon (empty scene)

| | |
|---|---|
| Name | Naft is Persian for naphtha, the oil that seeps from the ground and burns; the fields around Baku fed the fire temples. |
| Mode | Coop x Survival ([taxonomy](../../game-modes.md)), team endurance |
| Modifiers | Revive (a downed player can be raised) |
| Camera | Top-down; one yard, about one and a half screens wide |
| Players | 2 to 4 (+ spectators) |
| Physics | none from Rapier: character mover and casts; gel and fire are a tile layer the rules own |
| Screens | Shared (follows the group), local split screen (up to 2 x 2), or one screen per player online ([Screens](#screens)) |
| Scene | `games/__templates__/darius/assets/scenes/games/coop/naft.scene.rs` |
| Build rung | R5 Crowds, paths and objectives; the **second use of the Gun**, so it is extracted into an add-on here ([ladder](../README.md#the-build-ladder)) |
| Twin | [The Immortals](../cyrus/coop-survival-the-immortals.md) (Cyrus (3D), same mode) |

## Pitch

A naphtha well in a walled yard, and raiders who want it. The team has two weapons that are useless alone. The **Thrower** lobs sticky gel that pools on the ground and coats whoever walks in it. The **Igniter** fires a spark that does almost nothing by itself but lights every gel it touches. One player lays the trap, another lights it, and the whole puddle burns. The well is the team's life and its ammunition; the team is ranked by the wave it held.

## Why it is this mode

The whole side survives or falls together and is ranked by how long it holds: Coop x Survival (team endurance). The weapons are the cooperation: one player cannot kill a wave alone.

## Rules (the `Sim`)

- **State:** The well (HP), players (position, aim, planted, dash, hp, down), their weapons (magazine and total), raiders (kind, status), the gel and fire tile layer, wave number, a shared purse of darics.
- **Inputs (each player):** stick, Fire, Interact / Switch, Reload / Dash: the same pad as [Ahriman's Horde](solo-survival-ahrimans-horde.md).
- **Thrower:** hold Fire to plant and lob gel along the aim; each glob makes a puddle (about 2 tiles, lasts about 10 s). A puddle **slows** whoever stands in it and **coats** them (Gelled, about 6 s).
- **Igniter:** a fast spark in a line. Small direct damage. **It ignites any gel it touches.** Fire spreads from puddle to adjacent puddle, one cell per 0.1 s, and burns each cell for a moment (heavy damage per tick to everyone in it, **including players**). A gelled creature that is ignited keeps burning for a few seconds and lights the gel it runs through.
- **Supplies are places.** The Thrower's gel is refilled by **Interact at the well**; the Igniter's sparks by **Interact at the brazier**. Ammunition therefore costs movement, and the well is both the thing you defend and the thing you drink from.
- **Friendly fire is only fire.** A teammate standing in or coated by gel is visibly marked; lighting them burns them. Nothing else hurts the team.
- **Everyone keeps the unlimited pistol** for raiders that gel and fire cannot handle.
- **Raiders:** rushers (swarm, the teaching enemy), shield-bearers (immune from the front; burn them through the gel under them), washers (they douse gel and fire, so they are priority targets), a ram brute (needs two coats), fireproof magi (pistol only).
- **Between waves (about 20 s):** the team spends the shared purse: repair the well, raise a barricade that funnels raiders into a gel lane, refill. Interact at the site.
- **A round ends when:** the well falls, or all players are down at once. A downed player is revived by a teammate's Interact.
- **Result:** Finished, ranked by wave reached.

More players means bigger waves, not new tools: extra players carry a second Thrower or Igniter, and everyone can carry both through Switch (each has its own magazine). The intended play is a split.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD, mouse aim, hold left button = Fire, E Interact / Switch, Shift or R Reload / Dash; a second local player on the arrows + numpad | Stick + the stack of three buttons |

## Screens

- **Online:** one screen per player, a camera on that player. The yard is only a little wider than one screen, so each sees nearly all of it.
- **Local:** the camera follows the group on one screen while everyone is within a screen of each other, and **splits into viewports** (two side by side, up to 2 x 2) when players spread apart, joining again when they close up.
- Nothing in the rules is hidden, so every view shows the whole truth.

## Offline, online, spectate

- **Offline:** you and a **bot partner** that takes the other weapon: a Thrower bot lays gel in front of the rush; an Igniter bot waits for gel under raiders and for teammates to clear it. Or two to four on one device.
- **Online:** party code, 2 to 4.
- **Spectate:** the whole yard; the build phase is public.

## What it forces in the engine

- **Status effects as data** (Gelled, Burning, Wet) on creatures and on a tile layer, with spread rules, in `xerxes_sim`
- Friendly hazards (damage from a teammate's effect)
- **Several cameras and viewports in one scene**, dynamic split, a HUD per viewport
- A* paths that change when barricades are placed
- Wave director, build phase, shared purse, placement conflicts (two players, one cell)
- Revive
- The Gun as an add-on (second use: weapons as data, inventory, the pad)

## Slices

1. One Thrower, one Igniter, two players on one screen (or one plus a bot), rushers only: lay gel, ignite, watch the chain
2. The well, refills at the well and brazier, waves, shield-bearers and washers, revive
3. The build phase and the shared purse; bot partner in either role; split screen
4. Online, 2 to 4 players; spectate

## Open questions

- How long does gel last (about 10 s), and does a washer remove it faster?
- Does fire block the raiders' path (and so also the team's)?
- Can a player coated in gel dash out of it (the dash has i-frames)?
