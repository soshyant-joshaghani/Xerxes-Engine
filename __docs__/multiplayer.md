# Multiplayer: a Rust port of Colyseus

Phase 3, track M (was Phase 6). The multiplayer server is a **Rust port of [Colyseus](https://github.com/colyseus/colyseus)**, the Node.js multiplayer framework. Colyseus is the main reference. [Golyseus](https://github.com/soshyant-joshaghani/Golyseus), a Go conversion of it made with AI assistance, is the proof that converting a real, working server to another language is practical; it is a second reference for how the conversion was cut, not the source of truth.

It lives in `backend/src/modules/multiplayer/` and is built to split into its own container ([backend.md](backend.md)). Nothing is built before its stage (see [`PROGRESS.md`](../__plans__/PROGRESS.md)).

## How we port

1. **Read the reference, then write idiomatic Rust.** Port behaviour and wire protocol, not Node's structure. Colyseus' `Room` class with lifecycle hooks becomes a Rust trait; its event emitters become async channels; its dynamic schema becomes derived types.
2. **Same concepts, same names** where Rust allows, so Colyseus documentation still helps users: Room, Client, `onCreate`/`onAuth`/`onJoin`/`onLeave`/`onDispose` (as `on_create`, `on_auth`, ...), `onMessage`, matchmaker (`create`, `join`, `joinOrCreate`, `filterBy`), presence, `maxClients`, lock/unlock, simulation interval, patch rate, reconnection.
3. **Wire compatibility is a goal to decide early.** If the Rust server speaks the Colyseus protocol (WebSocket framing, schema encoding), the existing Colyseus client SDKs work against it, and Xerxes games get a client for free. If it does not, Xerxes needs its own Bevy client. Decide this in the first stage after reading the protocol; the choice shapes every later stage.
4. **Port in vertical slices, each driven by a game that needs it** (bottom-up, like the engine). Each slice has tests, including conformance tests against recorded Colyseus traffic where wire compatibility is chosen.
5. **License and attribution.** Colyseus is open source; check its license (it is believed to be MIT) and keep the required notices in ported code and a `NOTICE` file. Do the same for Golyseus if anything is taken from it.

## Colyseus concepts and where they land

From the author's knowledge of Colyseus; verify each against the repository when its stage starts.

| Colyseus | Xerxes (Rust) | Stage |
|----------|---------------|-------|
| Transport (WebSocket) | `multiplayer/transport` (Axum WebSocket upgrade at `/multiplayer/ws`) | M0 |
| Room (`onCreate`, `onJoin`, `onLeave`, `onMessage`, `onDispose`) | `multiplayer/room`: a `Room` trait, one task per room | M0 |
| Client (session id, send, leave) | `multiplayer/client` | M0 |
| Matchmaker (`create`, `join`, `joinOrCreate`, `filterBy`, room listing) | `multiplayer/matchmaker` | M2 |
| Schema state + delta patches | `multiplayer/state`: derived state types, binary patches at a patch rate | M1 |
| Simulation interval, lock/unlock, `maxClients`, auth hook | in `room` | M2 |
| Reconnection (`allowReconnection`) | `multiplayer/room` | M4 |
| Presence (local and Redis) | `multiplayer/presence`, Redis in `core` | M4 |
| Driver (room cache), multi-process scaling | Redis-backed; `SERVICES=multiplayer` containers | M6 |
| Client SDKs | Wire-compatible with the JS client, or a Bevy client crate for games (decision in M0) | M0 |

## Stages: the M track (pulled forward, 2026-10-06)

Multiplayer no longer waits for a late phase. Darius must run **every game mode online, offline and spectated** ([darius.md](darius.md)), so the Colyseus concepts are built early, in a track (`M0` to `M6` in [PROGRESS.md](../__plans__/PROGRESS.md)) that runs beside the single-player stages and is pulled by the scenes that need it. Phases 6 to 8 of the old plan are these stages.

| Stage | Colyseus concepts | What it unlocks in darius |
|---|---|---|
| **M0** Read and decide; Room and Client over WebSocket | Transport, Room (`on_create`, `on_join`, `on_leave`, `on_message`, `on_dispose`), Client. Decide wire compatibility and the state encoding first. | Two clients in one room that can send each other messages. |
| **M1** Server-authoritative state sync | Schema state with delta patches at a patch rate, simulation interval; the room hosts `xerxes_sim` (the Bevy-free rules and physics crate). Clients send inputs and predict with the same crate. | `solo-*`, `duel-*` online: the server runs the match. |
| **M2** Matchmaker, room options, accounts | `create`, `join`, `joinOrCreate`, `filterBy`, `maxClients`, lock/unlock, auth hook (accounts from the API series) | Every grid cell online (Coop, N-gon need many clients); the hub's online portals. |
| **M3** Spectators and replays | Not a Colyseus built-in: a read-only join, a delay buffer, redaction of hidden information, and input/snapshot recording | Spectate every scene live; replay finished matches; hidden-role and fog scenes stay fair. |
| **M4** Reconnect, presence | `allowReconnection`, presence (local, then Redis); online Warp scopes ([warp.md](warp.md)) | Dropping and rejoining a match; party warps in the hub. |
| **M5** Ratings, leaderboards, tournaments, live sessions | Not Colyseus: Glicko-2 pairwise ratings, brackets as a wrapper over Duel rooms, a 24/7 free-for-all room | `tournament`, `live-session`. |
| **M6** Scale out | Redis presence and room cache (Colyseus' driver and presence), a multiplayer container per series (`SERVICES=multiplayer`) | Many rooms across processes. |

Rules for the whole track:

- A stage is `done` only when a darius scene uses it and its matrix tests pass (online and spectate included for the scenes it unlocks).
- The server stays Bevy-free. Anything it needs from the game (rules, physics) comes from `xerxes_sim`, which the game also uses offline.
- Each stage names the Colyseus source it ported and the license notice kept (see "How we port").
