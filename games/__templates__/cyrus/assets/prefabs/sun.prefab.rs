//! The **sun**: the one directional light an outdoor 3D scene needs, pointed at the middle of
//! the world and casting shadows.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Sun")
            .with(DirectionalLight {
                shadows_enabled: true,
                ..default()
            })
            .with(Transform::from_xyz(4.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y)),
    }
}
