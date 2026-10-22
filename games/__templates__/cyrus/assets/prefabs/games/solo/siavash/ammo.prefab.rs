//! The **ammo** pickup: a pickup on the road, in one lane. `src/siavash` spawns one of these for every
//! pickup the rules lay, and moves it toward the knight. The rules decide what the knight takes.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const COLOR: Color = Color::srgb(0.30, 0.55, 0.95);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Ammo")
            .with(MeshDef::Cuboid {
                size: Vec3::splat(0.5),
            })
            .with(MaterialDef {
                color: COLOR,
                texture: None,
            }),
    }
}
