//! The wide **Back to menu** button at the foot of a title card (the HUD's own small one is
//! `hud_back_button`). Whole here: every title card has exactly this button.
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
                width: px(200),
                padding: UiRect::axes(px(16), px(10)),
                margin: UiRect::top(px(14)),
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(8)),
                ..default()
            })
            .with(BackgroundColor(GHOST))
            .child(
                Object::new("Back Label")
                    .with(Text::new("Back to menu"))
                    .with(TextFont::from_font_size(15.0))
                    .with(TextColor(TEXT)),
            ),
    }
}
