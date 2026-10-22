# Naqqal's Teahouse: Branching tales

Darius (2D) · **Narrative** · Coming soon (empty scene)

| | |
|---|---|
| Name | A naqqal is the storyteller who performs the Shahnameh in Persian teahouses. |
| Mode | Narrative ([taxonomy](../../game-modes.md)) |
| Modifiers | none |
| Camera | Side view, a walkable teahouse |
| Players | 1 (+ spectators) |
| Physics | none |
| Scene | `games/__templates__/darius/assets/scenes/games/outside/naqqals_teahouse.scene.rs` |
| Build rung | R7 Tools (Warp, save, quests, dialogue) ([ladder](../README.md#the-build-ladder)) |
| Twin | [The Dream of Astyages](../cyrus/narrative-dream-of-astyages.md) (Cyrus (3D), same mode) |

## Pitch

Evening in a teahouse. You are the storyteller's apprentice and **you perform the tale**: each beat of Rostam's story you pick what the puppet does, and the guests react and remember. Talk, fetch props, choose a line. No failure, only the story; the night ends one of several ways.

## Why it is this mode

Authored forward progress without a failure state: Narrative.

## Rules (the `Sim`)

- **State:** Story variables, quest states, the current beat of the tale, guest memory, the puppet's last act.
- **Inputs:** Walk, talk, choose a line, give an item, pick the puppet's next act.
- **A round ends when:** The story reaches an ending.
- **Result:** An ending (a branch, not a rank).

## Controls

| Keyboard / mouse | Touch |
|---|---|
| A/D walk, E talk, 1-3 choose | Tap to walk and talk |

## Offline, online, spectate

- **Offline:** You.
- **Online:** A shared teahouse where each player has their own story state.
- **Spectate:** Watch the chosen lines.

## What it forces in the engine

- Dialogue graphs (stage 3.9)
- Quests (3.8)
- Conditions and actions as data (3.5)
- Sequences
- Guest memory that reacts to the puppet's acts

## Slices

1. Three guests, one quest, puppet acts on one beat, one ending
2. Choices and three endings; guests remember
3. Shared teahouse online

## Open questions

- Voice-over or text only?
