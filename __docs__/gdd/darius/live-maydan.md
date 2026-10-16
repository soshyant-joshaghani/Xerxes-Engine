# Maydan: 24/7 Gun arena

Darius (2D) · **Live session** · Coming soon (empty scene)

| | |
|---|---|
| Name | A maydan is a great Persian square, like Isfahan's Naqsh-e Jahan, once a polo field. |
| Mode | Live session ([taxonomy](../../game-modes.md)) |
| Modifiers | Respawn |
| Camera | Top-down open square |
| Players | drop-in, up to 16 (+ spectators) |
| Physics | Rapier (character mover); bullets are cheap kinematic casts |
| Scene | `games/__templates__/darius/assets/scenes/games/outside/maydan.scene.rs` |
| Build rung | R10 Scale and live ([ladder](../README.md#the-build-ladder)) |
| Twin | [Tirgan](../cyrus/live-tirgan.md) (Cyrus (3D), same mode) |

## Pitch

An always-open arena that uses the **Gun** from [Ahriman's Horde](solo-survival-ahrimans-horde.md): you spawn with the pistol, weapons drop, anyone can join or leave. The Fire rule (hold to plant and shoot) keeps fights readable on a phone. You are ranked by a Glicko-2 pairwise rating, not by kills, so an empty server cannot be farmed.

## Why it is this mode

N-gon Score running as a 24/7 room instead of matches. Live session.

## Rules (the `Sim`)

- **State:** Fighters (position, aim, planted, inventory, health), ground guns, ratings.
- **Inputs:** Stick; Fire held; Interact / Switch; Reload / Dash — the same pad as Ahriman's Horde.
- **Spawn:** pistol only. Ground guns and Assassin Mode crates drop on a timer and on kills.
- **A round ends when:** Never (drop-in / drop-out).
- **Result:** Rating changes per kill (a mini 1v1 pairwise).

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD, mouse aim, left button Fire, E Context, Shift / R Utility | Stick + the stack of three (same as Ahriman's Horde) |

## Offline, online, spectate

- **Offline:** Practice vs bots with the Gun.
- **Online:** The always-on room.
- **Spectate:** Yes, the default way to see it.

## What it forces in the engine

- Persistent rooms, drop-in / drop-out
- The Gun add-on reused outside Ahriman's Horde (second user after Naft, or third if Atash lands first)
- Glicko-2 ratings (M5)

## Slices

1. Offline practice with bots and the pad
2. Always-on room, weapon drops
3. Ratings

## Open questions

- Rating change below N players: zero, or scaled?
- Soft aim assist on touch, or none (same question as Ahriman's Horde)?
