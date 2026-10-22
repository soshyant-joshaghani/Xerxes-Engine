//! Jamshid's Workshop (Sandbox): a side-view physics toy table. Drop crates and balls, drag
//! them, tilt gravity with the slider. No win condition. Design:
//! `__docs__/gdd/darius/sandbox-jamshids-workshop.md`.
//!
//! The workbench (floor, walls, starter toys, camera and the HUD) is the scene
//! `assets/scenes/games/outside/jamshids_workshop.scene.rs`; this module drives it through
//! marker components. Slice 1: spawn, drag, gravity slider, reset.

mod play;

pub use play::{
    Ball, GravityLabel, GravitySlider, LabCamera, SpawnKind, Toy, Workbench, WorkshopAction,
    WorkshopButton,
};

use bevy::prelude::*;
use xerxes_engine::modules::physics::PhysicsPlugin;

/// Adds the side-view physics world (once) and the lab systems. Called by
/// `src/logic.rs`. They only act when a level has a [`Workbench`].
pub fn plugin(app: &mut App) {
    // Ahriman's Horde also adds PhysicsPlugin (top-down). First one wins; this level sets gravity
    // itself when the bench appears.
    if !app.is_plugin_added::<PhysicsPlugin>() {
        app.add_plugins(PhysicsPlugin::platformer(Vec2::new(0.0, -980.0)));
    }
    app.add_plugins(play::WorkshopPlugin);
}
