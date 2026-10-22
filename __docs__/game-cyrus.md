# Cyrus

**Cyrus** (`games/__templates__/cyrus`) is the 3D sibling of [Darius](darius.md). Like Darius it covers the **whole** [game mode taxonomy](game-modes.md) with its own small game per mode, 21 in all, so every cell is proven twice: once in 2D, once in 3D. The two games of one cell share only the mode, never the code of a scene; what they really share is extracted into the engine (rules, match runtime, Warp, save, quests, dialogue, multiplayer), which is exactly what Cyrus checks: that those tools are not 2D-only.

Its job beyond the taxonomy is the 3D toolkit every 3D game needs: 3D physics in `xerxes_sim`, camera rigs (chase, orbit, isometric, fixed, first person), a third-person and a first-person controller, animation, navmesh, terrain, and larger rooms online. Phase 5 (the FPS) is not a separate game: it lands in Cyrus's first-person games (Atashkadeh first).

## When it is built

Cyrus does not wait for Darius to finish. One cheap Cyrus game is built early, beside Darius, to catch anything that only works in 2D while it is still cheap to fix:

- **Siavash** (Solo x Score, rung 3): an endless run and gun in the style of Blades of Brim, a knight with a gun against demons, with a 3D chase camera and ghosts reused from Chapar. Its rules need no 3D physics, so it tests the scene pipeline, the editor's 3D view and the camera rig in 3D.

**Arash's Arrow** (Solo x Objective) is a Neon White speedrun: demons to clear before the end point opens, cards that are shot or burned for abilities, and a clock that ranks the run. It is first person, so it waits for the first-person controller (rung 9). Its scene today is a 3D clone of Qanat's first room, which is the wrong idea for this game: the rework replaces it with the first course of the speedrun.

The rest follows Phase 4 (3D core: physics, third person, animation, navmesh) and Phase 5 (first person). The order is the [build ladder](gdd/README.md#the-build-ladder).

## The 21 games

The GDD of each is in [gdd/cyrus](gdd/README.md#cyrus-3d).

| Mode | Game | Camera | Exposes |
|---|---|---|---|
| Solo x Score | **Siavash** | third-person chase, three lanes | 3D camera rig, lanes, the gun's magazine and combo, ghosts in 3D |
| Solo x Objective | **Arash's Arrow** | first person, bow in view | first-person controller, arrows and card shots as casts, cards, level clocks and medals |
| Solo x Survival | **Atash** | isometric, night | instanced crowds, lighting as gameplay |
| Coop x Score | **Nowruz Feast** | fixed kitchen camera | carry and interact, couch coop in 3D |
| Coop x Objective | **Croesus' Gold** | third person | third-person controller, split-screen |
| Coop x Role | **Pasargadae Depths** | first person + a 2D map | a 3D and a 2D view of one match |
| Coop x Survival | **The Immortals** | third person | navmesh, wave director, revive |
| Duel x Score | **Farr** | third person | carrier state, pickups, viewports in 3D |
| Duel x Objective | **Derafsh** | third-person MOBA | navmesh minions, towers, skill shots |
| Duel x Role | **Atashkadeh** | first person | FPS, round economy, plant and defuse |
| Duel x Survival | **Ramz** | first person, binoculars | first person, line of sight, hidden-from-self numbers |
| N-gon x Score | **Kuchulu Cup** | chase, tiny karts | arcade vehicle, drift boost, items, bots on a racing line |
| N-gon x Objective | **Arzhang** | third person | paintable 3D surfaces, swim and wall climb, 4 local viewports |
| N-gon x Role | **Writing on the Wall** | first person | social space, proximity chat |
| N-gon x Survival | **Sandstorm** | third person | terrain streaming, interest management |
| Sandbox | **Pairidaeza** | free builder camera | in-game 3D placement, the 3D lab |
| Progression | **Haft Khan** | third person | animation, stats and gear |
| Narrative | **The Dream of Astyages** | cinematic | timeline, 3D dialogue staging |
| Composite | **Ecbatana** | third person hub | Warp in 3D |
| Tournament | **Farr League** | bracket over Farr | the wrapper with teams |
| Live session | **Tirgan** | first person | persistent 3D rooms, shared archery add-on |

## Where the scaffold is

The same shape as Darius: `assets/cyrus.project.rs` lists the 21 games as levels with their `GameMode`; the menu (`assets/scenes/menu.scene.rs`, a 2D UI scene) is the taxonomy grid; every game's scene is `assets/scenes/games/<family>/<game>.scene.rs`, a 3D scene (`Dimension::D3`: camera, sun, ground) with a HUD canvas showing its name, mode, camera and a Back button. Two are playable: Arash's Arrow (slice 1, a 3D diorama of Qanat's first room) and Pairidaeza (slice 0, a 3D physics lab). The other 19 are *Coming soon*.

## The same rules as Darius

Everything in [darius.md](darius.md) about participants (Local, Bot, Remote, Spectator), offline / online / spectate, the matrix test, plain-data scenes the editor can read, and [the four rules of every mini-game](game-darius.md#every-game-the-same-four-rules) holds for Cyrus unchanged.
