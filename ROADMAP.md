# Roadmap

Xerxes is a fullstack Rust game engine: a Bevy engine with a Dioxus editor, plus the API and multiplayer server games need, grown together from small real games. Engines usually leave the server to the developer; Xerxes ships it. Phases are tracked in [`__plans__/PROGRESS.md`](__plans__/PROGRESS.md).

- **Now (Phase 3): one small 2D game, every game mode, online, offline and spectated.** `games/__templates__/darius` stays the working game and gets a scene for each mode of the [game mode taxonomy](__docs__/game-modes.md) (21 scenes), while the engine grows the tools every game needs and other engines leave out: match rules, **Warp** (a level manager with ports, locks, parameters and quest integration), save, quests, dialogue, text and a UI kit, and the multiplayer server. See [`__docs__/coverage.md`](__docs__/coverage.md) and [`__docs__/darius.md`](__docs__/darius.md)
- Then a third-person game (3D scenes) and an FPS on top of those tools, extracting only what repeats
- Grow the engine modules (`engine/src/modules/`) from those games; physics is Rapier ([`__docs__/physics.md`](__docs__/physics.md))
- Multiplayer in `backend/src/modules/multiplayer/`, a Rust port of Colyseus, the main reference (Golyseus, a Go conversion, proves such a port is practical; see [`__docs__/multiplayer.md`](__docs__/multiplayer.md)): rooms, clients, server-authoritative state sync, matchmaking, spectators and replays, reconnect, ratings, scale-out, built from the start of Phase 3 as the game modes need it (the M track), and split into its own container when load calls for it. The API (accounts, game data) grows beside it
- Decide the native host for a Dioxus HUD over Bevy once a game needs it
- Track Bevy and Dioxus releases deliberately: bump, run `test all`, play each game
- Keep the backend on the [wire contract](../../CONTRACT.md)
