//! Pairidaeza (Sandbox): 3D physics lab (slice 0) — spawn cubes and spheres, drag them,
//! tilt gravity. Garden-builder slices come later. Design:
//! `__docs__/gdd/cyrus/sandbox-pairidaeza.md`.

mod play;

pub use play::{
    GravityLabel, GravitySlider, HintText, LabCamera, ModeLabel, Plot, SpawnKind, ToolMode, Toy,
    ToyKind, WorkshopAction, WorkshopButton,
};

use bevy::prelude::*;
use xerxes_engine::modules::physics::Physics3dPlugin;

/// Adds 3D physics and the lab systems (called by `src/logic.rs`).
pub fn plugin(app: &mut App) {
    if !app.is_plugin_added::<Physics3dPlugin>() {
        app.add_plugins(Physics3dPlugin::earth());
    }
    app.add_plugins(play::PairidaezaPlugin);
}
