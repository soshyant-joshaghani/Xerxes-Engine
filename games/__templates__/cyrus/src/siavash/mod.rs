//! Siavash (Solo · Score): a knight on an endless road of three lanes, running by himself. You
//! switch lane, jump and slide past gates, banners and columns; a hit ends the run. Design:
//! `__docs__/gdd/cyrus/solo-score-siavash.md`. This is slice 1: the run and the road, no demons
//! or gun yet.
//!
//! - `cyrus_rules::siavash` (the `rules/` crate): the Bevy-free run, shared with the room host
//! - `play`                  the Bevy side: input, the knight, the enemies, the camera, the header
//!
//! The knight, the camera, the road and the header are the scene
//! `assets/scenes/games/solo/siavash.scene.rs`; this module drives them through marker components.

mod play;

pub use play::{Banner, BannerText, DistanceText, Knight, RestartButton, Road, ShieldRing};

use bevy::prelude::*;

/// Adds Siavash's systems (called by `src/logic.rs`). They only act on a level whose scene has a
/// [`Knight`].
pub fn plugin(app: &mut App) {
    app.add_plugins(play::SiavashPlugin);
}
