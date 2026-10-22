//! The **ghost knight**: the best run so far, ridden again on the same road beside the knight. It is
//! a pale block, not the knight, so the player can tell them apart. `src/siavash` spawns one of these
//! and places it at the ghost's lane, height and distance every frame.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const GHOST: Color = Color::srgb(0.66, 0.82, 0.95);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Ghost Knight")
            .with(MeshDef::Cuboid {
                size: Vec3::new(0.8, 1.8, 0.8),
            })
            .with(MaterialDef {
                color: GHOST,
                texture: None,
            })
            // Its feet at the ghost's height; the game moves it along +Z as the knight runs.
            .with(Transform::from_xyz(0.0, 0.9, 0.0)),
    }
}
