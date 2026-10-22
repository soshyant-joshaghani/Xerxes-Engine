//! A **wall of the garden plot**: a unit cube scaled to the wall's size, with a fixed body of the
//! unscaled cube (the body follows the scale). Each one has its own place and size.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::modules::physics::{Body3d, Shape3d};
use xerxes_engine::prelude::*;

const WALL: Color = Color::srgb(0.55, 0.48, 0.40);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Wall")
            .with(MeshDef::Cube { size: 1.0 })
            .with(MaterialDef {
                color: WALL,
                texture: None,
            })
            .with(Transform::from_xyz(0.0, 0.75, -7.5).with_scale(Vec3::new(15.0, 1.5, 0.3)))
            .with(Body3d::fixed(Shape3d::Box {
                half: [0.5, 0.5, 0.5],
            })),
    }
}
