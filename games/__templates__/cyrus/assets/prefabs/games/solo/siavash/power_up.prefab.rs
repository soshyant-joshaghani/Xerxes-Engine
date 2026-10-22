//! The **power-up**: a pickup on the road, in one lane. `src/siavash` spawns one of these for every
//! pickup the rules lay, and moves it toward the knight. The rules decide what the knight takes.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const COLOR: Color = Color::srgb(0.40, 0.90, 0.55);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Power Up")
            .with(MeshDef::Sphere { radius: 0.35 })
            .with(MaterialDef {
                color: COLOR,
                texture: None,
            }),
    }
}
