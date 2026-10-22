//! The **ground**: the sand-coloured plane every empty 3D scene stands on. A scene that wants a
//! bigger or differently coloured one replaces the two defs on its instance.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const GROUND: Color = Color::srgb(0.62, 0.53, 0.38);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Ground")
            .with(MeshDef::Plane { size: 20.0 })
            .with(MaterialDef {
                color: GROUND,
                texture: None,
            }),
    }
}
