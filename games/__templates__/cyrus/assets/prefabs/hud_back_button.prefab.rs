//! The **Back to Menu** button of a band: the engine's `ExitToMenuButton` in the HUD button's
//! frame, label and all. Every game screen has exactly this one, so it is whole here.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const TEXT: Color = Color::srgb(0.914, 0.902, 0.863);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Back Button")
            .with(Button)
            .with(ExitToMenuButton)
            .with(Node {
                padding: UiRect::axes(px(10), px(6)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            })
            .with(BackgroundColor(GHOST))
            .child(
                Object::new("Back Label")
                    .with(Text::new("Back to Menu"))
                    .with(TextFont::from_font_size(13.0))
                    .with(TextColor(TEXT)),
            ),
    }
}
