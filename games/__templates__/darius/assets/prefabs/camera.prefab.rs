//! The **camera** of a flat scene: Darius is 2D, so this is the game camera with no projection of
//! its own (one pixel to one world unit). A game that is being built gives its camera its own
//! projection and components instead.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Camera").with(GameCamera2D),
    }
}
