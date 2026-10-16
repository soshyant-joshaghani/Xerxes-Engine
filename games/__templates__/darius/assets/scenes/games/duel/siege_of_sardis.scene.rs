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
const AMBER_DIM: Color = Color::srgb(0.541, 0.416, 0.180);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);
const BACKDROP: Color = Color::srgb(0.067, 0.086, 0.114);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Camera").with(GameCamera2D),
            Object::new("HUD")
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
                .with(BackgroundColor(BACKDROP))
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
                .child(
                    Object::new("Coming Soon")
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
                )
                .child(
                    Object::new("Back Button")
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
                ),
        ],
    }
}
