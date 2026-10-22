//! A **row of the main menu's grid**: the row's label cell and its tiles, side by side.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Strip").with(Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(6),
            ..default()
        }),
    }
}
