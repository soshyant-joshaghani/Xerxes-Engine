//! Tombak Duo (Coop / Score): call-and-response drumming. Coming soon: for now an empty
//! 2D scene with its title card and a Back button. Design:
//! `__docs__/gdd/darius/coop-score-tombak-duo.md`.
//!
//! Scenes are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const TEXT: Color = Color::srgb(0.914, 0.902, 0.863);
const DIM: Color = Color::srgb(0.596, 0.631, 0.675);
const FAINT: Color = Color::srgb(0.373, 0.416, 0.463);
const AMBER: Color = Color::srgb(0.878, 0.659, 0.243);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::prefab(crate::assets::prefabs::camera_prefab::prefab),
            Object::prefab(crate::assets::prefabs::title_card_prefab::prefab)
                .child(
                    Object::new("Mode")
                        .with(Text::new("COOP / SCORE"))
                        .with(TextFont::from_font_size(13.0))
                        .with(TextColor(AMBER)),
                )
                .child(
                    Object::new("Title")
                        .with(Text::new("Tombak Duo"))
                        .with(TextFont::from_font_size(40.0))
                        .with(TextColor(TEXT)),
                )
                .child(
                    Object::new("Subtitle")
                        .with(Text::new("Call-and-response drumming"))
                        .with(TextFont::from_font_size(16.0))
                        .with(TextColor(DIM)),
                )
                .child(
                    Object::new("Camera Note")
                        .with(Text::new("Two note lanes (UI), side by side"))
                        .with(TextFont::from_font_size(12.0))
                        .with(TextColor(FAINT)),
                )
                .child(Object::prefab(
                    crate::assets::prefabs::coming_soon_prefab::prefab,
                ))
                .child(Object::prefab(
                    crate::assets::prefabs::card_back_button_prefab::prefab,
                )),
        ],
    }
}
