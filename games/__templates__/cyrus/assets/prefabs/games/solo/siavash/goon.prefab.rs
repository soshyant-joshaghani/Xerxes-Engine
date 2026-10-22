//! A **goon**: an enemy standing on the road, in one lane. `src/siavash` spawns one of these for
//! every goon the rules put on the road, and moves it toward the knight.
//!
//! The sizes are the art; the rules decide what the knight hits
//! (`cyrus_rules::siavash::GOON_HEIGHT` is the 1.0 below).
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const GOON: Color = Color::srgb(0.70, 0.18, 0.16);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Goon")
            .with(MeshDef::Cuboid {
                size: Vec3::new(0.9, 1.0, 0.9),
            })
            .with(MaterialDef {
                color: GOON,
                texture: None,
            })
            // Its feet on the road; the game moves it along +Z as the knight runs.
            .with(Transform::from_xyz(0.0, 0.5, 0.0)),
    }
}
