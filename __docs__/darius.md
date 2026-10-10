# Darius (and Cyrus later)

**Darius** (`games/__templates__/darius`) is the working game of Phase 3 in [PROGRESS.md](../__plans__/PROGRESS.md), and **Cyrus** (`cyrus`, Phase 4) is its 3D sibling.

Darius is small and 2D on purpose, and its job is to **prove the engine against the whole [game mode taxonomy](../engine/src/modules/flow/game-mode-taxonomy.html)**: a small mini-game for every mode, each playable **offline**, **online** and **spectated**. If a mode cannot be built on Xerxes, that is an engine gap, and we find it here, cheaply, instead of in a big game.

## The flow

```text
Splash  ->  Main menu (a scene)  ->  a mode scene  ->  results  ->  back to the menu
```

- **Splash**: the engine's flow (`Preload -> Splash`), as today.
- **Main menu is itself a scene**, not an engine-drawn list. It is the first place of the game's Warp map ([warp.md](warp.md)): pick a family (Solo, Coop, Duel, N-gon, Other), pick how to play (Offline, Online, Spectate), and pick the mini-game. For Online it shows the matchmaking: quick play (join or create), create a room and share its code, and the list of live rooms to spectate. The engine's flow therefore has to let a project name a scene as its main menu (stage 3.0).
- **Each mode is its own scene**, reached through a Warp link with the mode, the play kind and the room as params.
- **The results screen** and the way back are shared; everything between is the scene's own.

## Each scene is its own mini-game

The point of 21 scenes is not 21 reskins. Each scene is a **different small game, built around the idea of its mode**, so the mode's concept is easy to see and the code stays clean:

- A kart race and a battle royale and a bomb-defusal conversation share a match, a result and an HUD, and almost nothing else.
- **Do not copy a scene to make the next one.** If two scenes need the same thing, it is extracted once into the engine or the add-ons, because it proved shared. If they do not, they stay separate.
- **Physics only where the game has physics.** The kart race, sumo, air hockey and shooters use Rapier ([physics.md](physics.md)); the puzzle, the bomb manual, tower defense and the paint game do not, and are built on grid and turn logic. An engine that only works for physics games would fail this test.
- **Matchmaking only where the game needs it** (see the table): a solo run needs none, a duel needs a queue, a social game needs a lobby that fills.
- Every scene is a plugin: `scenes/<name>/` with its rules (against `xerxes_sim`), its Bevy skin, and its bots.

## The 21 mini-games

Names are working titles. "Online needs" is what the matchmaker and rooms must provide. "Exposes" is the engine capability the scene forces us to build, which is the reason it is in the list.

**Solo**

| Scene | Mode | The mini-game | Physics | Online needs | Exposes |
|---|---|---|---|---|---|
| `solo-score` | Solo x Score | **Neon Dash**: a side-view platformer time trial; ranked against your best and a ghost of it | Rapier (platformer) | none; leaderboard and ghost sharing | ghost replays, leaderboards |
| `solo-objective` | Solo x Objective | **Crate Gauntlet**: Sokoban-like rooms; bronze, silver, gold by moves | none (grid) | none; tier results | step-based (non-realtime) rules, undo |
| `solo-survival` | Solo x Survival | **Swarm**: a top-down horde survivor with auto-attacks; ranked by time survived | light (circle separation) | none | many entities, waves, upgrades |

**Coop** (one side, shared fate)

| Scene | Mode | The mini-game | Physics | Online needs | Exposes |
|---|---|---|---|---|---|
| `coop-score` | Coop x Score | **Juggle**: keep a ball in the air together; bonus only when both touch it in turn | Rapier (ball) | party code, 2 players | shared score, cooperation bonuses |
| `coop-objective` | Coop x Objective | **Twin Plates**: two characters, two pressure plates and a door; clear time | Rapier (sensors) | party code, 2 players | shared fate, triggers |
| `coop-role` | Coop x Role | **Defuse**: one player sees the bomb, the other has the manual; talk and cut the right wire | none (UI only) | party code, 2 players | per-player private views, FogOfWar-like information split |
| `coop-survival` | Coop x Survival | **Hold the Vault**: tower defense, the team places towers against waves | none (paths) | party code, 2 to 4 | pathfinding, build phase, shared resources |

**Duel** (two sides)

| Scene | Mode | The mini-game | Physics | Online needs | Exposes |
|---|---|---|---|---|---|
| `duel-score` | Duel x Score | **Air Hockey**: score from a contested puck | Rapier (puck) | 1v1 quick queue, rating later | contested shared object, authoritative physics |
| `duel-objective` | Duel x Objective | **Tank Bases**: top-down tanks; destroy the other base, escort a payload; you can lose every fight and win | Rapier (shells) | 1v1 or team queue | Respawn, Escort, MovingObjective |
| `duel-role` | Duel x Role | **Raid**: the attacker rushes, the defender sets traps in a prep phase; roles swap each round | light | 1v1 queue, role swap | asymmetric sides, phases, round swapping |
| `duel-survival` | Duel x Survival | **Sumo**: push the other out of the ring; best of N rounds, no respawn in a round | Rapier (impulses) | 1v1 queue | Permadeath within a round, round elimination |

**N-gon** (three or more sides)

| Scene | Mode | The mini-game | Physics | Online needs | Exposes |
|---|---|---|---|---|---|
| `ngon-score` | N-gon x Score | **Kart Cup**: a small top-down kart race; points by finishing rank over several laps | Rapier (vehicles) | lobby that fills to 3 or more, countdown | vehicle control, laps and checkpoints, rank points |
| `ngon-objective` | N-gon x Objective | **Paint Territory**: sides paint a grid; most area wins; alliances shift | none (grid) | lobby, 3 or more | shared contested state, area scoring |
| `ngon-role` | N-gon x Role | **Impostor**: tasks on a map, then a meeting and a vote; hidden roles | none (tile movement) | lobby, 4 or more (bots fill) | hidden information, voting and chat primitives, phases |
| `ngon-survival` | N-gon x Survival | **Last Circle**: a small battle royale where you only move and shoot, in a shrinking zone | Rapier (bullets) | lobby, up to 16 | ShrinkingZone, permanent elimination, many clients |

**Outside the grid**

| Scene | Kind | The mini-game | Physics | Online needs | Exposes |
|---|---|---|---|---|---|
| `sandbox` | Sandbox | **Toy Box**: drop shapes, change gravity, spawn things; no win condition | Rapier (heavy) | none | LowGravity, spawners, the editor-in-game feel |
| `progression` | Progression | **Loot Crawl**: a roguelite room crawler with levels, gear, an economy | light | none (cloud save) | items, inventory, economy, save |
| `narrative` | Narrative | **Courtyard**: a story scene with dialogue, quests and no failure | none | none | dialogue, quests, sequences, Warp revisits |
| `composite` | Composite | **Town**: the open hub. Solo missions, free roam, progression, and portals into online rooms | kinematic | party and room portals | Warp at full size, mixing modes |
| `tournament` | Tournament (wrapper) | **Sumo Bracket**: wraps `duel-survival` over elimination rounds | Rapier | bracket registration | the wrapper over any Duel or N-gon cell |
| `live-session` | Live session | **Colosseum**: a 24/7 free-for-all tank shooter; ranked by Glicko-2, not kills | Rapier | always-on room, join any time | persistent rooms, rating, an empty server that cannot be farmed |

Every modifier of the taxonomy is shown by at least one scene: Permadeath (`duel-survival`), Respawn (`duel-objective`, `coop-survival`), TimeLimit (`solo-score`, `duel-score`), FogOfWar (`coop-role`), LowGravity (`sandbox`, `ngon-score`), ShrinkingZone (`ngon-survival`), Escort and MovingObjective (`duel-objective`, `ngon-objective`), and the asymmetric flag (`duel-role`).

## One match, many kinds of player

A match is always the same thing: a set of **sides**, each with **participants**, running the mode's **rules**. What changes is who the participants are:

| Participant | Who it is | Where it runs |
|---|---|---|
| **Local** | A person on this device (keyboard, gamepad, touch; several on one device for hot-seat or couch play) | the client |
| **Bot** | A simple AI that stands in for a person | wherever the match runs |
| **Remote** | A person in another client of the same room | the server, via the client |
| **Spectator** | A read-only viewer: sees the match, takes no part | the client |

| Kind | Participants | Where the rules run |
|---|---|---|
| **Offline** | Local + Bot (bots fill every side a mode needs: a duel needs two, an N-gon three or more) | in the game, on the device |
| **Online** | Remote (+ Bot to fill, optionally) | on the server, in a room |
| **Spectate** | Spectators of a live room, or a **replay** of a finished match | read-only: a live feed, or the recorded inputs run again |

Every mode is online and offline by construction, because Solo, Coop, Duel and N-gon are only different numbers of sides; spectating is "join as read-only". **Solo games are spectatable too**: a solo run is a room with one player and any number of spectators, and a finished run is a replay, a ghost or a leaderboard entry. That is what makes a solo game online.

The design card of every scene (rules, controls, bots, what a spectator sees) is in [darius-games.md](darius-games.md).

## The simulation core

For this to hold, the rules cannot live in Bevy systems that only exist on a client. They live in a small Bevy-free crate, **`xerxes_sim`** (planned at `engine/sim/`, a stable path like `engine/build/`):

- the match state machine (lobby, countdown, playing, round end, results) and scoring
- the sides, roles, win conditions and modifiers from the taxonomy ([game-modes.md](game-modes.md))
- a **world for each scene's rules**: a Rapier world for the physics scenes, a grid or turn state for the others (the crate does not force physics on a puzzle)
- inputs in, state and events out, stepped at a fixed rate

Offline, the Bevy game owns a `Sim` and feeds it Local and Bot inputs. Online, the same crate runs inside a room of the Rust multiplayer server ([multiplayer.md](multiplayer.md)); clients send inputs, receive state patches and predict locally with it. For replays and spectators, inputs and periodic snapshots are recorded and run through the same crate, so a match reproduces exactly (hence deterministic).

## Spectating, and what a spectator may see

Spectators join read-only and follow a player, a side, or an overview camera. Some modes are about hidden information, and spectating must not break them:

- In `ngon-role`, `coop-role` and any FogOfWar scene, a live spectator sees the *public* view only, or a delayed one (a delay buffer configured per room), so a spectator cannot feed secrets to a player. After the match the full replay is open.
- Replays record inputs and snapshots, not a video, so any viewer can pick a camera.

## Done means a matrix

A scene is not done when it plays. It is done when a test runs it as **offline**, **online** and **spectated**, with the participant mix it supports, and checks the result against its mode's rules: 21 scenes x 3 kinds, generated from one table, run in `test games`. The engine is done with a mode when its scene passes. The order in which the scenes unlock is in [PROGRESS.md](../__plans__/PROGRESS.md).

## Names, and the 3D sibling

- `games/__templates__/darius` shows as **Darius**; `games/__templates__/cyrus` will be **Cyrus**. They are developed together with the engine, so they live in `games/__templates__/` **in the engine repository** (`games/` itself is gitignored for `game new` games), and are run with the `template` commands (`template dev darius`). Both graduate to `games/__templates__/` (read-only, copy to make your own) when finished ([games README](../games/__templates__/README.md)).
- Cyrus (Phase 4) reuses the same tools (rules, Warp, quests, dialogue, save, multiplayer) with 3D scenes, which is how we check that they are not 2D-only.
