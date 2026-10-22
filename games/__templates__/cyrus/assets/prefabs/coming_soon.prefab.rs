//! The **COMING SOON** badge of a game that is not built yet: the outlined pill on its title
//! card. Whole here, because every one of them is the same.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const AMBER: Color = Color::srgb(0.878, 0.659, 0.243);
const AMBER_DIM: Color = Color::srgb(0.541, 0.416, 0.180);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Coming Soon")
            .with(Node {
                padding: UiRect::axes(px(12), px(5)),
                margin: UiRect::top(px(8)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            })
            .with(BorderColor::all(AMBER_DIM))
            .child(
                Object::new("Coming Soon Label")
                    .with(Text::new("COMING SOON"))
                    .with(TextFont::from_font_size(11.0))
                    .with(TextColor(AMBER)),
            ),
    }
}
