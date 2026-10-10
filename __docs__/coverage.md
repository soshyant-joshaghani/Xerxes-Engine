# What every game needs, and where Xerxes covers it

Xerxes answers what other engines leave to the developer. A renderer, physics and an editor are table stakes; the hard part of finishing a game is everything around them, which each team rebuilds. This is the checklist we plan against. A row is only `done` when a scene of darius ([darius.md](darius.md)) uses it.

Legend: stage numbers are in [PROGRESS.md](../__plans__/PROGRESS.md) (`3.x` single-player and tools, `M#` multiplayer track, `P#` other phases). "Today" is what exists in the repo now.

## Making the game

| Need | Usual engine answer | Xerxes | Stage |
|---|---|---|---|
| Rendering, audio, input, ECS | built in | Bevy (the runtime) | today |
| Editor, scenes, prefabs | built in (some) | Dioxus editor, typed `.rs` assets | today |
| Physics, 2D and 3D | built in or a plugin | Rapier ([physics.md](physics.md)) | 3.1 |
| Game modes and match flow (lobby, rounds, scoring, win conditions, roles, modifiers) | nothing | `xerxes_sim` rules from the taxonomy ([game-modes.md](game-modes.md)) | 3.2 |
| AI stand-ins (bots) so every mode plays alone | nothing | participants + bots | 3.3 |
| **Level management** (entries, exits, locks, revisits, parameters) | an array of scenes | **Warp** ([warp.md](warp.md)) | 3.7 |
| Variables, conditions, events as data | scripts or nothing | shared `Condition` / `Action` / event bus | 3.5 |
| Save and load, cloud save | a plugin | `modules/save` with per-platform storage | 3.6 |
| Quests | a plugin | `modules/quest` ([dialogue-quest.md](dialogue-quest.md)) | 3.8 |
| Dialogue, barks, cutscene sequencing | a plugin | `modules/dialogue`, `modules/sequence` | 3.9 |
| Localization | a plugin | text tables, `Text` fields | 3.10 |
| UI kit (panels, menus, prompts that follow the input device, settings screen) | partly | bevy_ui kit | 3.10 |
| Input remapping, per-device prompts, pointer lock on desktop, web and phones | partly | named actions (today) + prompts; pointer lock designed in [input.md](input.md), not scheduled | today, 3.10 |
| Game settings (audio, graphics profile, language, controls) | by hand | project + user settings, with the rendering profiles | P2e, 3.10 |
| Character and camera controllers | samples | add-ons (`engine/addons`), Tnua option | today, 3.1 |
| Inventory, items, economy | a plugin | after the first game that needs it (the `progression` scene) | 3.11 |
| Achievements | a platform SDK | after accounts exist | M2, later |

## Running the game online

| Need | Usual engine answer | Xerxes | Stage |
|---|---|---|---|
| Accounts, sessions, profiles | build a backend | the API series (`backend/src/modules/api`) | today |
| Rooms, clients, messages | a third-party service | Rust port of Colyseus ([multiplayer.md](multiplayer.md)) | M0 |
| Server-authoritative state sync, prediction | a netcode plugin | `xerxes_sim` in rooms, delta patches | M1 |
| Matchmaking | a service | matchmaker (create, join, joinOrCreate, filters) | M2 |
| Spectating and replays | rarely | read-only clients, input recordings | M3 |
| Reconnection, presence | by hand | reconnect tokens, presence | M4 |
| Ratings, leaderboards, tournaments, live sessions | a service | Glicko-2, leaderboards, brackets | M5 |
| Scale out | ops work | Redis presence and room cache, containers per series | M6 |
| Server hosting, deploys | ops work | Docker for every build, `backend run prod` | today |
| Cloud saves, player data | a service | API + save storage | 3.6 + M2 |
| Anti-cheat basics | a service | server authority is the first line; input validation in rooms | M1 |
| Admin and live-ops tools (ban, message, inspect a room) | build a dashboard | an admin area of the API, later | after M2 |
| Telemetry and crash reports | a service | later, once there are players | later |

## Shipping

| Need | Usual engine answer | Xerxes | Stage |
|---|---|---|---|
| Builds for web, desktop, phone | per-platform work | one pipeline for web, windows, android | today |
| Web build that runs on another server | by hand | a Dockerfile with every web build | today |
| Phone and low-end profiles | by hand | rendering profiles (Compat/Full, Low/Medium/High) | P2e |
| Store and installer packaging | per store | after a game needs it | later |

## How to use this list

When a stage is planned, find its rows here and ask whether the thing is something **every game of that kind** will need (then it is engine) or a choice (then it is an add-on or the game). When someone says "the engine should also have X", the answer is a new row, with a stage, or a reason it is not a Xerxes job.
