# Darius (and Cyrus)

**Darius** (`games/__templates__/darius`) is the working game of Phase 3 in [PROGRESS.md](../__plans__/PROGRESS.md), and **Cyrus** (`games/__templates__/cyrus`, [game-cyrus.md](game-cyrus.md)) is its 3D sibling. Both cover the whole taxonomy, each with its own 21 games; the design of all 42 is in [gdd/](gdd/README.md).

Darius is small and 2D on purpose, and its job is to **prove the engine against the whole [game mode taxonomy](../engine/src/modules/flow/game-mode-taxonomy.html)**: a small mini-game for every mode, each playable **offline**, **online** and **spectated**. If a mode cannot be built on Xerxes, that is an engine gap, and we find it here, cheaply, instead of in a big game.

## It must look right in the editor too

Darius grows with the engine, so it is checked on both sides: the running game and the editor's Scene view. Every 2D element is an object in a scene file (sprites, bodies, the player, the coins, the HUD canvas, the menu), written as plain data ([ui.md](ui.md#authoring-rules-so-the-editor-can-show-it)); game code only drives them through marker components. A project cloned from Darius then shows the whole level in the editor. The `ui_scene` test keeps it so, and a gap the editor has (something the game uses and the Scene view cannot show) is fixed in the engine.

## Mobile

Darius is the proving ground for the engine's touch controls ([ui.md](ui.md#mobile-inputs-virtual-joystick-and-touch-pad)). On a phone a thumb dropped in the lower-left of the screen starts a floating analog stick (`VirtualJoystick` in the HUD canvas); the keyboard still works beside it, and Pause / Restart are the HUD's buttons. A mode that needs aiming or looking uses a `TouchPad` on the other half of the screen.

## The flow

```text
Splash  ->  Main menu (a scene)  ->  a mode scene  ->  results  ->  back to the menu
```

- **Splash**: the engine's flow (`Preload -> Splash`), as today.
- **Main menu is itself a scene**, not an engine-drawn list. It is the first place of the game's Warp map ([warp.md](warp.md)): pick a family (Solo, Coop, Duel, N-gon, Other), pick how to play (Offline, Online, Spectate), and pick the mini-game. For Online it shows the matchmaking: quick play (join or create), create a room and share its code, and the list of live rooms to spectate. A project already names its menu scene (stage 3.0, done); the online parts of the menu (room list, matchmaking) wait for M0 to M2.
- **Each mode is its own scene**, reached through a Warp link with the mode, the play kind and the room as params.
- **The results screen** and the way back are shared; everything between is the scene's own.

## Each scene is its own mini-game

The point of 21 scenes is not 21 reskins. Each scene is a **different small game, built around the idea of its mode**, so the mode's concept is easy to see and the code stays clean:

- A chariot race, a serpent battle royale and a decipher-together conversation share a match, a result and an HUD, and almost nothing else.
- **Do not copy a scene to make the next one.** If two scenes need the same thing, it is extracted once into the engine or the add-ons, because it proved shared. If they do not, they stay separate.
- **Physics only where the game has physics.** Mobarezeh and the platformers use Rapier ([physics.md](physics.md)); Qanat, Behistun, Kashi and Zahhak do not, and are built on grid and turn logic. An engine that only works for physics games would fail this test.
- **Matchmaking only where the game needs it** (see the table): a solo run needs none, a duel needs a queue, a social game needs a lobby that fills.
- Every game is a scene (`assets/scenes/games/<family>/<game>.scene.rs`) plus, once it plays, its code in `src/<game>/` (rules against `xerxes_sim`, its Bevy skin, its bots) plugged in by `src/logic.rs` (listed in the project file's `LOGIC`).

## The 21 mini-games

Every cell of the taxonomy has its own game, each with a Persian name and its own GDD in [gdd/darius](gdd/README.md#darius-2d) (pitch, rules, controls, bots, offline/online/spectate, what it forces in the engine, slices). The build order is the [build ladder](gdd/README.md#the-build-ladder).

| Mode | Game | Camera | Exposes |
|---|---|---|---|
| Solo x Score | **Chapar** (slices 1 to 4 playable) | side-scrolling runner | platformer, seeded road, ghosts, daily board |
| Solo x Objective | **Qanat** (first room playable) | side cut-away, grid | step rules, undo, tiers, a timer and a board per room |
| Solo x Survival | **Ahriman's Horde** (first slice playable as the old courtyard) | top-down shooter | weapons, infinite waves, barricades, the three-button pad, leaderboards |
| Coop x Score | **Tombak Duo** | rhythm lanes | audio clock, timing windows |
| Coop x Objective | **Bound Bros** | top-down | a deterministic chain, linked fate, split screen |
| Coop x Role | **Behistun** | UI only | private views per player |
| Coop x Survival | **Naft** | top-down | status effects (gel, fire), waves, friendly fire, viewports |
| Duel x Score | **Sofreh** | top-down kitchen | carryables, recipes, teams on one device |
| Duel x Objective | **Siege of Sardis** | top-down lane MOBA | wave director, towers, items, Respawn |
| Duel x Role | **Karvan** | top-down escort | Escort, role kits, role swap, asymmetric |
| Duel x Survival | **Mobarezeh** | side view | bots, rounds, Permadeath, the platformer controller, first online duel |
| N-gon x Score | **Zahhak** | top-down serpents | N sides on one field, bots, many small entities |
| N-gon x Objective | **Kashi** | top-down glaze | territory deltas, MovingObjective, dive |
| N-gon x Role | **The False Smerdis** | top-down, fog | secret roles, votes, FogOfWar |
| N-gon x Survival | **Palang** | top-down battle royale | interest management, zone phases, loot, squads |
| Sandbox | **Jamshid's Workshop** | side view | the physics lab, LowGravity |
| Progression | **Caravanserai** | map + trading | save, inventory, economy |
| Narrative | **Naqqal's Teahouse** | side view | dialogue, quests |
| Composite | **Persepolis** | top-down hub | Warp at full size |
| Tournament | **Zurkhaneh Cup** | bracket over Mobarezeh | the wrapper |
| Live session | **Maydan** | top-down arena | 24/7 room, ratings |

Every modifier of the taxonomy is shown by at least one game: Permadeath (Mobarezeh), Respawn (Siege of Sardis, Karvan, Maydan), TimeLimit (Sofreh, Kashi), FogOfWar (The False Smerdis, Behistun), LowGravity (Jamshid's Workshop), ShrinkingZone (Mobarezeh, Palang), Escort (Karvan), MovingObjective (Kashi) and the asymmetric flag (Karvan's uneven teams).

### Where the scaffold is

The menu (`assets/scenes/menu.scene.rs`) is the taxonomy grid: four rows (Solo, Coop, Duel, N-gon) by four win conditions, Solo x Role shown as structurally empty, and a row for the six kinds outside the grid. Each tile shows the game's name, its mode and a button. Every game is a level in `assets/darius.project.rs` with its `GameMode`, and its scene is `assets/scenes/games/<family>/<game>.scene.rs` (`solo`, `coop`, `duel`, `ngon`, `outside`). Games not built yet are empty scenes (title, mode, camera, *Coming soon*, a Back button); their menu tile is greyed and says *Coming soon*, and still opens the empty scene. A game's code lives in `src/<game>/`, plugged in by a function in `src/logic.rs` and listed in the project's `LOGIC` (today `ahrimans_horde`, `chapar`, `jamshids_workshop` and `qanat`).

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

The design card of every scene (rules, controls, bots, what a spectator sees) is in [game-darius.md](game-darius.md).

## The simulation core

For this to hold, the rules cannot live only in Bevy systems that exist on a client. Split them like this:

| Layer | Where | What |
|-------|--------|------|
| **This game's rules** | `games/__templates__/darius/rules/` (crate `darius_rules`), Bevy-free. Chapar's rules move there from `src/chapar/rules.rs` at step 5, when its daily board runs in a room. Qanat's rules are already in `xerxes_sim::qanat` (shared tools), so Qanat needs no game rules crate | Darics, ghosts, tiles, bots — **offline and online** |
| **Shared tools** | `xerxes_sim` (`engine/sim/`) | Match phases, taxonomy scoring sides, Rapier physics — every game needs these |
| **Multiplayer host** | `backend/.../multiplayer/` (the room library) and a per-game room-host binary that `game publish` builds | Room runtime; **links** the game's rules crate, does not contain them ([backend-multiplayer.md](backend-multiplayer.md)) |

- inputs in, state and events out, stepped at a fixed rate
- Offline: the Bevy game owns the sim and feeds Local / Bot inputs
- Online: the **same game rules** run inside a room ([backend-multiplayer.md](backend-multiplayer.md)); clients send inputs, get patches, predict locally
- Replays and spectators record inputs and snapshots through that same code (hence deterministic)

Never put Chapar/Qanat/… logic under `backend/src/modules/api/editor` (that folder only scaffolds projects for the web editor).

## Spectating, and what a spectator may see

Spectators join read-only and follow a player, a side, or an overview camera. Some modes are about hidden information, and spectating must not break them:

- In `ngon-role`, `coop-role` and any FogOfWar scene, a live spectator sees the *public* view only, or a delayed one (a delay buffer configured per room), so a spectator cannot feed secrets to a player. After the match the full replay is open.
- Replays record inputs and snapshots, not a video, so any viewer can pick a camera.

## Done means a matrix

A scene is not done when it plays. It is done when a test runs it as **offline**, **online** and **spectated**, with the participant mix it supports, and checks the result against its mode's rules: 21 scenes x 3 kinds. This matrix is the planned definition of done; it is not yet generated or run by `test games`. The engine is done with a mode when its scene passes. The order in which the scenes unlock is in [PROGRESS.md](../__plans__/PROGRESS.md).

## Names, and the 3D sibling

- `games/__templates__/darius` shows as **Darius**; `games/__templates__/cyrus` as **Cyrus**. They are developed together with the engine, so they live in `games/__templates__/` **in the engine repository** (`games/` itself is gitignored for `game new` games), and are run with the `template` commands (`template dev darius`). Both graduate to `games/__templates__/` (read-only, copy to make your own) when finished ([games README](../games/__templates__/README.md)).
- Cyrus reuses the same tools (rules, Warp, quests, dialogue, save, multiplayer) with 3D scenes, which is how we check that they are not 2D-only. One cheap Cyrus game (Siavash) is built early, beside Darius, and the rest in Phase 4 ([game-cyrus.md](game-cyrus.md)).
