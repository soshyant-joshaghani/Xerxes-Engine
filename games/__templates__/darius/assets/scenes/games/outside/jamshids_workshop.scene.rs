//! Jamshid's Workshop (Sandbox): side-view physics toy table. Drop crates and balls, drag
//! them, tilt gravity. GDD: `__docs__/gdd/darius/sandbox-jamshids-workshop.md`.
//!
//! Root objects: `Camera`, `Game` (world-space workbench with floor, walls, starter toys) and
//! `HUD` (header with gravity slider, spawn buttons, reset, back). Plain data for the editor.

use xerxes_engine::modules::physics::{Body2d, Shape, rect};
use xerxes_engine::prelude::*;

use crate::jamshids_workshop::{
    Ball, GravityLabel, GravitySlider, LabCamera, SpawnKind, Toy, WorkshopAction, WorkshopButton,
    Workbench,
};
use crate::layout::{Band, BandContent, BandToggle, BandToggleLabel, HeaderBand};

const WALL: Color = Color::srgb(0.32, 0.36, 0.45);
const BENCH: Color = Color::srgb(0.18, 0.14, 0.11);
const BACK: Color = Color::srgb(0.12, 0.14, 0.18);
const CRATE: Color = Color::srgb(0.72, 0.48, 0.28);
const BALL: Color = Color::srgb(0.30, 0.66, 0.92);
const PANEL: Color = Color::srgb(0.047, 0.055, 0.071);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const DIM: Color = Color::srgb(0.62, 0.65, 0.70);
const STATUS: Color = Color::srgb(0.984, 0.827, 0.302);
const PRIMARY: Color = Color::srgb(0.22, 0.741, 0.973);
const PRIMARY_TEXT: Color = Color::srgb(0.043, 0.071, 0.125);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Camera")
                .with(GameCamera2D)
                .with(LabCamera)
                .with(Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::AutoMin {
                        min_width: 880.0,
                        min_height: 520.0,
                    },
                    ..OrthographicProjection::default_2d()
                })),
            Object::new("Game")
                .with(Workbench)
                .with(Canvas::world(800, 500, 1.0))
                .child(Object::new("Backdrop").with(SpriteDef {
                    image: None,
                    color: BACK,
                    size: Some(Vec2::new(800.0, 500.0)),
                }))
                .child(
                    Object::new("Floor")
                        .with(SpriteDef {
                            image: None,
                            color: BENCH,
                            size: Some(Vec2::new(760.0, 28.0)),
                        })
                        .with(Transform::from_xyz(0.0, -200.0, 0.4))
                        .with(Body2d::fixed(rect(Vec2::new(760.0, 28.0)))),
                )
                .child(
                    Object::new("Wall West")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(20.0, 420.0)),
                        })
                        .with(Transform::from_xyz(-390.0, 0.0, 0.4))
                        .with(Body2d::fixed_fit()),
                )
                .child(
                    Object::new("Wall East")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(20.0, 420.0)),
                        })
                        .with(Transform::from_xyz(390.0, 0.0, 0.4))
                        .with(Body2d::fixed_fit()),
                )
                .child(
                    Object::new("Shelf")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(200.0, 16.0)),
                        })
                        .with(Transform::from_xyz(-180.0, 40.0, 0.4))
                        .with(Body2d::fixed_fit()),
                )
                .child(
                    Object::new("Starter Crate")
                        .with(Toy)
                        .with(SpriteDef {
                            image: None,
                            color: CRATE,
                            size: Some(Vec2::new(40.0, 40.0)),
                        })
                        .with(Transform::from_xyz(-120.0, -160.0, 0.5))
                        .with(Body2d::dynamic(rect(Vec2::new(40.0, 40.0)))),
                )
                .child(
                    Object::new("Starter Ball")
                        .with(Toy)
                        .with(Ball { radius: 18.0 })
                        // Sprite is a stand-in for the editor; at play time `dress_balls`
                        // replaces it with a Mesh2d circle.
                        .with(SpriteDef {
                            image: None,
                            color: BALL,
                            size: Some(Vec2::new(36.0, 36.0)),
                        })
                        .with(Transform::from_xyz(80.0, -140.0, 0.5))
                        .with(Body2d::dynamic(Shape::Circle { radius: 18.0 })),
                ),
            Object::new("HUD")
                .with(Canvas::overlay())
                .with(Node {
                    width: percent(100),
                    height: percent(100),
                    ..default()
                })
                .child(
                    Object::new("Header")
                        .with(HeaderBand)
                        .with(RectTransform {
                            anchor_min: Vec2::new(0.0, 1.0),
                            anchor_max: Vec2::new(1.0, 1.0),
                            pivot: Vec2::new(0.5, 1.0),
                            position: Vec2::ZERO,
                            size: Vec2::new(0.0, 0.0),
                        })
                        .with(Node {
                            width: percent(100),
                            height: percent(20),
                            ..default()
                        })
                        .with(BackgroundColor(PANEL))
                        .with(ZIndex(1))
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
                                    Object::new("Title")
                                        .with(Text::new("Jamshid's Workshop"))
                                        .with(TextFont::from_font_size(18.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Hint")
                                        .with(Text::new(
                                            "Drag toys  |  Spawn crates and balls  |  Slide gravity",
                                        ))
                                        .with(TextFont::from_font_size(11.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Gravity Row")
                                        .with(Node {
                                            align_items: AlignItems::Center,
                                            column_gap: px(10),
                                            ..default()
                                        })
                                        .child(
                                            Object::new("Gravity Label")
                                                .with(GravityLabel)
                                                .with(Text::new("Gravity 0.35x"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(STATUS)),
                                        )
                                        .child(
                                            Object::new("Gravity Slider")
                                                .with(GravitySlider)
                                                .with(Slider::new(0.0, 1.5, 0.35))
                                                .with(Node {
                                                    width: px(180),
                                                    height: px(14),
                                                    ..default()
                                                }),
                                        ),
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
                                    Object::new("Crate Button")
                                        .with(Button)
                                        .with(WorkshopButton(WorkshopAction::Spawn(SpawnKind::Crate)))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(PRIMARY))
                                        .child(
                                            Object::new("Crate Label")
                                                .with(Text::new("Crate"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(PRIMARY_TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Ball Button")
                                        .with(Button)
                                        .with(WorkshopButton(WorkshopAction::Spawn(SpawnKind::Ball)))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(PRIMARY))
                                        .child(
                                            Object::new("Ball Label")
                                                .with(Text::new("Ball"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(PRIMARY_TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Reset Button")
                                        .with(Button)
                                        .with(WorkshopButton(WorkshopAction::Reset))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Reset Label")
                                                .with(Text::new("Reset"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Back Button")
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
                                ),
                        )
                        .child(
                            Object::new("Header Toggle")
                                .with(Button)
                                .with(BandToggle(Band::Header))
                                .with(RectTransform::anchored(
                                    Anchor::TopRight,
                                    Vec2::new(-10.0, -8.0),
                                    Vec2::new(36.0, 28.0),
                                ))
                                .with(Node {
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(px(6)),
                                    ..default()
                                })
                                .with(BackgroundColor(GHOST))
                                .child(
                                    Object::new("Header Toggle Label")
                                        .with(BandToggleLabel(Band::Header))
                                        .with(Text::new("-"))
                                        .with(TextFont::from_font_size(16.0))
                                        .with(TextColor(TEXT)),
                                ),
                        ),
                ),
        ],
    }
}
