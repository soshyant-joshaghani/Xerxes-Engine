//! A **flyer**: an enemy hanging over one lane, which a jump kills and a roll passes under. The
//! game spawns one for every flyer the rules put on the road.
//!
//! The sizes are the art; the rules decide what the knight hits
//! (`cyrus_rules::siavash::FLYER_BOTTOM` + `FLYER_HEIGHT` are the 1.536 and 0.6 below).
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const FLYER: Color = Color::srgb(0.45, 0.30, 0.70);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Flyer")
            .with(MeshDef::Cuboid {
                size: Vec3::new(1.0, 0.6, 0.6),
            })
            .with(MaterialDef {
                color: FLYER,
                texture: None,
            })
            // Its middle at FLYER_BOTTOM + FLYER_HEIGHT / 2: a jumping knight's head reaches it.
            .with(Transform::from_xyz(0.0, 1.836, 0.0)),
    }
}
