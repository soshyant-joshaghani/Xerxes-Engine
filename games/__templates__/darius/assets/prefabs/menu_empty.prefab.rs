//! A **structurally empty cell** of the main menu's grid: a combination of the taxonomy that
//! cannot exist (Solo x Role). Only an outline, with its label as the scene's child.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const LINE: Color = Color::srgb(0.169, 0.208, 0.259);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Empty Cell")
            .with(Node {
                width: px(196),
                height: px(58),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            })
            .with(BorderColor::all(LINE)),
    }
}
