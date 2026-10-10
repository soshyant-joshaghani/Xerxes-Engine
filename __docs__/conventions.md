# Conventions

**Engine.** `engine/src/modules/` holds engine modules only: no gameplay, no game assets. Something a game needs once stays in the game; it moves into the engine only after several games repeat it and it is clearly generic.

**Games.** A game is created with `game new <name>` and lives in `games/<name>/`. It is its own crate (and Git repository, except the Darius games, which are committed with the engine) and depends on `../../engine` with `default-features = false`. Only finished games move to `games/__templates__/`. One folder per scene under `src/` (`mod.rs` with `impl Scene`, `world.rs` for Bevy systems, `hud.rs` for the Dioxus HUD and the bridge types). Assets go in the game's `assets/` (GLB/glTF from Blender; fix orientation at export, not in game code).

**Bridge.** The HUD never touches the ECS and the world never touches the DOM. Each scene defines its own `Command` and `Snapshot` types; keep snapshots to what the HUD shows.

**Backend.** Product code goes in `backend/src/modules/api/<name>/` (router → service → repository → schemas); platform code stays in `api/base/`, `api/system/`, `api/engine/` and `api/editor/`. Realtime code goes in `backend/src/modules/multiplayer/`; the two series never import each other. JSON fields are `snake_case`, paths stay under `/api/v1`, errors are `{"detail": "..."}`, tables follow [CONTRACT.md](../../../CONTRACT.md).

**Platforms.** Gate platform behaviour with `cfg(target_arch = "wasm32")`, `cfg(target_os = "...")` or a cargo feature inside the module that needs it.
