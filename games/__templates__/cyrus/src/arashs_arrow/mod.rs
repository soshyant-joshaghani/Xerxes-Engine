//! Arash's Arrow (Solo · Objective). Slice 1 is a 3D diorama: Qanat's channel rules on a 3D
//! diorama with an orbit camera. The game becomes a Neon White-style speedrun at R9. Design:
//! `__docs__/gdd/cyrus/solo-objective-arashs-arrow.md`.

mod play;

pub use play::{
    GateAction, GateBoard, GateButton, GateCamera, GateCell, GateCursor, ResultBanner, ResultText,
    StatusText,
};
pub use xerxes_sim::qanat::Ground;

use bevy::prelude::*;
use xerxes_engine::modules::matches::MatchPlugin;

/// Adds Arash's Arrow systems (called by `src/logic.rs`).
pub fn plugin(app: &mut App) {
    // Darius gets this from Ahriman's Horde; Cyrus had no match until this diorama.
    if !app.is_plugin_added::<MatchPlugin>() {
        app.add_plugins(MatchPlugin);
    }
    app.add_plugins(play::ArashsArrowPlugin);
}
