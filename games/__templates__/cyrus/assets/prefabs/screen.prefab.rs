//! The **screen**: a HUD canvas over the whole window that centres what it holds. Cyrus's empty
//! scenes put their title card in it.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("HUD").with(Canvas::overlay()).with(Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        }),
    }
}
