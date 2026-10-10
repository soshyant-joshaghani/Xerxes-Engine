//! The top-down rules: player, coins, collision and the HUD (`src/top_down`), and the match
//! they play: collect every coin before the time runs out.

use bevy::prelude::*;
use xerxes_engine::modules::matches::{MatchPlugin, RulesTweak};

pub fn plugin(app: &mut App) {
    app.insert_resource(RulesTweak(crate::top_down::tweak_rules))
        .add_plugins(MatchPlugin);
    crate::top_down::plugin(app);
}
