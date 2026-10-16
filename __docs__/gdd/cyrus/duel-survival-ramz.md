# Ramz: The number on your head

Cyrus (3D) · **Duel x Survival** · Coming soon (empty scene, still named `kamandar.scene.rs`; renamed in the batch fix)

| | |
|---|---|
| Name | Ramz is Persian for a code or a secret. |
| Mode | Duel x Survival ([taxonomy](../../game-modes.md)), round elimination |
| Modifiers | Permadeath (inside a round) |
| Camera | First person, with binoculars |
| Players | 2 teams, 2v2 up to 4v4 (+ spectators) |
| Physics | Rapier 3D (character mover, line of sight) |
| Screens | Local: split viewports (up to 2 x 2). Online: one camera per player ([Screens](#screens)) |
| Scene | `games/__templates__/cyrus/assets/scenes/games/duel/ramz.scene.rs` |
| Build rung | R9 First person ([ladder](../README.md#the-build-ladder)) |
| Twin | [Mobarezeh](../darius/duel-survival-mobarezeh.md) (Darius (2D), same mode) |

## Pitch

Everyone in the district wears a four-digit number on their head, and nobody can see their own. There are no guns: you have **binoculars** and a **device**. Read an enemy's number from a rooftop, key it in, and they die. Key in the wrong number three times and the device kills **you**. Key in a teammate's number and it costs you **two** wrong guesses.

## Why it is this mode

No respawn inside a round, and the team that is left standing wins: Duel x Survival (round elimination). The kill is information, not aim.

## Rules (the `Sim`)

- **State:** the district (rooftops, alleys, cover), each player (position, number, team, alive), each player's wrong-guess count, the round, the teams' round wins.
- **Numbers:** each player has a four-digit number visible on their head to **everyone but themselves**. You know your teammates' numbers from a roster; you do not know the enemy's until you read them.
- **Binoculars:** hold Fire to zoom (you stop, the Darius rule). The number is legible only at a distance and a clear angle through cover; movement and distance blur it.
- **The device:** Interact opens it; entering a number takes a couple of seconds, during which you are exposed.
  - A correct **enemy** number kills that player.
  - A wrong number is **one wrong guess**.
  - A **teammate's** number counts as **two** wrong guesses.
  - **Three** wrong guesses and the device kills you.
- **A round ends when:** one team has no player alive.
- **A match is:** best of 5.
- **Result:** ranked: the team that wins three rounds.
- **Assumed, to confirm:** the wrong-guess count resets each round; neutral NPCs also wear numbers (entering one is a wrong guess), so the district has a crowd to hide in.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD + mouse, hold left button binoculars, E device, Shift sprint | Stick + look pad + the stack of three buttons |

## Screens

- **Local:** split viewports; since every number is public to the other players, nothing secret is exposed by a shared screen.
- **Online:** one camera per player.
- **Spectators** are delayed.

## Offline, online, spectate

- **Offline:** you plus **bots** that scan, read numbers, avoid their teammates' numbers and hide.
- **Online:** a queue, 2v2 to 4v4.
- **Spectate:** delayed, with numbers; replay.

## What it forces in the engine

- First-person controller and a zoom camera
- A "visible to everyone but you" attribute and legibility by distance and line of sight
- Wrong-guess accounting and a lethal device
- Crowds of NPCs as decoys

## Slices

1. One district, two players, numbers, binoculars, the device and its three guesses
2. Rounds, teams, the teammate penalty
3. Bots; NPC crowd
4. Online; delayed spectating

## Open questions

- Is a kill announced to everyone, or only to the dead player's team?
- How long does entering a number take?
- Does the crowd wear numbers (assumed)?
