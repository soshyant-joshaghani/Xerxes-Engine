//! Engine modules: what games build on. Bevy only, no Dioxus. Gameplay never lives here;
//! it goes in a game under `games/`.
//!
//! - `runtime`   the window setup shared by every platform
//! - `scene`     a hand-written game's Bevy world, and `launch` to run it
//! - `assets`    the `.rs` asset model: scenes, prefabs, metas, bundles
//! - `bundles`   shipping files in bundles: download, extract, `BundleTexture`
//! - `camera`    gameplay GameCamera2D / GameCamera3D and editor viewport camera markers
//! - `clock`     the wall clock and the UTC calendar day (daily seeds), the same on web and native
//! - `flow`      the game flow (Preload → Splash → MainMenu → Level) and game modes
//! - `input`     the input manager: actions bound to keyboard, mouse, gamepad and touch
//! - `matches`   the level's match: `xerxes_sim` rules for the game mode, reports in, events out
//! - `physics`   2D physics (the `xerxes_sim` world in Bevy): bodies, sensors, contacts, movers
//! - `project`   project settings (which configure the input map) and `launch_project`
//! - `safe_area` the device-reserved insets (Android nav bar, display cutout) to keep UI clear of
//! - `storage`   small persistent text storage for games (local storage, a data file, app storage)
//! - `ui`        the Canvas (overlay, camera, world space) and widgets (Toggle, Slider, Dropdown)

pub mod assets;
pub mod bundles;
pub mod camera;
pub mod clock;
pub mod flow;
pub mod input;
pub mod matches;
pub mod physics;
pub mod project;
pub mod runtime;
pub mod safe_area;
pub mod scene;
pub mod storage;
pub mod ui;
