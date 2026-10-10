//! Xerxes engine. Games are Bevy: the world and its UI (bevy_ui). The engine's own editor
//! (feature `editor`) adds Dioxus for the editor UI; games never build it.
//!
//! A game made of `.rs` assets (`assets/`, see `modules::assets`, `modules::project`) runs with
//! `xerxes_engine::launch_project(assets::project())`. A hand-written game describes a
//! [`Scene`] and calls [`launch`]:
//!
//! ```ignore
//! fn main() {
//!     xerxes_engine::launch::<MyScene>();
//! }
//! ```

pub mod modules;

/// The engine app (feature `editor`): Dioxus editor UI over a Bevy viewport, shared by the
/// desktop/web binary and the Android entry (`platform/android/main.rs`).
#[cfg(feature = "editor")]
pub mod editor;

pub use modules::project::launch_project;
pub use modules::scene::{Scene, launch};

/// What `.rs` assets (scenes, prefabs, metas, project settings, logic modules) import:
/// Bevy's prelude and the engine's asset types.
pub mod prelude {
    pub use crate::modules::assets::*;
    pub use crate::modules::bundles::*;
    pub use crate::modules::flow::*;
    pub use crate::modules::project::*;
    pub use bevy::prelude::*;
}

/// Includes the modules `xerxes_build` generated from the game's `assets/` folder, as
/// `pub mod assets` (with `assets::project()`). Call it once in the game's `lib.rs`.
#[macro_export]
macro_rules! include_assets {
    () => {
        include!(concat!(env!("OUT_DIR"), "/xerxes_assets.rs"));
    };
}
