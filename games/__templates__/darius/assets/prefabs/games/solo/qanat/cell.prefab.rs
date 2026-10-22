//! One **cell of Qanat's room**: a square of ground the player lays a channel tile on. The room
//! *is* these cells — `src/qanat` reads its rules from them — so every one carries its place and
//! what it is made of. The prefab is a plain soil cell; an instance says which cell it is, where
//! it sits, and (when it is not soil) what colour it is:
//!
//! ```ignore
//! Object::prefab(crate::assets::prefabs::games::solo::qanat::cell_prefab::prefab)
//!     .named("Cell 0,0")
//!     .with(QanatCell { col: 0, row: 0, ground: Ground::Spring })
//!     .with(SpriteDef { image: None, color: SPRING, size: Some(Vec2::new(52.0, 52.0)) })
//!     .with(Transform::from_xyz(-54.0, 118.0, 0.1))
//! ```
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

use crate::qanat::{Ground, QanatCell};

const SOIL: Color = Color::srgb(0.56, 0.43, 0.31);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Cell")
            .with(QanatCell {
                col: 0,
                row: 0,
                ground: Ground::Soil,
            })
            .with(SpriteDef {
                image: None,
                color: SOIL,
                size: Some(Vec2::new(52.0, 52.0)),
            })
            .with(Transform::from_xyz(0.0, 0.0, 0.1)),
    }
}
