//! Zahhak (N-gon · Score): a serpent arena, you and bots, longest at the bell wins. Slice 1 is one
//! field with you and three bots: steering, pellets, deaths and the length score, on the match
//! and its participants. Design: `__docs__/gdd/darius/ngon-score-zahhak.md`.
//!
//! The field's rules live in [`rules`] (Bevy-free, tested there). `play` is the Bevy side: it steers
//! the serpents on the fixed clock, reports what they did to the match, draws them and fills the HUD.
//! The scene is `assets/scenes/games/ngon/zahhak.scene.rs`.

mod play;
pub mod rules;

pub use play::{
    ZahhakBanner, ZahhakField, ZahhakRestart, ZahhakResult, ZahhakStatus, ZahhakTrackPad,
};

use bevy::prelude::*;

/// Adds Zahhak's systems (called by `src/logic.rs`). They only act on a level whose match is up.
pub fn plugin(app: &mut App) {
    app.add_plugins(play::ZahhakPlugin);
}

#[cfg(test)]
mod tests {
    use super::*;
    use xerxes_engine::modules::assets::spawn_scene;

    fn fields_in(scene: &xerxes_engine::prelude::SceneAsset) -> usize {
        let mut world = World::new();
        spawn_scene(&mut world, scene);
        world.query::<&ZahhakField>().iter(&world).count()
    }

    #[test]
    fn the_scene_has_the_restart_and_a_track_pad_that_steers() {
        let mut world = World::new();
        spawn_scene(
            &mut world,
            &crate::assets::scenes::games::ngon::zahhak_scene::scene(),
        );
        assert_eq!(world.query::<&ZahhakRestart>().iter(&world).count(), 1);
        assert_eq!(world.query::<&ZahhakTrackPad>().iter(&world).count(), 1);
        let pads: Vec<String> = world
            .query::<&xerxes_engine::modules::ui::TouchPad>()
            .iter(&world)
            .map(|p| p.source.to_string())
            .collect();
        assert_eq!(pads, vec!["track".to_string()]);
    }

    #[test]
    fn only_the_zahhak_scene_has_the_arena() {
        assert_eq!(
            fields_in(&crate::assets::scenes::games::ngon::zahhak_scene::scene()),
            1
        );
        assert_eq!(
            fields_in(&crate::assets::scenes::games::solo::ahrimans_horde_scene::scene()),
            0
        );
        assert_eq!(
            fields_in(&crate::assets::scenes::games::solo::chapar_scene::scene()),
            0
        );
        assert_eq!(
            fields_in(&crate::assets::scenes::games::solo::qanat_scene::scene()),
            0
        );
    }
}
