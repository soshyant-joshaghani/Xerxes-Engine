//! A **row label** of the main menu's grid: the narrow cell on the left (SOLO, COOP, DUEL...),
//! with its name and subtitle as the scene's children.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Row Label").with(Node {
            width: px(96),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            ..default()
        }),
    }
}
