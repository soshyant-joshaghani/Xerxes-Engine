//! Siege of Sardis (Duel / Objective): battering-ram tug of war. Coming soon: for now an empty
//! 2D scene with its title card and a Back button. Design:
//! `__docs__/gdd/darius/duel-objective-siege-of-sardis.md`.
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
                        .with(Text::new("DUEL / OBJECTIVE"))
                        .with(TextFont::from_font_size(13.0))
                        .with(TextColor(AMBER)),
                )
                .child(
                    Object::new("Title")
                        .with(Text::new("Siege of Sardis"))
                        .with(TextFont::from_font_size(40.0))
                        .with(TextColor(TEXT)),
                )
                .child(
                    Object::new("Subtitle")
                        .with(Text::new("Battering-ram tug of war"))
                        .with(TextFont::from_font_size(16.0))
                        .with(TextColor(DIM)),
                )
                .child(
                    Object::new("Camera Note")
                        .with(Text::new("Side view, one long lane, scrolling"))
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
