//! One **mesa on Chapar's horizon**: a slab of rock that drifts by slowly. They share the colour
//! and the slow parallax; each one has its own size, height and place along the loop.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

use crate::chapar::Parallax;

const MESA: Color = Color::srgb(0.73, 0.50, 0.38);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Mesa")
            .with(Parallax {
                factor: 0.12,
                offset: 0.0,
                span: 2400.0,
            })
            .with(SpriteDef {
                image: None,
                color: MESA,
                size: Some(Vec2::new(170.0, 70.0)),
            })
            .with(Transform::from_xyz(0.0, -75.0, -5.0)),
    }
}
