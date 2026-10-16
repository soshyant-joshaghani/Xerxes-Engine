# Game design: Darius and Cyrus

Two games, one engine. **Darius** (2D) and **Cyrus** (3D) each cover the **whole** [game mode taxonomy](../game-modes.md): every cell of Sides x Win condition, plus the six kinds outside the grid. That is 21 games each, 42 in all. Each is its own small game built around the idea of its mode, not a reskin of another; the two games of one cell share only the mode.

This folder is the design of every one of them: one GDD per game, all in the same shape. The *why* of the project (prove the engine against every mode, offline, online and spectated) is in [darius.md](../darius.md) and [cyrus.md](../cyrus.md).

## Names

Xerxes, Darius and Cyrus are Achaemenid kings, so every game is named from Persian history and myth, with a plain English subtitle in the menu. Darius (2D) draws on the empire at its height (Persepolis, the Royal Road, the daric coin); Cyrus (3D) on the founding and the legends (Babylon, Ecbatana, the Shahnameh). Each GDD says what its name means.

## Design rules

**Art serves the genre, history does not.** A game keeps the core features of the genre it is built on (a battle royale has building, vehicles and a buy-back). The art is adapted so those features make sense in its world. Items are fictional; historical accuracy is not a goal.

**The cell test.** Ask what the number is and what ends the run, then read the taxonomy.

- **Score**: you push a number up (distance, combo, points). Nothing is trying to wipe you out on its own clock. Endurance is not the point; execution is.
- **Objective**: a defined goal, pass or fail, tiered instead of ranked.
- **Role**: different jobs, different information.
- **Survival**: the game pushes back and the number is how long you outlast it (depth, waves, last standing).

**The proven loop.** Every game starts from a loop already known to be fun and adds one twist that is the reason it exists here.

**One pad (Darius).** Every Darius game that needs more than movement uses the same layout: a stick on the left, a stack of three buttons on the right. The slots are fixed; each game only changes what they mean. "To act you must stand still" is Darius's rule when Fire commits (hold Fire and you stop; the stick becomes aim). Chapar, Qanat, Tombak Duo and Behistun need none of it and use the stick or taps.

| Slot | Role | In Ahriman's Horde |
|---|---|---|
| **Stick** (left) | Move. While Fire is held it is the aim instead. | same |
| **Fire** (top) | The game's main act, and **it commits**: hold it and you stop. | shoot |
| **Context** (middle) | Near an interactable it **interacts**; far from one it does the game's alternate. | interact / switch weapon |
| **Utility** (bottom) | Gated by readiness: ready, it is the movement tool; not ready, it restores readiness. | dash when loaded / reload when not |

Keyboard: WASD move, mouse aims (hold left button = Fire) or Space = Fire with arrows aiming, E = Context, Shift or R = Utility. Gamepad: left stick, right trigger = Fire, X = Context, B = Utility. The widget is built once in Ahriman's Horde and reused.

**Screens.** Every multiplayer game works two ways and states which in its GDD: local split screen (one viewport per player, or one shared camera that splits when they spread apart), and online one screen per player. A game whose cell depends on hidden information (Behistun, The False Smerdis) cannot use a literal split screen; it says what it does instead.

**The Gun is a spine.** Ahriman's Horde's weapon stays in that game until a second user needs it, then becomes an add-on (`engine/addons/gun`). Reused by Naft (Thrower and Igniter), Maydan (24/7 arena), Persepolis contracts, and Atash (the same weapon in 3D).

## Darius (2D)

| | Score | Objective | Role | Survival |
|---|---|---|---|---|
| **Solo** | [Chapar](darius/solo-score-chapar.md)<br>side-scroller | [Qanat](darius/solo-objective-qanat.md)<br>side cut-away | *structurally empty* | [Ahriman's Horde](darius/solo-survival-ahrimans-horde.md)<br>top-down shooter |
| **Coop** | [Tombak Duo](darius/coop-score-tombak-duo.md)<br>rhythm lanes | [Bound Bros](darius/coop-objective-bound-bros.md)<br>top-down, chained | [Behistun](darius/coop-role-behistun.md)<br>UI, split info | [Naft](darius/coop-survival-naft.md)<br>top-down, gel and fire |
| **Duel** | [Sofreh](darius/duel-score-sofreh.md)<br>top-down kitchen | [Siege of Sardis](darius/duel-objective-siege-of-sardis.md)<br>top-down lane MOBA | [Karvan](darius/duel-role-karvan.md)<br>top-down escort | [Mobarezeh](darius/duel-survival-mobarezeh.md)<br>side view |
| **N-gon** | [Zahhak](darius/ngon-score-zahhak.md)<br>top-down serpents | [Kashi](darius/ngon-objective-kashi.md)<br>top-down glaze | [The False Smerdis](darius/ngon-role-false-smerdis.md)<br>top-down, crowd | [Palang](darius/ngon-survival-palang.md)<br>top-down, battle royale |

| Sandbox | Progression | Narrative | Composite | Tournament | Live session |
|---|---|---|---|---|---|
| [Jamshid's Workshop](darius/sandbox-jamshids-workshop.md)<br>toys + gun range | [Caravanserai](darius/progression-caravanserai.md)<br>events, darics | [Naqqal's Teahouse](darius/narrative-naqqals-teahouse.md)<br>perform the tale | [Persepolis](darius/composite-persepolis.md)<br>contract gates | [Zurkhaneh Cup](darius/tournament-zurkhaneh-cup.md) | [Maydan](darius/live-maydan.md)<br>Gun arena |

## Cyrus (3D)

| | Score | Objective | Role | Survival |
|---|---|---|---|---|
| **Solo** | [Rakhsh](cyrus/solo-score-rakhsh.md)<br>lane run | [Arash's Arrow](cyrus/solo-objective-arashs-arrow.md)<br>timed speedrun | *structurally empty* | [Atash](cyrus/solo-survival-atash.md)<br>light is health |
| **Coop** | [Nowruz Feast](cyrus/coop-score-nowruz-feast.md)<br>kitchen + bonfire | [Croesus' Gold](cyrus/coop-objective-croesus-gold.md)<br>two-person litter | [Pasargadae Depths](cyrus/coop-role-pasargadae-depths.md)<br>first person + 2D map | [The Immortals](cyrus/coop-survival-the-immortals.md)<br>formation command |
| **Duel** | [Farr](cyrus/duel-score-farr.md)<br>third person | [Derafsh](cyrus/duel-objective-derafsh.md)<br>MOBA, lit push | [Atashkadeh](cyrus/duel-role-atashkadeh.md)<br>first person | [Ramz](cyrus/duel-survival-ramz.md)<br>first person, binoculars |
| **N-gon** | [Kuchulu Cup](cyrus/ngon-score-kuchulu-cup.md)<br>chase, tiny karts | [Arzhang](cyrus/ngon-objective-arzhang.md)<br>third person, ink | [Writing on the Wall](cyrus/ngon-role-writing-on-the-wall.md)<br>first person | [Sandstorm](cyrus/ngon-survival-sandstorm.md)<br>third person, battle royale |

| Sandbox | Progression | Narrative | Composite | Tournament | Live session |
|---|---|---|---|---|---|
| [Pairidaeza](cyrus/sandbox-pairidaeza.md) | [Haft Khan](cyrus/progression-haft-khan.md) | [The Dream of Astyages](cyrus/narrative-dream-of-astyages.md) | [Ecbatana](cyrus/composite-ecbatana.md) | [Farr League](cyrus/tournament-farr-league.md) | [Tirgan](cyrus/live-tirgan.md) |

## Cameras

Neither game is all one view. Darius: side-scrollers and side cut-aways (Chapar, Qanat, Mobarezeh, Jamshid's Workshop, the teahouse), top-down (Ahriman's Horde, Naft, Bound Bros, Sofreh, Karvan, Siege of Sardis, Zahhak, Kashi, The False Smerdis, Palang, Persepolis, Maydan) and UI-only (Tombak Duo, Behistun, Caravanserai). Cyrus: chase cameras (Rakhsh), third person (Arash's Arrow, Croesus' Gold, The Immortals, Farr, Derafsh, …), isometric (Atash), a fixed kitchen camera (Nowruz Feast), first person, and one game (Pasargadae Depths) where the two players see the same match as a 3D view and a 2D map.

## The build ladder

The order in which the games get built. Each rung adds **one engine layer** and is proven by the cheapest games that need it; the next game that needs the same thing is where it gets extracted (code used once stays in the game). Darius fills most of its grid first; one cheap Cyrus game, Rakhsh (R3), starts early to catch anything that only works in 2D while it is still cheap to fix. Arash's Arrow needs the 3D core (R8) and waits for it.

| Rung | Engine layer it adds | Darius | Cyrus | PROGRESS stages |
|---|---|---|---|---|
| R0 Catalogue | Menu grid, placeholder scenes, Ahriman's Horde becomes the wave shooter; engine gaps noted (levels by id, per-level logic) | [Ahriman's Horde](darius/solo-survival-ahrimans-horde.md) | - | today |
| R1 Rules without physics | A step (turn-based) world, undo, rejected inputs, tiered results (in the game first, `src/qanat/rules.rs`; into `xerxes_sim` when a second step game needs them); the physics lab | [Qanat](darius/solo-objective-qanat.md), [Jamshid's Workshop](darius/sandbox-jamshids-workshop.md) | - | 3.2 hardening |
| R2 Participants and bots | Participants (Local, Bot), hot-seat, N sides, best-of rounds, split-screen viewports; the first Cyrus scene comes at R3 (Rakhsh) | [Sofreh](darius/duel-score-sofreh.md), [Zahhak](darius/ngon-score-zahhak.md), [Bound Bros](darius/coop-objective-bound-bros.md) | - | 3.3, M0-M1 |
| R3 Side view and ghosts | 2D follow camera, platformer controller (extracted on its 2nd use), input recording for ghosts; 3D camera rig | [Chapar](darius/solo-score-chapar.md), [Mobarezeh](darius/duel-survival-mobarezeh.md) | [Rakhsh](cyrus/solo-score-rakhsh.md) | 3.4, M2 |
| R4 Hidden information | A private view per participant, phases, votes; spectator redaction comes for free | [Behistun](darius/coop-role-behistun.md), [The False Smerdis](darius/ngon-role-false-smerdis.md) | - | 3.4, M3 |
| R5 Crowds, paths and objectives | A*, waves, vehicles, territory, escort | [Naft](darius/coop-survival-naft.md), [Siege of Sardis](darius/duel-objective-siege-of-sardis.md), [Karvan](darius/duel-role-karvan.md), [Kashi](darius/ngon-objective-kashi.md) | - | 3.4, M2 |
| R6 Audio clock | An audio clock the rules read | [Tombak Duo](darius/coop-score-tombak-duo.md) | - | 3.4 |
| R7 Tools (Warp, save, quests, dialogue) | Variables, save, Warp, quests, dialogue, UI kit | [Caravanserai](darius/progression-caravanserai.md), [Naqqal's Teahouse](darius/narrative-naqqals-teahouse.md), [Persepolis](darius/composite-persepolis.md) | - | 3.5-3.11, M4 |
| R8 3D core | 3D physics, third-person controller, animation, navmesh | - | [Atash](cyrus/solo-survival-atash.md), [Arash's Arrow](cyrus/solo-objective-arashs-arrow.md), [Nowruz Feast](cyrus/coop-score-nowruz-feast.md), [Croesus' Gold](cyrus/coop-objective-croesus-gold.md), [The Immortals](cyrus/coop-survival-the-immortals.md), [Farr](cyrus/duel-score-farr.md), [Derafsh](cyrus/duel-objective-derafsh.md), [Kuchulu Cup](cyrus/ngon-score-kuchulu-cup.md), [Arzhang](cyrus/ngon-objective-arzhang.md), [Pairidaeza](cyrus/sandbox-pairidaeza.md), [Haft Khan](cyrus/progression-haft-khan.md), [The Dream of Astyages](cyrus/narrative-dream-of-astyages.md), [Ecbatana](cyrus/composite-ecbatana.md) | Phase 4 |
| R9 First person | First-person controller, pointer lock, lag compensation | - | [Pasargadae Depths](cyrus/coop-role-pasargadae-depths.md), [Atashkadeh](cyrus/duel-role-atashkadeh.md), [Ramz](cyrus/duel-survival-ramz.md), [Writing on the Wall](cyrus/ngon-role-writing-on-the-wall.md) | Phase 5 |
| R10 Scale and live | Ratings, brackets, 24/7 rooms, interest management | [Zurkhaneh Cup](darius/tournament-zurkhaneh-cup.md), [Maydan](darius/live-maydan.md), [Palang](darius/ngon-survival-palang.md) (pure online, after M1) | [Sandstorm](cyrus/ngon-survival-sandstorm.md), [Farr League](cyrus/tournament-farr-league.md), [Tirgan](cyrus/live-tirgan.md) | M5-M6 |

Why this order:

- **Cheapest proof first.** Bots start on Zahhak's open field and Sofreh's kitchen (small, no physics), not on a physics-heavy game.
- **Extract on the second use.** The platformer controller moves into an add-on when Mobarezeh (the next side view) needs what Chapar has; Tirgan keeps its own archery.
- **Hidden information before spectators.** Rung 4 builds a private view per participant offline (a pass-the-device screen); M3's spectator redaction then reuses it.
- **3D early, cheaply.** Rakhsh is the first Cyrus scene: lanes and a camera rig, no 3D physics, so the 3D pipeline is tested before a physics world exists. Arash's Arrow (a speedrun with a character controller) waits for the 3D core.

## Status today

Both games are scaffolded: a menu grid showing every mode, and a scene for every game. Four Darius games play: **Chapar** (slices 1 to 4), **Qanat** (its first room), **Ahriman's Horde** (courtyard slice still the old room-clear; waves are the next slice) and **Jamshid's Workshop** (slice 1: spawn/drag toys, gravity slider). Cyrus: **Pairidaeza** slice 0 (3D physics lab, `Physics3d` pulled forward) and **Arash's Arrow** (its first slice is a 3D diorama, to be rebuilt as a speedrun at R8). Renames are applied in the code (2026-10-14 and 2026-10-17). Every other game is an empty scene that shows its name, mode and camera, with a Back button; its menu tile says *Coming soon* and still opens that scene, so the wiring is real.

## A GDD's shape

Every GDD has the same sections, so a game can be built without guessing: the header table (name, mode, modifiers, camera, players, physics, screens for multiplayer games, scene, rung, twin), **Pitch**, **Why it is this mode**, **Rules** (state, inputs, when a round ends, the result, as the `xerxes_sim` sees them), **Controls** (keyboard and touch), **Offline, online, spectate**, **What it forces in the engine**, **Slices** (thin playable steps, first one offline) and **Open questions**. Answer the open questions in the GDD when they are decided.
