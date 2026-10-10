//! The main menu, as a scene: a title and a Play button (`LevelButton` starts a level). It
//! grows into the Warp map's first place: family, how to play (offline, online, spectate), then
//! the mini-game.

use xerxes_engine::prelude::*;

const BACKDROP: Color = Color::srgb(0.043, 0.051, 0.071);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const ACCENT: Color = Color::srgb(0.95, 0.45, 0.12);

fn label(name: &str, value: &str, size: f32, color: Color) -> Object {
    Object::new(name)
        .with(Text::new(value))
        .with(TextFont::from_font_size(size))
        .with(TextColor(color))
}

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Camera").with(Camera2d),
            Object::new("Menu")
                .with(Node {
                    width: percent(100),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: px(14),
                    ..default()
                })
                .with(BackgroundColor(BACKDROP))
                .child(label("Title", "Darius", 34.0, TEXT))
                .child(
                    Object::new("Play Button")
                        .with(Button)
                        .with(LevelButton(0))
                        .with(Node {
                            width: px(220),
                            padding: UiRect::axes(px(16), px(10)),
                            justify_content: JustifyContent::Center,
                            border_radius: BorderRadius::all(px(8)),
                            ..default()
                        })
                        .with(BackgroundColor(ACCENT))
                        .child(label("Play Label", "Play", 16.0, BACKDROP)),
                ),
        ],
    }
}
