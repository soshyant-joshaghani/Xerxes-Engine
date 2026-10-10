# Physics: Rapier

**Decision (2026-10-06): Xerxes uses [Rapier](https://rapier.rs) for 2D and 3D physics.** On the Bevy side through `bevy_rapier2d` / `bevy_rapier3d`; in the Rust multiplayer server and in the shared simulation core through the plain `rapier2d` / `rapier3d` crates (no Bevy). Character movement builds on Rapier's own character controller, with `bevy-tnua` as the optional higher-level controller.

**Built in stage 3.1: we do not use `bevy_rapier`.** The Bevy plugin in the Rapier docs ("Getting started with Bevy") is a second layer: its own components, its own step schedule, its own transform sync. We would then have two worlds, the plugin's in the game and the plain Rapier one in `xerxes_sim` on the server, and nothing guarantees they match. Instead the game uses `xerxes_sim::Physics2d` (plain `rapier2d` 0.36, `enhanced-determinism`) directly, and `engine/src/modules/physics` is a thin Bevy plugin over it: a `Body2d` component adds a body, `Mover` walks a kinematic body, sensor and touch events arrive as `ContactEvent` messages, and the `Transform` follows the body. One physics world, one set of numbers, on the client, the server and in a replay. `bevy_rapier` stays out unless a need appears that the plain crate cannot meet (its debug renderer, say).

This was a decision for Phase 3 stage 3.1 ([PROGRESS.md](../__plans__/PROGRESS.md)). Until that stage, nothing depends on it; `games/__templates__/darius` still uses its hand-written sliding collision.

## What decides it

Xerxes is not choosing a physics plugin for one game. Three long-term needs pick the engine, in this order:

1. **The same physics on the client and on the server.** The multiplayer server is a Rust port of Colyseus with no Bevy in it ([multiplayer.md](multiplayer.md)). Server-authoritative play, client prediction and "every game mode runs offline too" all need one simulation that runs in both places. Rapier is a standalone engine: `rapier2d` / `rapier3d` run inside Axum with no ECS, and `bevy_rapier` is only the glue that puts the same engine into Bevy. Avian is built on Bevy's ECS, so a server would have to run a headless Bevy app just to use it.
2. **Determinism, for replays and spectating.** A spectatable match can be recorded as its inputs and replayed bit for bit; rollback and lockstep netcode need the same. Rapier has an `enhanced-determinism` feature: the same simulation gives the same result on any platform that follows IEEE 754-2008, WASM included ([Rapier determinism guide](https://rapier.rs/docs/user_guides/rust/determinism/)). It cannot be combined with its SIMD fast path, so it costs speed; we turn it on for simulations that must agree (matches) and can leave it off for cosmetic physics.
3. **2D and 3D from one family**, because scenes are 2D or 3D and a game may mix both.

## The candidates

| Crate | What it is | Bevy 0.18 | Verdict |
|-------|------------|-----------|---------|
| [Rapier](https://github.com/dimforge/rapier) + [`bevy_rapier`](https://github.com/dimforge/rapier/tree/master/bindings/bevy_rapier) | The reference Rust physics engine; 2D + 3D; the Bevy plugin now lives in the Rapier repository. The same engine as rapier.js (which you know), natively. | Yes: `bevy_rapier` 0.32/0.33 line works with Bevy 0.17 to 0.18 (verify the exact pair when adopting). | **Chosen.** Runs without Bevy, cross-platform deterministic mode. |
| [Avian](https://github.com/avianphysics/avian) (`avian2d`, `avian3d`) | A Bevy-native ECS physics engine (formerly bevy_xpbd). Nicer ergonomics inside Bevy. | Avian 0.5 and 0.6 for Bevy 0.18; 0.7 is for 0.19. | **Not chosen.** Bevy-only, so the server cannot share it; its docs list cross-platform determinism as a future feature. Revisit if it gains both. |
| [`bevy-tnua`](https://github.com/idanarye/bevy-tnua) | A floating character controller (run, jump, coyote time, slopes, moving platforms, wall slide, climbing) over Rapier or Avian, 2D and 3D. | Yes (0.29 to 0.31 with the Rapier 0.33 / Avian 0.6 pair). | **Adopt as an optional piece** of the add-ons (`chr-ctrl`) for platformer and first-person movement. The server-side movement for authoritative play uses Rapier's own character controller. |
| [`bevy_fpc`](https://codeberg.org/Eternahl/bevy_fpc) | A first-person controller plugin on Rapier's character controller. | Last released for Bevy 0.15 (May 2025). | **No.** Too far behind; build the FPS controller from Rapier's controller or Tnua plus our camera logic (Phase 5). |
| `bevy_physimple` | A simple 2D collision plugin. | Last release targets Bevy 0.8. | **No.** Abandoned. |
| `physme` | A small 2D physics plugin. | Could not be confirmed. | **No.** Unknown maintenance, and tiny next to Rapier. |

Everything above that says "yes" for Bevy 0.18 was checked against the crates' own pages on 2026-10-06. The Bevy versions move every few months: re-check the pairing before stage 3.1 starts.

## How it fits the engine

- **Optional module, not a given.** A visual novel needs no physics, so it is `modules/physics` behind a game's Cargo feature (`physics-2d`, `physics-3d`), configured in the project settings (gravity, collision layers, fixed time step), like the other "every game may have it" pieces.
- **One simulation core, Bevy-free.** Match rules, scoring and the physics world live in a small crate (`xerxes_sim`, planned at `engine/sim/`, a new stable path like `engine/build/`) that has no Bevy. The Bevy client wraps it (rendering, input, UI); the server wraps it (rooms, snapshots). See [darius.md](darius.md).
- **Fixed time step and ordered stepping**, so the same inputs always give the same state (a requirement for replays and for spectators joining from a snapshot).
- **Collision layers and sensors are data.** The editor's Inspector edits them as typed components, in the same `.rs` scene format as everything else.
- **Characters:** Rapier's kinematic character controller for authoritative movement; Tnua on top for games that want its feel.

## Open questions for stage 3.1

- Which `bevy_rapier` / `rapier` pair exactly, and whether to pin to Bevy 0.18 or move to 0.19 first (AGENTS.md pins 0.18 "until there is a concrete reason"; a physics release gap could be one).
- The cost of `enhanced-determinism` on the target phones, measured on the Android device.
- Whether client prediction rolls back with Rapier snapshots or re-simulates from the last server state.
