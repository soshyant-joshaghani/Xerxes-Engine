//! Arzhang (N-gon / Objective): ink on walls and floors. Coming soon: for now an empty
//! 3D scene with its title card and a Back button. Design:
//! `__docs__/gdd/cyrus/ngon-objective-arzhang.md`.
//!
//! Scenes are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const TEXT: Color = Color::srgb(0.914, 0.902, 0.863);
const DIM: Color = Color::srgb(0.596, 0.631, 0.675);
const FAINT: Color = Color::srgb(0.373, 0.416, 0.463);
const AMBER: Color = Color::srgb(0.878, 0.659, 0.243);
const AMBER_DIM: Color = Color::srgb(0.541, 0.416, 0.180);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);
const CARD: Color = Color::srgba(0.067, 0.086, 0.114, 0.82);
const GROUND: Color = Color::srgb(0.62, 0.53, 0.38);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D3,
        objects: vec![
            Object::new("Camera")
                .with(GameCamera3D)
                .with(Transform::from_xyz(0.0, 9.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y)),
            Object::new("Sun")
                .with(DirectionalLight {
                    shadows_enabled: true,
                    ..default()
                })
                .with(Transform::from_xyz(4.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y)),
            Object::new("Ground")
                .with(MeshDef::Plane { size: 20.0 })
                .with(MaterialDef {
                    color: GROUND,
                    texture: None,
                }),
            Object::new("HUD")
                .with(Canvas::overlay())
                .with(Node {
                    width: percent(100),
                    height: percent(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .child(
                    Object::new("Card")
                        .with(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: px(10),
                            padding: UiRect::axes(px(40), px(28)),
                            border_radius: BorderRadius::all(px(12)),
                            ..default()
                        })
                        .with(BackgroundColor(CARD))
                        .child(
                            Object::new("Mode")
                                .with(Text::new("N-GON / OBJECTIVE"))
                                .with(TextFont::from_font_size(13.0))
                                .with(TextColor(AMBER)),
                        )
                        .child(
                            Object::new("Title")
                                .with(Text::new("Arzhang"))
                                .with(TextFont::from_font_size(40.0))
                                .with(TextColor(TEXT)),
                        )
                        .child(
                            Object::new("Subtitle")
                                .with(Text::new("Hold the provinces"))
                                .with(TextFont::from_font_size(16.0))
                                .with(TextColor(DIM)),
                        )
                        .child(
                            Object::new("Camera Note")
                                .with(Text::new("Third person"))
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
                ),
        ],
    }
}
