//! A **narrow tile** of the main menu: the row of kinds that sit outside the taxonomy grid holds
//! more of them, so they are narrower. Everything else is the grid's tile — this prefab is that
//! one with a width of its own.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::prefab(crate::assets::prefabs::menu_tile_prefab::prefab).with(Node {
            width: px(146),
            height: px(58),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(px(10), px(6)),
            row_gap: px(2),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        }),
    }
}
