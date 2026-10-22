//! A **pickup**: something lying in the courtyard that is taken by walking over it — a fixed
//! sensor body fitted to its sprite. The darics, the gun and the heart are all this, each with its
//! own marker, colour and size (see `daric`, which is this prefab with the coin on it).
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::modules::physics::Body2d;
use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Pickup")
            .with(SpriteDef {
                image: None,
                color: Color::WHITE,
                size: Some(Vec2::new(16.0, 16.0)),
            })
            // Over the floor and the walls, under the player.
            .with(Transform::from_xyz(0.0, 0.0, 1.0))
            .with(Body2d::fixed_fit().sensor()),
    }
}
