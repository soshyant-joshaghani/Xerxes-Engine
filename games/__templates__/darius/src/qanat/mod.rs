//! Qanat (Solo · Objective): lay channel tiles in a cut-away hillside so the spring's water runs
//! down to the village; fewer tiles earn a better tier. Design:
//! `__docs__/gdd/darius/solo-objective-qanat.md`.
//!
//! Rules live in [`xerxes_sim::qanat`] (shared with Cyrus's Arash's Arrow). `play` is the Bevy
//! side: reads the room from the scene's cells, takes pointer / touch / key input, runs the water
//! on the fixed clock, draws the tiles and fills the HUD.
//!
//! The room itself is the scene `assets/scenes/games/solo/qanat.scene.rs`.

mod play;

pub use play::{
    BoardCamera, CellCursor, QanatAction, QanatButton, QanatCell, ResultBanner, ResultText,
    StatusText,
};
pub use xerxes_sim::qanat::Ground;

use bevy::prelude::*;

/// Adds Qanat's systems (called by `src/logic.rs`). They only act on a level whose
/// scene has Qanat's cells.
pub fn plugin(app: &mut App) {
    if !app.is_plugin_added::<crate::layout::LayoutPlugin>() {
        app.add_plugins(crate::layout::LayoutPlugin);
    }
    app.add_plugins(play::QanatPlugin);
}
