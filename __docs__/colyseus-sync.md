# Colyseus sync: the pin, the mirror and the port map

The multiplayer series is a Rust port of [Colyseus](https://github.com/colyseus/colyseus) ([multiplayer.md](multiplayer.md)). Colyseus keeps changing, so the port is always **pinned** to one upstream commit, and every later upstream change is brought in as a reviewed diff, never as a silent sync.

## The pin

| | |
|---|---|
| Upstream | `https://github.com/colyseus/colyseus` |
| **Pinned release** | tag `colyseus@0.18.9` |
| **Pinned commit** | `992b45f2f8f163d11da1b4c4d88d3bc3b9975f46` |
| Licence | MIT, `Copyright (c) 2015-2026 Endel Dreyer` (keep this notice in ported code and in a `NOTICE` file when M0 starts) |
| Moving head, for reference | `bba9bf678224d5ba53b50b15f3c9590ebe248b0d` (2026-10-13). Not pinned: `HEAD` changes daily |

**Why a release tag and not `HEAD`:** a tag is a fixed, reviewed state with a version number. The port targets it and the mirror moves the pin one tag at a time. Pinning `HEAD` would make every day's commits part of the first port.

**Status:** the pin is chosen; nothing is ported yet. The port starts at step 5 of the [line in PROGRESS.md](../__plans__/PROGRESS.md#the-line) (M0).

To see the newest stable tags:

```bash
git ls-remote --tags --sort=-v:refname https://github.com/colyseus/colyseus.git 'refs/tags/colyseus@*'
```

## Where each upstream package lands

From the `packages/` folder at the pin. Verify each row when its stage starts; a row can change then.

| Upstream package | Xerxes home | Stage |
|---|---|---|
| `core` | `backend/src/modules/multiplayer/room` (Room trait, lifecycle, simulation interval, lock, `maxClients`, auth hook) and `matchmaker` | M0, M2 |
| `transport` | `multiplayer/transport` (WebSocket upgrade at `/multiplayer/ws`) | M0 |
| `serializer` | `multiplayer/state` (state encoding and delta patches) | M0 decides, M1 builds |
| `sdk` | reference for the client; the Bevy client lives in `engine/src/modules/net` | M0 |
| `presence` | `multiplayer/presence` (local first, Redis later) | M4 |
| `drivers` | Redis in `core` (room cache, presence) | M6 |
| `testing` | reference for conformance tests | M0 onward |
| `auth` | the API series' accounts (`backend/src/modules/api`), not a port | M2 |
| `database` | not ported: PostgreSQL is the system of record ([backend.md](backend.md)) | - |
| `shared-types`, `admin`, `monitor`, `playground`, `tools`, `traefik`, `loadtest`, `example`, `greeting-banner`, `room-plugins`, `better-call` | not ported (dev tooling, or the HTTP layer Axum already is); read only when a stage needs one | - |

The state schema (the encoding Colyseus uses for patches) is believed to live in a separate library, `@colyseus/schema`, not in this monorepo. Check that at M0: it decides how much of the wire format the port must copy.

## Open decisions (made at M0, before step 5's code)

1. **Wire compatibility.** If the Rust server speaks the Colyseus protocol, the existing JS SDKs work against it, and an Xerxes game gets a client for free. If not, Xerxes needs its own Bevy client only. The choice shapes every later stage ([multiplayer.md](multiplayer.md#how-we-port)).
2. **State encoding.** Match the Colyseus schema encoding, or a Xerxes encoding with the same delta-patch behaviour.
3. **Sync cadence.** Proposal: mirror on each new `colyseus@` tag, not on every commit. Ad hoc only when a bug or a needed feature points at upstream.

## The mirror: how an upstream change becomes our change

Each sync is one tag (or one commit) at a time, from the pin to the target.

1. **Pick the target.** A new tag from the command above, or a commit SHA the user names.
2. **Diff the mapped packages only.** In a scratch clone (outside the repo), run `git diff --stat <pin> <target> -- packages/core packages/transport packages/serializer packages/sdk packages/presence packages/testing`, then read each changed file.
3. **Classify every change:**
   - *wire protocol* (message framing, join or reconnect handshake, state encoding): the most important; it can break existing clients;
   - *behaviour* (room lifecycle, matchmaking, patch rate, timeouts, auth);
   - *API* (names and signatures our port mirrors);
   - *internal* (refactors, speed-ups): skip unless they fix a bug we have;
   - *dependency* bumps: skip unless we need them;
   - *tests and docs*: use as conformance input.
4. **Write the report** in `__plans__/colyseus-sync/<target>.md`: one row per change with the upstream commit, the file, the kind, the Xerxes file it touches, the decision (port, defer, or skip with the reason) and the conformance test it needs.
5. **The user approves the decisions.** Nothing is ported before this.
6. **Port on a branch.** Change only `backend/src/modules/multiplayer/` (and `engine/src/modules/net` when the wire changes). Conformance tests use fixtures recorded from the old and the new pinned server, in `tests/backend/multiplayer/fixtures/<tag>/`. Each ported file names its source in its header, for example `// Ported from colyseus@0.18.9 packages/core/src/Room.ts`.
7. **Move the pin** only when `test all` is green: update the table above and add a dated line to [LOG.md](../__plans__/LOG.md).

## Brief for a sync agent

Paste this into a fresh agent, with the target filled in. It is the only context the agent gets, so it is complete on purpose.

> Read `__docs__/colyseus-sync.md` and `__docs__/multiplayer.md` in the Xerxes repo. Pinned upstream: `colyseus@0.18.9` (`992b45f2f8f163d11da1b4c4d88d3bc3b9975f46`). Target: `<target tag or SHA>`. Clone upstream outside the repo, diff only the packages listed in step 2 of the mirror section, and classify every change as the doc says. Write the report to `__plans__/colyseus-sync/<target>.md` with one row per change. **Do not edit any code, and do not move the pin.** Do not copy upstream code into the report beyond a short quote. Stop after the report and tell the user the wire-protocol changes first.

## Rules for the whole port

- Port behaviour and wire protocol, not Node's structure ([multiplayer.md](multiplayer.md#how-we-port)).
- The server stays Bevy-free; rules and physics come from `xerxes_sim`.
- A stage is `done` only when a darius scene uses it and its tests pass (see the line in [PROGRESS.md](../__plans__/PROGRESS.md)).
