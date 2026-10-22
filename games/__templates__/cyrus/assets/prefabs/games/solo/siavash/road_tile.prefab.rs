//! One **chunk of Siavash's road**: a 20 m slab the game lays ahead of the knight and takes away
//! behind him. Every other chunk is the darker colour, which the instance replaces.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const TILE: Color = Color::srgb(0.62, 0.53, 0.38);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Road Chunk")
            // 6 m wide (the three lanes), 20 m along the road: `play::CHUNK`.
            .with(MeshDef::Cuboid {
                size: Vec3::new(6.0, 0.05, 20.0),
            })
            .with(MaterialDef {
                color: TILE,
                texture: None,
            })
            // Just above the ground, so the two surfaces never share a plane (that made them
            // flicker); the game moves it along the road.
            .with(Transform::from_xyz(0.0, 0.035, 0.0)),
    }
}
