# Darius and Cyrus: how every mini-game is built

The rules every mini-game of [Darius](darius.md) and [Cyrus](cyrus.md) follows. The design of each game (pitch, rules, controls, bots, spectating, slices) is its GDD in [gdd/](gdd/README.md). Read [game-modes.md](game-modes.md) for the taxonomy (Sides x Win condition) and [darius.md](darius.md#one-match-many-kinds-of-player) for participants (Local, Bot, Remote, Spectator).

Games are Bevy only: bevy_ui for menus and HUD, sprites (Darius) or meshes (Cyrus) for the world. Dioxus never appears in a game; it is the editor's UI. The small Dioxus games in `rust-template/rust-dioxus-learn/src/routes/games` (Guess, Rock Paper Scissors, Snake, Tic Tac Toe) are only *reference for how to keep rules apart from the screen*: each has a plain rules module (`new`, `play`/`step`, a status enum, a seeded start) and a thin page on top. That is exactly the `xerxes_sim` shape: the rules are a Bevy-free value that takes inputs and gives state, and the Bevy scene is its skin. We reuse the **ideas**, not the code: Snake's fixed tick and `step()` become the sim's fixed step (and Snake itself grows into Zahhak), Tic Tac Toe's `Result<Status, MoveError>` becomes a rejected input, `use_seeded` becomes the match seed.

## Every game, the same four rules

1. **Rules are a `Sim`**, in `xerxes_sim`, stepped at a fixed rate with a seed. Inputs in, state and events out. The scene never decides who won; the sim does.
2. **Offline = Local + Bots** on this device. A bot is a small function `(state, side) -> input` in the sim crate, so the same bot fills a room on the server. Every game ships a bot (an easy one is enough).
3. **Online = Remote** players in a room running the same `Sim` on the server; clients send inputs and predict.
4. **Spectate = read-only**, for *every* game, including solo ones. A solo run is spectatable live (a "watch" list of running rooms) and as a replay (recorded inputs + snapshots, run again through the `Sim`). That is what makes a solo game online: a solo room has one player and any number of spectators, and a finished run can be a ghost, a leaderboard entry or a replay link.

Every GDD says what a spectator sees when it differs from the player (hidden information) and what the camera options are.

## The games

The old working titles of this page were replaced on 2026-10-12 by named games with a GDD each (and more varied cameras: side views, cut-aways, UI-only games, not only top-down):

| Mode | Was | Now (Darius, 2D) | Cyrus (3D) |
|---|---|---|---|
| Solo x Score | Neon Dash | [Chapar](gdd/darius/solo-score-chapar.md) | [Rakhsh](gdd/cyrus/solo-score-rakhsh.md) |
| Solo x Objective | Crate Gauntlet | [Qanat](gdd/darius/solo-objective-qanat.md) | [Arash's Arrow](gdd/cyrus/solo-objective-arashs-arrow.md) |
| Solo x Survival | Swarm | [Ahriman's Horde](gdd/darius/solo-survival-ahrimans-horde.md) | [Atash](gdd/cyrus/solo-survival-atash.md) |
| Coop x Score | Juggle | [Tombak Duo](gdd/darius/coop-score-tombak-duo.md) | [Nowruz Feast](gdd/cyrus/coop-score-nowruz-feast.md) |
| Coop x Objective | Twin Plates | [Bound Bros](gdd/darius/coop-objective-bound-bros.md) | [Croesus' Gold](gdd/cyrus/coop-objective-croesus-gold.md) |
| Coop x Role | Defuse | [Behistun](gdd/darius/coop-role-behistun.md) | [Pasargadae Depths](gdd/cyrus/coop-role-pasargadae-depths.md) |
| Coop x Survival | Hold the Vault | [Naft](gdd/darius/coop-survival-naft.md) | [The Immortals](gdd/cyrus/coop-survival-the-immortals.md) |
| Duel x Score | Air Hockey | [Sofreh](gdd/darius/duel-score-sofreh.md) | [Farr](gdd/cyrus/duel-score-farr.md) |
| Duel x Objective | Tank Bases | [Siege of Sardis](gdd/darius/duel-objective-siege-of-sardis.md) | [Derafsh](gdd/cyrus/duel-objective-derafsh.md) |
| Duel x Role | Raid | [Karvan](gdd/darius/duel-role-karvan.md) | [Atashkadeh](gdd/cyrus/duel-role-atashkadeh.md) |
| Duel x Survival | Sumo | [Mobarezeh](gdd/darius/duel-survival-mobarezeh.md) | [Ramz](gdd/cyrus/duel-survival-ramz.md) |
| N-gon x Score | Kart Cup | [Zahhak](gdd/darius/ngon-score-zahhak.md) | [Kuchulu Cup](gdd/cyrus/ngon-score-kuchulu-cup.md) |
| N-gon x Objective | Paint Territory | [Kashi](gdd/darius/ngon-objective-kashi.md) | [Arzhang](gdd/cyrus/ngon-objective-arzhang.md) |
| N-gon x Role | Impostor | [The False Smerdis](gdd/darius/ngon-role-false-smerdis.md) | [Writing on the Wall](gdd/cyrus/ngon-role-writing-on-the-wall.md) |
| N-gon x Survival | Last Circle | [Palang](gdd/darius/ngon-survival-palang.md) | [Sandstorm](gdd/cyrus/ngon-survival-sandstorm.md) |
| Sandbox | Toy Box | [Jamshid's Workshop](gdd/darius/sandbox-jamshids-workshop.md) | [Pairidaeza](gdd/cyrus/sandbox-pairidaeza.md) |
| Progression | Loot Crawl | [Caravanserai](gdd/darius/progression-caravanserai.md) | [Haft Khan](gdd/cyrus/progression-haft-khan.md) |
| Narrative | Courtyard | [Naqqal's Teahouse](gdd/darius/narrative-naqqals-teahouse.md) | [The Dream of Astyages](gdd/cyrus/narrative-dream-of-astyages.md) |
| Composite | Town | [Persepolis](gdd/darius/composite-persepolis.md) | [Ecbatana](gdd/cyrus/composite-ecbatana.md) |
| Tournament | Sumo Bracket | [Zurkhaneh Cup](gdd/darius/tournament-zurkhaneh-cup.md) | [Farr League](gdd/cyrus/tournament-farr-league.md) |
| Live session | Colosseum | [Maydan](gdd/darius/live-maydan.md) | [Tirgan](gdd/cyrus/live-tirgan.md) |

## Build order

The order is the [build ladder](gdd/README.md#the-build-ladder): each rung adds one engine layer and is proven by the cheapest games that need it, Darius first, with two cheap Cyrus games early.
