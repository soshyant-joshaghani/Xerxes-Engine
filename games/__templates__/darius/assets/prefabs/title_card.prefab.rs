//! The **title card** of a game that is not built yet: the whole screen of an empty scene — the
//! HUD canvas, dark, with everything stacked down the middle. The scene adds the lines that are
//! its own (the mode, the title, the subtitle, the note) and the badge and the Back button:
//!
//! ```ignore
//! Object::prefab(crate::assets::prefabs::title_card_prefab::prefab)
//!     .child(Object::new("Mode").with(Text::new("COOP / ROLE")) /* ... */)
//!     .child(Object::prefab(crate::assets::prefabs::coming_soon_prefab::prefab))
//!     .child(Object::prefab(crate::assets::prefabs::card_back_button_prefab::prefab))
//! ```
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const BACKDROP: Color = Color::srgb(0.067, 0.086, 0.114);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("HUD")
            .with(Canvas::overlay())
            .with(Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(10),
                ..default()
            })
            .with(BackgroundColor(BACKDROP)),
    }
}
