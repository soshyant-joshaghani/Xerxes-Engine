//! One **arrow of the D-pad** in the footer. All four are the same button; an instance says which
//! way it points (its `TouchButton` id) and adds its label.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Dpad Button")
            .with(Button)
            .with(Node {
                padding: UiRect::axes(px(16), px(10)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            })
            .with(BackgroundColor(GHOST)),
    }
}
