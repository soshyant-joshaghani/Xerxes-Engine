//! A **wall or shelf of the workshop**: stone the toys cannot pass (a fixed body fitted to the
//! sprite). Each one has its own size and place.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::modules::physics::Body2d;
use xerxes_engine::prelude::*;

const WALL: Color = Color::srgb(0.32, 0.36, 0.45);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Wall")
            .with(SpriteDef {
                image: None,
                color: WALL,
                size: Some(Vec2::new(20.0, 420.0)),
            })
            .with(Transform::from_xyz(0.0, 0.0, 0.4))
            .with(Body2d::fixed_fit()),
    }
}
