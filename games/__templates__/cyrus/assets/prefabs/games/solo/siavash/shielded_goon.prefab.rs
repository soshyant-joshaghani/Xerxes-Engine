//! A **shielded goon**: a goon with a gold shield on it. Only a dive strikes it (the rules say so),
//! so it looks different from a plain goon. `src/siavash` spawns one of these for every shielded
//! goon the rules put on the road.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const SHIELDED: Color = Color::srgb(0.85, 0.62, 0.20);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Shielded Goon")
            .with(MeshDef::Cuboid {
                size: Vec3::new(0.9, 1.0, 0.9),
            })
            .with(MaterialDef {
                color: SHIELDED,
                texture: None,
            })
            // Its feet on the road; the game moves it along +Z as the knight runs.
            .with(Transform::from_xyz(0.0, 0.5, 0.0)),
    }
}
