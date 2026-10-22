//! The **title card** of a game that is not built yet: the panel in the middle of the screen,
//! over the empty 3D scene. The scene adds the lines that are its own (the mode, the title, the
//! subtitle, the camera note) and the badge and the Back button:
//!
//! ```ignore
//! Object::prefab(crate::assets::prefabs::screen_prefab::prefab).child(
//!     Object::prefab(crate::assets::prefabs::title_card_prefab::prefab)
//!         .child(Object::new("Mode").with(Text::new("COOP / OBJECTIVE")) /* ... */)
//!         .child(Object::prefab(crate::assets::prefabs::coming_soon_prefab::prefab))
//!         .child(Object::prefab(crate::assets::prefabs::card_back_button_prefab::prefab)),
//! )
//! ```
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const CARD: Color = Color::srgba(0.067, 0.086, 0.114, 0.82);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Card")
            .with(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(10),
                padding: UiRect::axes(px(40), px(28)),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            })
            .with(BackgroundColor(CARD)),
    }
}
