//! The game's logic plugins. Each one plugs a mode's code (`src/<mode>/`) into the project, and the
//! project file lists them in its `LOGIC`. Logic is project-wide, so each plugin's systems only
//! act on a level whose scene has their markers.

use bevy::prelude::*;
use xerxes_engine::modules::matches::{MatchPlugin, RulesTweak};

/// Chapar: the courier's run, its input, scrolling view and HUD. Acts only on a level with the rider.
pub fn chapar(app: &mut App) {
    crate::chapar::plugin(app);
}

/// Ahriman's Horde: courtyard combat (pistol, pad, rusher, warp portal). Adds the match once, for the
/// whole project.
pub fn ahrimans_horde(app: &mut App) {
    app.insert_resource(RulesTweak(crate::ahrimans_horde::tweak_rules))
        .add_plugins(MatchPlugin);
    crate::ahrimans_horde::plugin(app);
}

/// Jamshid's Workshop: spawn and drag toys, gravity slider. Acts only when a level has a Workbench.
pub fn jamshids_workshop(app: &mut App) {
    crate::jamshids_workshop::plugin(app);
}

/// Qanat: the channel puzzle's rules, input, water and HUD. Acts only on a level with Qanat's cells.
pub fn qanat(app: &mut App) {
    crate::qanat::plugin(app);
}
