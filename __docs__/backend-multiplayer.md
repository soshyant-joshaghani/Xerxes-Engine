# Backend: multiplayer

The multiplayer series runs **online matches**: a room holds one match, the server keeps its authoritative state, and clients send inputs and receive state changes. It holds **no game logic**: each game's rules come from that game's `rules/` crate.

For accounts, boards and other services see [backend-api.md](backend-api.md). Stages are in [PROGRESS.md](../__plans__/PROGRESS.md) (track M0 to M6).

## What online games need

- **Rooms.** A match is a room. Clients join, leave, and rejoin after a drop.
- **Authoritative state.** The server runs the rules. Clients send inputs; the server sends state changes as patches at a fixed rate.
- **Matchmaking.** Create a room, join one by id, join any open room or create one, filter by room options, set a maximum number of clients, lock a room.
- **A fixed simulation.** The room steps the rules at a fixed tick, the same step the offline game uses, so one run gives one result everywhere.
- **Spectating and replays.** Read-only join, a short delay, hidden information kept hidden, and recording of inputs for replay.
- **Presence.** Who is in which room, for friends lists and the hub.
- **Changing rooms.** A player can leave one room and join another, as a warp does ([warp.md](warp.md)).

## Principles

1. **The rules are not here.** A game's rules live in its `rules/` crate: Bevy-free, depending on `xerxes_sim` and the game's data. `game publish` builds a room-host binary for that game that links the rules crate and this series' room library.
2. **One protocol, ours.** Rust on both sides. No other server's protocol is a goal.
3. **One client bridge.** `engine/src/modules/multiplayer` is the only online code a game calls: join, send inputs, receive patches, spectate.
4. **Deterministic.** The rules run the same step on the server, the client and offline. The room can compare state hashes to catch drift.
5. **No cross-series imports.** This series shares only `core` with the API series; it never imports it, and `tests/backend/modules.rs` enforces that.

## Parts of the room runtime

| Part | What it does | Stage |
|---|---|---|
| Transport | WebSocket at `/multiplayer/ws` | M0 |
| Room | Lifecycle: create, authorize, join, leave, message, dispose. One task per room | M0 |
| Client | A connected player: session id, send, leave | M0 |
| State and patches | Diffs the room's state and sends changed fields at a patch rate | M1 |
| Simulation tick | Steps the rules at a fixed interval | M1 |
| Matchmaker | Create, join, join-or-create, filters, max clients, lock | M2 |
| Authorization | Checks the account from the API series before a join | M2 |
| Spectators and replays | Read-only join, delay buffer, redaction, input recording | M3 |
| Reconnect and presence | Rejoin a match; who is where | M4 |
| Ratings and live rooms | Brackets as a wrapper over duel rooms; a 24/7 room | M5 |
| Scale-out | Shared presence and room cache; one container per host | M6 |

## Protocol (fixed at M0)

M0 decides the wire format. These are the requirements it must meet:

- Framed messages: a message name and a binary payload.
- A join and a reconnect handshake, each with a session token.
- Patches carry a version, so a late joiner gets a full snapshot and then patches.
- Every message type is listed in one place in `engine/src/modules/multiplayer` and in the room host, so both sides cannot drift.

**Chosen at M0 (2026-10-22).** One binary WebSocket message is one frame: byte 0 is the length of the
name (at most 64), then the name as UTF-8, then the payload as raw bytes. The names are in
`backend/src/modules/multiplayer/protocol.rs`: `join` (payload: the room id), `joined` and
`peer_joined` / `peer_left` (payload: a session id, 8 bytes big-endian), `error` (a reason),
`message` (relayed by the room to the other clients, unchanged). The socket is `/multiplayer/ws`.
A client's first frame must be `join`; otherwise the socket closes. M0 has no session token and no
reconnect yet (M2 and M4); a room takes 8 clients by default.

The room is one task per room, in memory, in the backend process (`room.rs`). A later stage moves
the rules in: `message` carries the game's inputs and the room steps the rules (M1).

## Deployment

- Each game has its own room host, built by `game publish`. The proxy routes `/multiplayer/<game>` to that host.
- The global API is a separate service (`backend/` with `SERVICES=api` in production).
- In development, `backend run dev` starts the room library in the backend process, with no game rules, for testing the transport and the matchmaker.

**Not yet built:** the per-game room host and the `SERVICES` split are planned for M0 and M1. The code today runs the multiplayer series inside the backend process only.
