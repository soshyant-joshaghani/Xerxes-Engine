//! The **banner**: a dim sheet over the whole screen, hidden until the game shows it (paused,
//! finished, crashed). The scene puts its own marker on the instance and adds the text as a
//! child, with a marker of its own:
//!
//! ```ignore
//! Object::prefab(crate::assets::prefabs::banner_prefab::prefab)
//!     .with(ResultBanner)
//!     .child(Object::new("Banner Text").with(ResultText) /* ... */)
//! ```
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Banner")
            .with(RectTransform::stretch(0.0, 0.0, 0.0, 0.0))
            .with(Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)))
            .with(Visibility::Hidden),
    }
}
