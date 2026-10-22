//! Sofreh (Duel · Score): the kitchen cook-off. Slice 1 is one shared kitchen, two cooks (one per
//! team), one dish, delivery and the clock. Design: `__docs__/gdd/darius/duel-score-sofreh.md`.
//!
//! The rules live in [`rules`] (Bevy-free, tested there); [`play`] connects them to the scene.

pub mod play;
pub mod rules;

pub use play::{
    SofrehBanner, SofrehBoard, SofrehCook, SofrehGrill, SofrehPlugin, SofrehResult, SofrehStatus,
};

use bevy::prelude::*;

/// Adds Sofreh's systems (called by `src/logic.rs`). They only act on a level with its board.
pub fn plugin(app: &mut App) {
    app.add_plugins(SofrehPlugin);
}

#[cfg(test)]
mod tests {
    use super::*;
    use xerxes_engine::modules::assets::spawn_scene;

    #[test]
    fn scene_has_the_kitchen_cooks_and_hud_markers() {
        let mut world = World::new();
        spawn_scene(
            &mut world,
            &crate::assets::scenes::games::duel::sofreh_scene::scene(),
        );

        assert_eq!(world.query::<&SofrehBoard>().iter(&world).count(), 1);
        let mut cooks = world.query::<&SofrehCook>();
        let mut cook_ids: Vec<_> = cooks.iter(&world).map(|cook| cook.0).collect();
        cook_ids.sort_unstable();
        assert_eq!(cook_ids, [0, 1]);
        assert_eq!(world.query::<&SofrehGrill>().iter(&world).count(), 1);
        assert_eq!(world.query::<&SofrehStatus>().iter(&world).count(), 1);
        assert_eq!(world.query::<&SofrehBanner>().iter(&world).count(), 1);
        assert_eq!(world.query::<&SofrehResult>().iter(&world).count(), 1);
    }
}
