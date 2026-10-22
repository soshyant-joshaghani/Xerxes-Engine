//! One **daric**: a gold coin lying in the courtyard, picked up by walking over it. It is the
//! courtyard's `pickup` with the coin on it, so only its place differs from one to the next.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

use crate::ahrimans_horde::Daric;

const COIN: Color = Color::srgb(0.98, 0.82, 0.30);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::prefab(
            crate::assets::prefabs::games::solo::ahrimans_horde::pickup_prefab::prefab,
        )
        .named("Daric")
        .with(Daric)
        .with(SpriteDef {
            image: None,
            color: COIN,
            size: Some(Vec2::new(14.0, 14.0)),
        }),
    }
}
