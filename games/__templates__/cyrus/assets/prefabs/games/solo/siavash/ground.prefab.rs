//! The **ground** under Siavash's road: the desert the run crosses, 200 m of it, so the horizon is
//! past the fog. (The game's other scenes use the smaller `prefabs/ground`.)
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const GROUND: Color = Color::srgb(0.62, 0.53, 0.38);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Ground")
            .with(MeshDef::Plane { size: 200.0 })
            .with(MaterialDef {
                color: GROUND,
                texture: None,
            }),
    }
}
