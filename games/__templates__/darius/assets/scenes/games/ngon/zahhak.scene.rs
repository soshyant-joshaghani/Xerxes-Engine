//! Zahhak (N-gon / Score): a Slither.io-style serpent arena. Slice 1: one round field, you and three
//! bots, the serpents and pellets drawn on it by the game, the length in the header, and the sheet
//! when the three minutes are up. Design: `__docs__/gdd/darius/ngon-score-zahhak.md`.
//!
//! The layout is the same as the other games': the field is a world canvas the game draws on, and
//! the header and content bands sit over it (the engine keeps the field in the gap between them).
//!
//! Scenes are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

use crate::zahhak::{
    ZahhakBanner, ZahhakField, ZahhakRestart, ZahhakResult, ZahhakStatus, ZahhakTrackPad,
};

const TEXT: Color = Color::srgb(0.914, 0.902, 0.863);
const DIM: Color = Color::srgb(0.596, 0.631, 0.675);
const AMBER: Color = Color::srgb(0.878, 0.659, 0.243);
const FIELD: Color = Color::srgb(0.07, 0.085, 0.10);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Camera")
                .with(GameCamera2D)
                .with(Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::AutoMin {
                        min_width: 520.0,
                        min_height: 560.0,
                    },
                    ..OrthographicProjection::default_2d()
                })),
            Object::new("Game")
                .with(Canvas::world(800, 500, 1.0))
                .child(
                    // The arena's floor. The wall and the serpents are drawn on it by the game
                    // while it plays (see `src/zahhak/play.rs`).
                    Object::new("Field")
                        .with(ZahhakField)
                        .with(SpriteDef {
                            image: None,
                            color: FIELD,
                            size: Some(Vec2::new(440.0, 440.0)),
                        })
                        .with(Transform::from_xyz(0.0, 0.0, -1.0)),
                ),
            Object::new("HUD")
                .with(Canvas::overlay())
                .child(
                    Object::prefab(crate::assets::prefabs::header_prefab::prefab)
                        .child(
                            Object::new("Info")
                                .with(BandContent(Band::Header))
                                .with(RectTransform::anchored(
                                    Anchor::MiddleLeft,
                                    Vec2::new(16.0, 0.0),
                                    Vec2::new(0.0, 0.0),
                                ))
                                .with(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(4),
                                    ..default()
                                })
                                .child(
                                    Object::new("Mode")
                                        .with(Text::new("N-GON / SCORE"))
                                        .with(TextFont::from_font_size(11.0))
                                        .with(TextColor(AMBER)),
                                )
                                .child(
                                    Object::new("Title")
                                        .with(Text::new("Zahhak"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Hint")
                                        .with(Text::new(
                                            "YOU = cyan/white  |  WASD/arrows point direction  |  drag to steer  |  Shift boosts after growing",
                                        ))
                                        .with(TextFont::from_font_size(11.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Status")
                                        .with(ZahhakStatus)
                                        .with(Text::new("Length 10  |  3 min"))
                                        .with(TextFont::from_font_size(16.0))
                                        .with(TextColor(AMBER)),
                                ),
                        )
                        .child(
                            Object::new("Buttons")
                                .with(BandContent(Band::Header))
                                .with(RectTransform::anchored(
                                    Anchor::MiddleRight,
                                    Vec2::new(-52.0, 0.0),
                                    Vec2::new(0.0, 0.0),
                                ))
                                .with(Node {
                                    align_items: AlignItems::Center,
                                    column_gap: px(8),
                                    ..default()
                                })
                                .child(
                                    Object::prefab(crate::assets::prefabs::hud_button_prefab::prefab)
                                        .named("Boost Button")
                                        .with(TouchButton("boost"))
                                        .child(
                                            Object::new("Boost Label")
                                                .with(Text::new("Boost"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(
                                    Object::prefab(crate::assets::prefabs::hud_button_prefab::prefab)
                                        .named("Restart Button")
                                        .with(ZahhakRestart)
                                        .child(
                                            Object::new("Restart Label")
                                                .with(Text::new("Restart"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(Object::prefab(
                                    crate::assets::prefabs::hud_back_button_prefab::prefab,
                                )),
                        ),
                )
                // The track pad: the whole content area steers. A drag from wherever the finger or the
                // mouse goes down turns the serpent that way; no ring is drawn, so nothing sits in a
                // corner. The header above it keeps its buttons.
                .child(
                    Object::prefab(crate::assets::prefabs::content_prefab::prefab).child(
                        Object::new("Track Pad")
                            .with(ZahhakTrackPad)
                            .with(TouchPad::new().with_mouse().source("track"))
                            .with(RectTransform::stretch(0.0, 0.0, 0.0, 0.0)),
                    ),
                )
                .child(
                    Object::prefab(crate::assets::prefabs::banner_prefab::prefab)
                        .with(ZahhakBanner)
                        .child(
                            Object::new("Result Text")
                                .with(ZahhakResult)
                                .with(Text::new(""))
                                .with(TextFont::from_font_size(28.0))
                                .with(TextColor(TEXT)),
                        ),
                ),
        ],
    }
}
