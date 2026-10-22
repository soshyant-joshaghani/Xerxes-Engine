//! Chapar (Solo · Score): a royal courier on an endless relay road. The rider runs by
//! himself; you jump (hold for height) and duck. A crash or a fall ends the run. The score is
//! metres + darics + grazes, times a chain multiplier (a relay token bursts it). Design:
//! `__docs__/gdd/darius/solo-score-chapar.md`. All of this lives in the **game** crate (offline
//! and online); the backend only hosts rooms, it does not own these rules.
//!
//! - `rules`  the Bevy-free run: rider, road, darics, grazes, score, stations
//! - `daily`  the day's seed from the date
//! - `ghost`  recording a run's inputs and riding them again beside the player
//! - `play`   the Bevy side: input, scroll, draw, HUD, match report
//!
//! The rider, the road, the scenery, the camera and the HUD are the scene
//! `assets/scenes/games/solo/chapar.scene.rs`; this module drives them through marker components.

pub mod daily;
pub mod ghost;
mod play;
pub mod rules;

pub use play::{
    Banner, BannerText, ChaparAction, DayText, DistanceText, GhostRider, HintText, HudButton,
    Parallax, PauseLabel, Rider, Road,
};

use bevy::prelude::*;

/// Adds Chapar's systems (called by `src/logic.rs`). They only act on a level whose
/// scene has a [`Rider`]. The match itself (`MatchPlugin`) is added once by Ahriman's Horde's logic
/// module, which every level shares, as Qanat relies on too; adding it here would panic.
pub fn plugin(app: &mut App) {
    app.add_plugins(play::ChaparPlugin);
}
