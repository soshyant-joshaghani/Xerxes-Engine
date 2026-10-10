//! Engine modules: what games build on. Bevy only, no Dioxus. Gameplay never lives here;
//! it goes in a game under `games/`.
//!
//! - `runtime`   the window setup shared by every platform
//! - `scene`     a hand-written game's Bevy world, and `launch` to run it
//! - `assets`    the `.rs` asset model: scenes, prefabs, metas, bundles
//! - `bundles`   shipping files in bundles: download, extract, `BundleTexture`
//! - `flow`      the game flow (Preload → Splash → MainMenu → Level) and game modes
//! - `matches`   the level's match: `xerxes_sim` rules for the game mode, reports in, events out
//! - `physics`   2D physics (the `xerxes_sim` world in Bevy): bodies, sensors, contacts, movers
//! - `project`   project settings, input actions and `launch_project`

pub mod assets;
pub mod bundles;
pub mod flow;
pub mod matches;
pub mod physics;
pub mod project;
pub mod runtime;
pub mod scene;
