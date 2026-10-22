# The False Smerdis: Crowd assassin

Darius (2D) · **N-gon x Role** · Coming soon (empty scene)

| | |
|---|---|
| Name | In 522 BC Darius and six nobles unmasked Gaumata, a Magus ruling as the dead prince Smerdis. |
| Mode | N-gon x Role ([taxonomy](../../game-modes.md)) |
| Modifiers | FogOfWar (vision cones) |
| Camera | Top-down palace courtyard with fog |
| Players | 6 to 8 (+ spectators); ~30 NPC courtiers fill the crowd |
| Physics | none (tile / soft collision movement) |
| Screens | No literal split screen: a second device is not required, but fog and roles hide what others see ([Screens](#screens)) |
| Scene | `games/__templates__/darius/assets/scenes/games/ngon/false_smerdis.scene.rs` |
| Build rung | R4 Hidden information ([ladder](../README.md#the-build-ladder)) |
| Twin | [Writing on the Wall](../cyrus/ngon-role-writing-on-the-wall.md) (Cyrus (3D), same mode) |

## Pitch

A palace festival: thirty NPC courtiers wander, and six to eight players look identical to them. One player is the Magus; the rest are the Seven. The Magus wins by silencing enough nobles **without being seen** (vision cones, fog). The nobles win by accusing the Magus, but each gets **one** accusation and a wrong one eliminates the accuser. Behaviour that does not match the crowd is the tell. No map, no tasks: cheaper to build and better to spectate. References: Hitman crowds, Spy Party, Assassin's Creed Wanted.

## Why it is this mode

Each role has its own win condition and its own information: N-gon x Role (hidden role). The crowd is the information: matching it hides you; breaking it exposes you.

## Rules (the `Sim`)

- **State:** Crowd NPCs (wander paths, idle cycles), players (role secret, position, facing, vision cone, alive), who saw what (fog), accusation tokens remaining, silence count.
- **Inputs:** Move, silence (Magus, near a noble), accuse (one allowed per noble), run (a tell).
- **The pad:** Fire = silence (near a noble; Magus only, or no-op for others), Context = accuse (the one allowed), Utility = run (faster, but a tell the crowd never does).
- **Silence:** the Magus must be in a noble's blind spot (outside every living player's vision cone, including spectators' delayed view). A seen silence fails and marks the Magus for a short window.
- **Accuse:** one shot per noble. Wrong → the accuser is out. Right → Magus out, nobles win.
- **A round ends when:** Magus silenced enough nobles (Magus wins), Magus is correctly accused (nobles win), or too few nobles remain to accuse.
- **Result:** The winning role's living players.

## Controls

| Keyboard / mouse | Touch |
|---|---|
| WASD move, left button / Space silence, E accuse, Shift run | Stick + the stack of three (silence / accuse / run) |

## Screens

- **Local:** one shared camera with fog per player is impossible on one screen; local play passes the device, or uses a private second device for the Magus's identity only. Prefer online or pass-and-play.
- **Online:** one screen per player; fog and roles stay private.
- Hidden information is the whole mode: no literal split screen that would show the Magus.

## Offline, online, spectate

- **Offline:** You + bots with roles (bots match the crowd, or break it on purpose when Magus).
- **Online:** A lobby of 6 to 8.
- **Spectate:** Live: public state only, with a delay (no Magus identity); after the match the full replay.

## What it forces in the engine

- Crowd simulation (many cheap wanderers that look like players)
- Secret roles
- Vision cones (FogOfWar)
- Accuse / eliminate without a full meeting UI
- Spectator-safe views with a delay

## Slices

1. Courtyard, crowd NPCs, one Magus, silence and vision, offline with bots
2. Accuse (one shot) and win conditions
3. Online lobby
4. Delayed spectating

## Open questions

- How many silences to win (fixed count, or half the nobles)?
- Do NPC courtiers ever look at players, or only wander?
- Quick-accuse phrases for phones, or a tap on a player only?
