# Darius: the mini-game ideas

One design card per scene of [darius.md](darius.md). The table there says *which* mode a scene proves; this page says *what the game is*, so a scene can be built without guessing. Read [game-modes.md](game-modes.md) for the taxonomy (Sides x Win condition) and [darius.md](darius.md#one-match-many-kinds-of-player) for participants (Local, Bot, Remote, Spectator).

Games are Bevy 2D only: bevy_ui for menus and HUD, sprites for the world. Dioxus never appears in a game; it is the editor's UI. The small Dioxus games in `rust-template/rust-dioxus-learn/src/routes/games` (Guess, Rock Paper Scissors, Snake, Tic Tac Toe) are only *reference for how to keep rules apart from the screen*: each has a plain rules module (`new`, `play`/`step`, a status enum, a seeded start) and a thin page on top. That is exactly the `xerxes_sim` shape: the rules are a Bevy-free value that takes inputs and gives state, and the Bevy scene is its skin. We reuse the **ideas**, not the code: Snake's fixed tick and `step()` become the sim's fixed step, Tic Tac Toe's `Result<Status, MoveError>` becomes a rejected input, `use_seeded` becomes the match seed.

## Every game, the same four rules

1. **Rules are a `Sim`**, in `xerxes_sim`, stepped at a fixed rate with a seed. Inputs in, state and events out. The scene never decides who won; the sim does.
2. **Offline = Local + Bots** on this device. A bot is a small function `(state, side) -> input` in the sim crate, so the same bot fills a room on the server. Every game ships a bot (an easy one is enough).
3. **Online = Remote** players in a room running the same `Sim` on the server; clients send inputs and predict.
4. **Spectate = read-only**, for *every* game, including solo ones. A solo run is spectatable live (a "watch" list of running rooms) and as a replay (recorded inputs + snapshots, run again through the `Sim`). That is what makes a solo game online: a solo room has one player and any number of spectators, and a finished run can be a ghost, a leaderboard entry or a replay link.

Each card says what a spectator sees when it differs from the player (hidden information) and what the camera options are.

Legend: **Sides** x **Win**; players = people at once; **Bots** = what offline needs.

---

## Solo

### 1. `solo-score` Neon Dash (Solo x Score)
- **Idea:** a short side-view platformer course; run, jump, collect shards, finish fast. Score = shards x time bonus. A 90-second `TimeLimit`.
- **Controls:** left/right, jump (touch: two thumbs).
- **Sim:** Rapier platformer (box colliders, one-way platforms, sensors for shards and the exit).
- **Offline:** your run against your **ghost** (best run, replayed from its inputs). **Bots:** none needed; the ghost is the opponent.
- **Online:** a leaderboard (best score per course), ghost download of the top run.
- **Spectate:** watch a live run (follow the runner), or a replay of any leaderboard run.
- **Exposes:** ghosts, leaderboards, replay of one input stream.

### 2. `solo-objective` Crate Gauntlet (Solo x Objective)
- **Idea:** Sokoban rooms. Push every crate on a target. Bronze/silver/gold by move count.
- **Controls:** four directions, undo, restart.
- **Sim:** pure grid, turn-based; no physics. An input is a move and can be **rejected** (blocked), like Tic Tac Toe's `MoveError`.
- **Offline:** a bot is a solver (BFS) used as a **hint** button. **Bots:** the solver.
- **Online:** none needed for play; results (moves per room) are stored.
- **Spectate:** watch someone solve live (a turn-based stream is cheap), or replay the solution.
- **Exposes:** non-realtime rules, undo, step-based replay.

### 3. `solo-survival` Swarm (Solo x Survival)
- **Idea:** top-down horde survival. Auto-attacks; pick one of three upgrades at each level-up. Ranked by time survived.
- **Controls:** move only.
- **Sim:** light circle separation (no Rapier), many entities, waves from the seed.
- **Offline:** a seeded daily run. **Bots:** none.
- **Online:** daily seed leaderboard.
- **Spectate:** live follow (large entity counts: the room sends compact deltas) or replay.
- **Exposes:** many entities, waves, upgrade choices, seeded determinism.

---

## Coop (one side, shared fate)

### 4. `coop-score` Juggle (Coop x Score)
- **Idea:** keep a ball in the air together. A hit by the *other* player than last time doubles the combo.
- **Controls:** move along the floor, hit.
- **Sim:** Rapier (ball, two paddles).
- **Offline:** you + a bot partner. **Bots:** a partner that tracks the ball and sometimes misses on purpose on easy.
- **Online:** party code, 2 players. **Spectate:** overview camera.
- **Exposes:** shared score, cooperation bonus.

### 5. `coop-objective` Twin Plates (Coop x Objective)
- **Idea:** two characters, two pressure plates, a door. Each plate holds only while someone stands on it; both must be held to open the door. Clear time.
- **Controls:** each player moves their own character (hot-seat: WASD and arrows).
- **Sim:** Rapier sensors, a trigger graph (plate -> door).
- **Offline:** you + bot, or two people on one device. **Bots:** one that stands on the plate it is given.
- **Online:** party code, 2. **Spectate:** overview.
- **Exposes:** triggers, shared fate, local multiplayer (several Locals).

### 6. `coop-role` Defuse (Coop x Role)
- **Idea:** one player (Defuser) sees a bomb with wires and symbols; the other (Expert) holds the manual that says which wire to cut. They must talk.
- **Controls:** pointer/tap on wires; the Expert scrolls the manual.
- **Sim:** UI-only, turn-less state; **per-player private views** (the bomb vs the manual).
- **Offline:** you + bot: the bot plays the *other role* (it reads the manual out as text lines you choose from). **Bots:** manual-reader and defuser.
- **Online:** party code, 2; voice or text chat.
- **Spectate:** a live spectator sees **neither secret** until the end, or sees the bomb only (configurable). Replay shows both.
- **Exposes:** private per-player state, information splits, spectator view rules.

### 7. `coop-survival` Hold the Vault (Coop x Survival)
- **Idea:** tower defense; a shared coin pool; each player places towers on a grid between waves. The vault falls when it takes N hits.
- **Controls:** pick a tower, tap a cell.
- **Sim:** grid + path (A*) + wave spawner; build phase, then wave phase.
- **Offline:** you + bots that place towers by a fixed plan. **Bots:** a builder.
- **Online:** party code, 2 to 4. **Spectate:** overview, build phase is public.
- **Exposes:** pathfinding, phases, shared resources, simultaneous placement conflicts (two players pick one cell).

---

## Duel (two sides)

### 8. `duel-score` Air Hockey (Duel x Score)
- **Idea:** first to 7 goals; the puck is contested.
- **Controls:** drag your mallet (touch) or move with the mouse.
- **Sim:** Rapier (puck, mallets, walls with goals). Authoritative on the server.
- **Offline:** vs bot (predict the puck line). **Online:** 1v1 queue. **Spectate:** side view, replay.
- **Exposes:** contested shared object, authoritative physics, input prediction.

### 9. `duel-objective` Tank Bases (Duel x Objective)
- **Idea:** top-down tanks, one base each. Destroy the enemy base or escort a payload to it. You can lose every fight and still win by the objective. Respawn after 3 seconds.
- **Controls:** move, aim, fire.
- **Sim:** Rapier (shells, tanks, walls), objective HP, respawn.
- **Offline:** vs bot (go to base, shoot what is in range). **Online:** 1v1 or 2v2 queue. **Spectate:** overview or follow a tank.
- **Exposes:** Respawn, Escort, MovingObjective.

### 10. `duel-role` Raid (Duel x Role)
- **Idea:** the Attacker rushes a vault; the Defender, in a prep phase, places three traps. Then roles swap. Fewest steps to the vault wins over two rounds.
- **Controls:** grid moves and trap placement.
- **Sim:** grid, light; phases `Prep` -> `Raid` -> `Swap`; the Defender's traps are **hidden** from the Attacker.
- **Offline:** vs bot (random-weighted traps; shortest-path attacker). **Online:** 1v1, role swap each round.
- **Spectate:** live spectators see traps **after** they trigger or after the round (a delay). Replay shows all.
- **Exposes:** asymmetric sides, phases, hidden information, round swapping.

### 11. `duel-survival` Sumo (Duel x Survival)
- **Idea:** two discs in a ring. Push the other out. Best of 3. No respawn within a round.
- **Controls:** move, dash (cooldown).
- **Sim:** Rapier impulses and ring bounds.
- **Offline:** vs bot (aims at you, avoids the edge). **Online:** 1v1 queue. **Spectate:** fixed camera. Also the base for the `tournament` wrapper.
- **Exposes:** Permadeath inside a round, round elimination.

---

## N-gon (three or more sides)

### 12. `ngon-score` Kart Cup (N-gon x Score)
- **Idea:** a small top-down kart race, three laps. Points by finishing rank across 3 tracks. Low gravity patches on one track.
- **Controls:** steer, accelerate, drift (touch: tilt or thumbs).
- **Sim:** Rapier vehicles, checkpoints, laps.
- **Offline:** you + 3 bots following a racing line. **Online:** lobby that fills to 3+, countdown. **Spectate:** follow a kart or leader.
- **Exposes:** vehicle control, laps/checkpoints, rank points, lobby fill.

### 13. `ngon-objective` Paint Territory (N-gon x Objective)
- **Idea:** a grid of tiles; walk to paint them your colour, painting over others. Most tiles when the timer ends... but the objective is a **capture goal**: hold 60% for 10 seconds. A MovingObjective (a golden tile that moves).
- **Controls:** four directions.
- **Sim:** grid, no physics.
- **Offline:** vs 2-3 bots (greedy nearest unowned tile). **Online:** lobby of 3 to 6. **Spectate:** overview, tile counts.
- **Exposes:** MovingObjective, territory state patches (many small deltas), bot fill.

### 14. `ngon-role` Impostor (N-gon x Role)
- **Idea:** a small ship; most are Crew doing tasks, one or two are Impostors who sabotage and eliminate. Meetings and votes.
- **Controls:** move, interact, vote.
- **Sim:** grid + rooms, tasks, meeting phase, votes; roles are **secret**.
- **Offline:** you + bots with roles (bots lie by a script). **Online:** lobby 5 to 8.
- **Spectate:** live spectators see **public** state only, with a delay (they must not feed secrets to a player). After the match, the full replay.
- **Exposes:** secret roles, voting, meeting phases, spectator-safe views.

### 15. `ngon-survival` Last Circle (N-gon x Survival)
- **Idea:** a shrinking arena; the last one standing wins. Pickups give a shove.
- **Controls:** move, shove.
- **Sim:** Rapier bodies, a shrinking zone (ShrinkingZone modifier), out-of-bounds = out.
- **Offline:** you + bots. **Online:** lobby 4 to 8. **Spectate:** overview or follow the last two.
- **Exposes:** ShrinkingZone, elimination order, placement score.

---

## Other cells

### 16. `sandbox` Toy Box (Sandbox)
- **Idea:** no goal; a free play field with low gravity and toys (balls, crates, springs) to try the physics and spawn tools. Where new engine features are tried first.
- **Offline:** you. **Online:** shared room, 1 to 4; **Spectate:** yes. **Exposes:** LowGravity, free spawning, no win condition.

### 17. `progression` Loot Crawl (Progression)
- **Idea:** a tiny dungeon; kill, loot, level up, and the character **persists** between runs (save). Stats, gear, unlocked rooms.
- **Offline:** you. **Online:** save in the backend per player. **Spectate:** watch a run live.
- **Exposes:** save/load, inventory, unlock state ([save](dialogue-quest.md)).

### 18. `narrative` Courtyard (Narrative)
- **Idea:** a courtyard with three people; talk (dialogue), take a quest, return with an item. Choices change a variable and the ending.
- **Offline:** you. **Online:** a shared courtyard where each player has their own quest state. **Spectate:** watch the chosen lines.
- **Exposes:** dialogue, quests, conditions and actions, per-player state in a shared space.

### 19. `composite` Town (Composite)
- **Idea:** a town map (a Warp world): doors lead to Courtyard, Toy Box, Loot Crawl; party-wide state follows. Proves Warp at full size and a scene mixing modes.
- **Offline / Online / Spectate:** all, per place. **Exposes:** Warp places, ports, links, party scope.

### 20. `tournament` Sumo Bracket (Tournament)
- **Idea:** a bracket of Sumo matches, 4 to 8 players, winners advance. Any Duel or N-gon cell can be wrapped the same way.
- **Offline:** you + bots in the bracket. **Online:** registration, a bracket room, match rooms. **Spectate:** the bracket page and each live match.
- **Exposes:** wrapper over any cell, bracket state, scheduled starts.

### 21. `live-session` Colosseum (Live session)
- **Idea:** a 24/7 free-for-all tank arena; anyone joins or leaves any time; ranked by Glicko-2 rating, not kills; an empty server cannot be farmed (no rating change below N players).
- **Offline:** practice vs bots. **Online:** the always-on room. **Spectate:** yes, the default way to see it.
- **Exposes:** persistent rooms, drop-in/drop-out, ratings.

---

## Build order

The order in which scenes become possible follows the engine, and is the one in [PROGRESS.md](../__plans__/PROGRESS.md) (stages 3.4 to 3.11): grid games first (`solo-objective`, `ngon-objective`, `duel-role`, `coop-role` need no physics), then the physics ones when Rapier (3.1) is in, then the room-needing ones when the multiplayer track (M0 to M6) has rooms, spectators and a matchmaker.
