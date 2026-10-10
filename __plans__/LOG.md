# Log

A few dated lines per day, newest first: what was done, what was learned, what is next. For the days you are away. (See the daily rhythm in [EXECUTION.md](EXECUTION.md).)

## 2026-10-10

- **Done (evening):** `template run` / `game run` (alias of `dev`, plus `--built`) in `__ctrl__`. Darius (`template run 1 android`) and the engine app (`engine run android`) built, installed and ran on a Redmi 23090RA98G (Android 16, Mali-G610, Vulkan). Darius reached Coin Dash level 1 on the phone.
- **Verified:** `test engine` (all suites + web check) and `test templates` (darius + web check) green.
- **Learned:** the first Android dev build is about 15 min per app (Bevy for aarch64); Xiaomi needs *Install via USB* on and a tap on the install prompt; an old APK signed with another key needs `adb uninstall dev.xerxes.engine` first.
- **Still not checked by eye:** Darius menu/arena layout and touch feel on the phone, `template dev darius web`.
- **Done:** the Darius games are renamed (`darius`, `cyrus`). Stage 3.0: Darius is on the asset model (project file, `menu.scene.rs`, `arena.scene.rs`, a logic module); `MainMenuDef.scene` lets a scene be the main menu and `LevelButton(n)` starts a level. Stage 3.1: new Bevy-free crate `xerxes_sim` (`engine/sim/`, Rapier 0.36 with enhanced determinism: bodies, layers, sensors, contact events, character mover, rays, state hash) and `modules/physics` over it (`Body2d`, `Mover`, `ContactEvent`); Darius collision now uses it. Design cards for the 21 mini-games are in `__docs__/darius-games.md`.
- **Verified:** sim crate 8 tests, engine all test binaries green (incl. flow with a menu scene, physics), build crate 19, backend editor and engine tests, `cargo check` for the web target (engine and Darius). Darius ran 60 s on Windows with no error.
- **Not verified:** Darius on screen (menu and arena) and on the web with `template dev`, Android; deterministic-mode cost on the phone. `test all` itself was not run (its target folder is on E:, which is nearly full); the same cargo steps were run with targets on C:.
- **Learned:** `bevy_rapier` is not used (a second physics world next to the server's); the character mover must ignore sensors or it stops before a coin; Python edits on Windows turn LF files into CRLF, which broke the text-matching settings test (fixed).
- **Next:** stage 3.2, first slice (see the Next step line in `PROGRESS.md`).
- **Done (later the same day):** stage 3.2, the match runtime. The game mode types moved to `xerxes_sim::mode`; `xerxes_sim::matches` plays them (Lobby, Countdown, Playing, RoundEnd, Results; Score, Objective, Role and Survival; best-of rounds; time limit, respawn, permadeath, shrinking zone, low gravity, fog/escort/moving-objective flags; ranking with shared places). `modules/matches` runs the level's match in Bevy (`MatchReport` in, `MatchEvent` out, `MatchPause`, `RulesTweak`). Darius is now "Coin Dash", a Solo x Objective level with a 90 s limit, a countdown and a result banner.
- **Verified:** sim crate 22 tests (14 match, 8 physics), clippy clean; all engine test binaries incl. matches (3) and flow; `cargo check` for web (engine and Darius); Darius ran 70 s on Windows with no error. Not seen on screen.
- **Next:** stage 3.3, first slice (see the Next step line in `PROGRESS.md`).

## 2026-10-06

- **Done:** repo reorganized: backend split into `api` and `multiplayer` series (`SERVICES=all|api|multiplayer`); typed assets `Name.Type.rs` with one rule in `xerxes_build::types`; the engine's own `starter` project (no more `game-template`); add-ons (`engine/addons`) embedded in the build crate; template games read-only in the editor and backend; `xerxes-build` is now `engine/build`, `vendor/winit` is `engine/winit`; docs moved to `__docs__`; CLI is `backend run dev|prod` (no more `dev run all`).
- **Plan:** Phase 3 is Darius (`games/__templates__/darius`): a mini-game for every game mode, offline, online and spectated, with Warp, quests, dialogue, save and the Colyseus-port multiplayer track (M0 to M6). Physics is Rapier. See `PROGRESS.md`, `__docs__/darius.md`.
- **Verified:** `test all` green; engine and Darius build and run on web, Windows and the phone (Android). The Project Manager lists Starter and Darius, and creating a project from the starter works on the phone.
- **Learned:** this Windows PC reserves port ranges (5100, 5200 and adb's 5037 fail with error 10013), so dev servers fall back to the next free port and adb uses `ANDROID_ADB_SERVER_PORT=5043`; a full drive shows up as a linker error (LNK1140); the phone needs "Install via USB" and unlocked screen for `adb install`.
- **Gaps seen:** Darius on the phone has no touch controls yet (hint still says WASD); pointer lock is documented, not built (`__docs__/input.md`).
- **Decided:** scripting (Python or any other) is out of the plan for now; it can wait until the APIs are settled and several games exist.
- **Next:** stage 3.0, first slice (see the Next step line in `PROGRESS.md`).
