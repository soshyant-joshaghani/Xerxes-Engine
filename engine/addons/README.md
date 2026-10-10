# Add-ons

**Optional gameplay logic** a game copies and then owns: things a game may or may not have (a camera controller, a character controller, a HUD, pause). Nothing here is engine code, and nothing every game has.

What every game has lives in the engine, not here:

- scenes, prefabs, bundles (`BundleTexture` included), flow, game modes, input actions, project settings: `engine/src/modules/`
- the editor's starter scenes for New Scene (`2d.scene.rs`, `3d.scene.rs`): `engine/src/editor/project/starters/`

The other two things people call "templates" are different: the **starter** is the engine's empty new project (`engine/starter/`), and **template games** are finished games in `games/__templates__/` (read-only; copy one into your own project).

## How it is used

This folder is compiled into the `xerxes_build` crate (`engine/build/build.rs`), so the editor, the backend (`GET /engine/addons`, `POST /engine/projects/{name}/addons`) and the CLI all offer exactly this set, with or without the repo. Adding a module to a game copies it, and what it `needs`, into the game's `assets/logic/`, never overwriting a file the game already has. After that the game owns the copy and changes it freely.

Nothing builds these files in this repo: they are only compiled inside a game that copied them. Keep each one small and working against the current engine; a change to the engine that breaks one must fix it here.

## Layout

One folder per feature. A module's folder is only for finding it; every module lands in the project's `logic/` under its own file name, so file names are unique across feature folders (a test checks it).

| Folder | Holds |
|--------|-------|
| `camera-ctrl/` | Follow cameras (`camera_controller_2d.rs`, `_3d.rs`) |
| `chr-ctrl/` | Character controllers (`character_controller_2d.rs`, `_3d.rs`) |
| `hud/` | bevy_ui HUD (`hud.rs`) |
| `game/` | Game-level logic a game may want (`pause.rs`) |

## Adding a module

- Put it in the feature folder it belongs to, or start a new one (kebab-case, short). One `.rs` file with `pub fn plugin(app: &mut App)`.
- First doc line says what it does (`//! Template: ...`); `//! needs: a, b` names the modules it uses by file name (`pause`, not a path), wherever they live. A test checks that every summary exists and every need resolves.
- Ask first: does every game need it? Then it is an engine feature, not logic here. Gameplay only one game needs stays in that game (a goal zone, a specific enemy). Library logic is what a game proved worth reusing, and is generic.
- A feature folder created before its first file gets a `.gitkeep`; delete it with the first file. The build embeds only `.rs` files inside feature folders.
