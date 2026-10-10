//! Top-down rules for the arena level: a player square that slides along the scene's walls (the
//! engine's physics module),
//! coins to collect. The HUD shows the score and can pause and restart. The arena itself
//! (floor, walls, camera) is the scene `assets/scenes/arena.scene.rs`.
//!
//! - `world`  the Bevy systems, and the state and actions the HUD uses
//! - `hud`    the bevy_ui overlay

mod hud;
mod world;

use bevy::prelude::*;
use xerxes_engine::modules::matches::Rules;
use xerxes_engine::modules::physics::PhysicsPlugin;

/// The mode's default rules with this game's goal (every coin) and a short countdown.
pub fn tweak_rules(rules: Rules) -> Rules {
    rules.goal(world::COIN_COUNT as u32).countdown(60)
}

/// Adds the physics, the rules and the HUD (called by `assets/logic/top_down.rs`).
pub fn plugin(app: &mut App) {
    app.add_plugins((
        PhysicsPlugin::top_down(),
        world::TopDownPlugin,
        hud::HudPlugin,
    ));
}
