//! One **button of the action pad** in the footer: Fire, Switch or Reload / Dash. They share the
//! shape and stretch to fill the pad; an instance says which action it is (a `TouchButton` id and
//! its `PadSlot`), adds its label, and the primary one replaces the background colour.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Pad Button")
            .with(Button)
            .with(Node {
                flex_grow: 1.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(8)),
                ..default()
            })
            .with(BackgroundColor(GHOST)),
    }
}
