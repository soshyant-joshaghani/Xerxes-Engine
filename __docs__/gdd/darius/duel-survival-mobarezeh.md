# Mobarezeh: One-hit duel

Darius (2D) · **Duel x Survival** · Coming soon (empty scene)

| | |
|---|---|
| Name | Mobarezeh is Persian for a duel, a single combat. |
| Mode | Duel x Survival ([taxonomy](../../game-modes.md)), round elimination |
| Modifiers | Permadeath (inside a round), ShrinkingZone (late in a round) |
| Camera | Side view, one small arena on one screen |
| Players | 2 teams, 1v1 up to 3v3 (+ spectators) |
| Physics | Rapier (platformer, one-hit bullets as casts) |
| Screens | Local: one shared screen, nothing hidden. Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/darius/assets/scenes/games/duel/mobarezeh.scene.rs` |
| Build rung | R3 Side view and ghosts ([ladder](../README.md#the-build-ladder)) |
| Twin | [Ramz](../cyrus/duel-survival-ramz.md) (Cyrus (3D), same mode) |

## Pitch

A tiny arena, two teams, one hit kills. Everyone starts with a pistol and a few shots; better weapons, with limited ammo, drop on the floor on a timer. The round is over in seconds. The whole game is one question: **who shoots first loses their dash.**

## Why it is this mode

No respawn inside a round and the last side standing takes it: Duel x Survival (round elimination). A fatal hit makes every round a tiny, tense survival.

## Rules (the `Sim`)

- **State:** the arena (platforms, walls, spawn points), each fighter (position, velocity, planted, weapons, dash state, alive), weapon pickups and their timers, the round number and each team's round wins.
- **The pad (as Ahriman's Horde):** Fire held = **plant and shoot** along the stick; Interact / Switch = pick up or swap a weapon; Reload / Dash = **dash when the weapon is fully loaded, reload otherwise**. The stick flicked up **jumps** (a side view needs it).
- **One hit kills.** A dash has i-frames for its active frames only. There are no hearts and no revives.
- **Every shot spends the dash.** A fired weapon is not full, so the next press reloads instead of dashing. The mind game: bait a shot out of the other fighter, then close in on their empty weapon. Heavier weapons reload slowly and so dash rarely.
- **Weapons:** the pistol (a few shots, a short reserve), then found weapons: bow, javelin, spread thrower, naphtha pot; all with limited ammo.
- **A round ends when:** one team has no fighter alive. If the round runs long, **the arena closes in** from the sides so nobody can hide.
- **A match is:** best of 7. Rounds are 15 to 30 seconds.
- **Result:** ranked: the team that wins four rounds.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| A / D move, W jump, mouse aim, hold left button Fire, E pick up / swap, Shift or R Reload / Dash; P2 arrows | Stick (flick up to jump) + the stack of three buttons |

## Screens

- **Local:** the arena is one screen, so one shared view for up to six fighters. Nothing is hidden.
- **Online:** one camera per player.

## Offline, online, spectate

- **Offline:** you plus **bots**: they aim with a reaction delay, bait and dash, and fill the other team.
- **Online:** a quick queue, 1v1 up to 3v3; the first online duel (a small state).
- **Spectate:** the arena from the side; replay from the inputs.

## What it forces in the engine

- Participants and bots over one input interface, best-of rounds
- **The platformer controller's second user (after Chapar): extract it into an add-on**
- A deterministic one-hit sim with cheap projectiles
- The weapon data and inventory of the Gun add-on, in a duel
- The base of the tournament wrapper

## Slices

1. One arena, two local fighters, the pistol, the plant, the dash and the reload rule
2. Floor weapons, rounds and best of 7, the closing arena
3. Bots; a headless bot-vs-bot test to a result
4. Online 1v1 to 3v3; the Zurkhaneh Cup bracket

## Open questions

- Is a flick up to jump readable on a phone, or is a fourth button needed?
- How many shots does the starting pistol have so the dash still matters?
- Do teammates get to share weapons by dropping them?
