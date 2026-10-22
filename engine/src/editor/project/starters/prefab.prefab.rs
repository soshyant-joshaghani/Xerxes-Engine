//! Template: an empty prefab. Put what every use of it has here, and let a scene add what
//! varies: an instance replaces components by type and appends children.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("__NAME__"),
    }
}
