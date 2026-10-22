//! A **column head** of the main menu's grid: as wide as a tile, with its title as the scene's
//! child.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Head").with(Node {
            width: px(196),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            ..default()
        }),
    }
}
