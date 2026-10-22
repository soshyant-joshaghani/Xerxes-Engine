//! The **camera** of an empty 3D scene: a fixed three-quarter view of the middle of the world.
//! A game that is being built gives its own camera its own pose and components instead.
//!
//! It carries the game's `Sky`, the colour the camera clears to. A 3D camera fills the window, so
//! everything above the ground is sky; leaving it the dark HUD colour made the sky and the header
//! bar one shape, and collapsing the header then looked like it made the bar taller. A scene with
//! a sky of its own overrides this one (`.with(Sky(...))` on the instance).
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

/// Daylight haze over the desert: lighter than the ground, so the horizon reads.
const SKY: Color = Color::srgb(0.608, 0.686, 0.761);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Camera")
            .with(GameCamera3D)
            .with(Sky(SKY))
            .with(Transform::from_xyz(0.0, 9.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y)),
    }
}
