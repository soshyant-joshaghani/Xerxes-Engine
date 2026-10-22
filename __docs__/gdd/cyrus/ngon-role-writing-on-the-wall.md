# Writing on the Wall: Banquet deduction

Cyrus (3D) · **N-gon x Role** · Coming soon (empty scene)

| | |
|---|---|
| Name | At Belshazzar's feast a hand wrote on the wall; that night Babylon fell to Cyrus. |
| Mode | N-gon x Role ([taxonomy](../../game-modes.md)) |
| Modifiers | FogOfWar (private clues) |
| Camera | First person in a banquet hall |
| Players | 5 to 10 (+ spectators) |
| Physics | none (character movement) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/ngon/writing_on_the_wall.scene.rs` |
| Build rung | R9 First person ([ladder](../README.md#the-build-ladder)) |
| Twin | [The False Smerdis](../darius/ngon-role-false-smerdis.md) (Darius (2D), same mode) |

## Pitch

The last feast in Babylon. Among the guests is a traitor who poisons cups. Clues write themselves on the wall between courses; each guest also holds a private clue. Talk, trade clues, accuse; the hall votes.

## Why it is this mode

Secret roles with different wins and different information: N-gon x Role (social deduction).

## Rules (the `Sim`)

- **State:** Roles, private clues, cups, the wall's clues, phase (Feast, Accuse, Vote).
- **Inputs:** Walk, look, talk (proximity), trade a clue, poison (traitor), vote.
- **A round ends when:** The traitor is voted out, or enough guests fall.
- **Result:** The winning role.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD + mouse, E interact | Stick + look pad |

## Offline, online, spectate

- **Offline:** You + bots with scripted lies.
- **Online:** A lobby of 5 to 10.
- **Spectate:** Public view with a delay; full replay after.

## What it forces in the engine

- A first-person social space
- Proximity chat
- Per-player secrets in 3D

## Slices

1. Hall, guests, one traitor, vote
2. Clues on the wall
3. Online lobby

## Open questions

- Proximity voice, or text bubbles?
