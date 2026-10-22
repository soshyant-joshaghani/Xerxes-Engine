//! One **dash of Chapar's road marking**. Every dash is the same stripe at the same height; only
//! its `Parallax` offset (where it sits along the loop) differs, so an instance replaces that:
//!
//! ```ignore
//! Object::prefab(crate::assets::prefabs::games::solo::chapar::dash_prefab::prefab)
//!     .named("Dash 2")
//!     .with(Parallax { factor: 1.00, offset: 160.0, span: 2240.0 })
//! ```
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

use crate::chapar::Parallax;

const DASH: Color = Color::srgb(0.90, 0.76, 0.55);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Dash")
            .with(Parallax {
                factor: 1.00,
                offset: 0.0,
                span: 2240.0,
            })
            .with(SpriteDef {
                image: None,
                color: DASH,
                size: Some(Vec2::new(70.0, 6.0)),
            })
            .with(Transform::from_xyz(0.0, -146.0, 0.2)),
    }
}
