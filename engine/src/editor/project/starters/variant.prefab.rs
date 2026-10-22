//! Template: a prefab **variant** — another prefab with changes of its own. A component on the
//! root replaces the one the base has of that type, and a child added here is added to the ones
//! the base already holds.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::prefab(__BASE__),
    }
}
