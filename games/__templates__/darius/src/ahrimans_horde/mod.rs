//! Ahriman's Horde (Solo · Survival). Planned as infinite waves, COD Zombies style (GDD). Slice 1 is
//! the old room-clear courtyard: pistol, Fire / Switch / Reload-Dash pad, one rusher; kill it, step
//! into the warp. Design: `__docs__/gdd/darius/solo-survival-ahrimans-horde.md`.

mod combat;
mod controls;
mod hud;
mod world;

pub use combat::{Enemy, GunPickup, HealthPickup, PadLabel, PadSlot};
pub use controls::{ControlSwitch, ControlSwitchLabel, Dpad};
pub use hud::{Banner, BannerText, HintText, HudButton, PauseLabel, ScoreText};
pub use world::{Action, Daric, Player};

use bevy::prelude::*;
use xerxes_engine::modules::matches::Rules;
use xerxes_engine::modules::physics::PhysicsPlugin;

/// Survival run: no score-target win; the portal reports Win. Darics still add score.
pub fn tweak_rules(rules: Rules) -> Rules {
    rules
}

/// Adds the physics, combat, HUD and pad (called by `src/logic.rs`).
pub fn plugin(app: &mut App) {
    if !app.is_plugin_added::<crate::layout::LayoutPlugin>() {
        app.add_plugins(crate::layout::LayoutPlugin);
    }
    if !app.is_plugin_added::<PhysicsPlugin>() {
        app.add_plugins(PhysicsPlugin::top_down());
    }
    app.add_plugins((
        world::TopDownPlugin,
        combat::CombatPlugin,
        hud::HudPlugin,
        controls::ControlsPlugin,
    ));
}
