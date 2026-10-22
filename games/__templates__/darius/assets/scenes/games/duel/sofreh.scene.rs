//! Sofreh (Duel / Score): two cooks race to serve kebabs from one shared kitchen. Slice 1 is the
//! playable local game: one recipe, two hatches, a shared grill and a four-minute clock.
//!
//! Scenes are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

use crate::sofreh::{
    SofrehBanner, SofrehBoard, SofrehCook, SofrehGrill, SofrehResult, SofrehStatus,
};

const TABLE: Color = Color::srgb(0.20, 0.16, 0.13);
const FLOOR: Color = Color::srgb(0.31, 0.25, 0.20);
const GRID: Color = Color::srgb(0.43, 0.34, 0.27);
const WALL: Color = Color::srgb(0.16, 0.14, 0.13);
const PANTRY: Color = Color::srgb(0.53, 0.34, 0.20);
const GRILL: Color = Color::srgb(0.75, 0.28, 0.15);
const PLATES: Color = Color::srgb(0.79, 0.77, 0.69);
const TEAM_ONE: Color = Color::srgb(0.878, 0.659, 0.243);
const TEAM_TWO: Color = Color::srgb(0.30, 0.66, 0.92);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const DIM: Color = Color::srgb(0.62, 0.65, 0.70);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Camera")
                .with(GameCamera2D)
                .with(Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::AutoMin {
                        min_width: 720.0,
                        min_height: 480.0,
                    },
                    ..OrthographicProjection::default_2d()
                })),
            Object::new("Kitchen")
                .with(SofrehBoard)
                .with(Canvas::world(800, 500, 1.0))
                .child(Object::new("Table").with(SpriteDef {
                    image: None,
                    color: TABLE,
                    size: Some(Vec2::new(468.0, 276.0)),
                }))
                .child(Object::new("Floor").with(SpriteDef {
                    image: None,
                    color: FLOOR,
                    size: Some(Vec2::new(432.0, 240.0)),
                }))
                .child(
                    Object::new("Grid Vertical 1")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(2.0, 240.0)),
                        })
                        .with(Transform::from_xyz(-168.0, 0.0, 0.1)),
                )
                .child(
                    Object::new("Grid Vertical 2")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(2.0, 240.0)),
                        })
                        .with(Transform::from_xyz(-120.0, 0.0, 0.1)),
                )
                .child(
                    Object::new("Grid Vertical 3")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(2.0, 240.0)),
                        })
                        .with(Transform::from_xyz(-72.0, 0.0, 0.1)),
                )
                .child(
                    Object::new("Grid Vertical 4")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(2.0, 240.0)),
                        })
                        .with(Transform::from_xyz(-24.0, 0.0, 0.1)),
                )
                .child(
                    Object::new("Grid Vertical 5")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(2.0, 240.0)),
                        })
                        .with(Transform::from_xyz(24.0, 0.0, 0.1)),
                )
                .child(
                    Object::new("Grid Vertical 6")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(2.0, 240.0)),
                        })
                        .with(Transform::from_xyz(72.0, 0.0, 0.1)),
                )
                .child(
                    Object::new("Grid Vertical 7")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(2.0, 240.0)),
                        })
                        .with(Transform::from_xyz(120.0, 0.0, 0.1)),
                )
                .child(
                    Object::new("Grid Vertical 8")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(2.0, 240.0)),
                        })
                        .with(Transform::from_xyz(168.0, 0.0, 0.1)),
                )
                .child(
                    Object::new("Grid Horizontal 1")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(432.0, 2.0)),
                        })
                        .with(Transform::from_xyz(0.0, 72.0, 0.1)),
                )
                .child(
                    Object::new("Grid Horizontal 2")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(432.0, 2.0)),
                        })
                        .with(Transform::from_xyz(0.0, 24.0, 0.1)),
                )
                .child(
                    Object::new("Grid Horizontal 3")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(432.0, 2.0)),
                        })
                        .with(Transform::from_xyz(0.0, -24.0, 0.1)),
                )
                .child(
                    Object::new("Grid Horizontal 4")
                        .with(SpriteDef {
                            image: None,
                            color: GRID,
                            size: Some(Vec2::new(432.0, 2.0)),
                        })
                        .with(Transform::from_xyz(0.0, -72.0, 0.1)),
                )
                .child(
                    Object::new("Top Wall West")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(-192.0, 96.0, 0.2)),
                )
                .child(
                    Object::new("Top Wall West Inner")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(-144.0, 96.0, 0.2)),
                )
                .child(
                    Object::new("Top Wall Middle West")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(-48.0, 96.0, 0.2)),
                )
                .child(
                    Object::new("Top Wall Middle East")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(48.0, 96.0, 0.2)),
                )
                .child(
                    Object::new("Top Wall East Inner")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(144.0, 96.0, 0.2)),
                )
                .child(
                    Object::new("Top Wall East")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(192.0, 96.0, 0.2)),
                )
                .child(
                    Object::new("Bottom Wall West")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(-192.0, -96.0, 0.2)),
                )
                .child(
                    Object::new("Bottom Wall West Inner")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(-144.0, -96.0, 0.2)),
                )
                .child(
                    Object::new("Bottom Wall East Inner")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(144.0, -96.0, 0.2)),
                )
                .child(
                    Object::new("Bottom Wall East")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(192.0, -96.0, 0.2)),
                )
                .child(
                    Object::new("Pantry")
                        .with(SpriteDef {
                            image: None,
                            color: PANTRY,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(-96.0, 96.0, 0.3)),
                )
                .child(
                    Object::new("Grill")
                        .with(SpriteDef {
                            image: None,
                            color: GRILL,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(0.0, 96.0, 0.3)),
                )
                .child(
                    Object::new("Grill Progress")
                        .with(SofrehGrill)
                        .with(SpriteDef {
                            image: None,
                            color: Color::srgb(0.98, 0.72, 0.28),
                            size: Some(Vec2::new(32.0, 8.0)),
                        })
                        .with(Transform::from_xyz(0.0, 96.0, 1.0)),
                )
                .child(
                    Object::new("Plates")
                        .with(SpriteDef {
                            image: None,
                            color: PLATES,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(96.0, 96.0, 0.3)),
                )
                .child(
                    Object::new("Team 1 Hatch Top")
                        .with(SpriteDef {
                            image: None,
                            color: TEAM_ONE,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(-192.0, 48.0, 0.3)),
                )
                .child(
                    Object::new("Team 1 Hatch Middle")
                        .with(SpriteDef {
                            image: None,
                            color: TEAM_ONE,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(-192.0, 0.0, 0.3)),
                )
                .child(
                    Object::new("Team 1 Hatch Bottom")
                        .with(SpriteDef {
                            image: None,
                            color: TEAM_ONE,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(-192.0, -48.0, 0.3)),
                )
                .child(
                    Object::new("Team 2 Hatch Top")
                        .with(SpriteDef {
                            image: None,
                            color: TEAM_TWO,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(192.0, 48.0, 0.3)),
                )
                .child(
                    Object::new("Team 2 Hatch Middle")
                        .with(SpriteDef {
                            image: None,
                            color: TEAM_TWO,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(192.0, 0.0, 0.3)),
                )
                .child(
                    Object::new("Team 2 Hatch Bottom")
                        .with(SpriteDef {
                            image: None,
                            color: TEAM_TWO,
                            size: Some(Vec2::new(42.0, 42.0)),
                        })
                        .with(Transform::from_xyz(192.0, -48.0, 0.3)),
                )
                .child(
                    Object::new("Cook 1")
                        .with(SofrehCook(0))
                        .with(SpriteDef {
                            image: None,
                            color: TEAM_ONE,
                            size: Some(Vec2::new(28.0, 28.0)),
                        })
                        .with(Transform::from_xyz(-144.0, 48.0, 2.0)),
                )
                .child(
                    Object::new("Cook 2")
                        .with(SofrehCook(1))
                        .with(SpriteDef {
                            image: None,
                            color: TEAM_TWO,
                            size: Some(Vec2::new(28.0, 28.0)),
                        })
                        .with(Transform::from_xyz(144.0, 48.0, 2.0)),
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
                                    Vec2::ZERO,
                                ))
                                .with(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(4),
                                    ..default()
                                })
                                .child(
                                    Object::new("Title")
                                        .with(Text::new("Sofreh"))
                                        .with(TextFont::from_font_size(16.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Controls")
                                        .with(Text::new(
                                            "P1: WASD + E   |   P2: Arrows + Enter",
                                        ))
                                        .with(TextFont::from_font_size(12.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Status")
                                        .with(SofrehStatus)
                                        .with(Text::new(
                                            "Team 1: 0 dishes   Team 2: 0 dishes   240 s left",
                                        ))
                                        .with(TextFont::from_font_size(16.0))
                                        .with(TextColor(TEAM_ONE)),
                                ),
                        )
                        .child(Object::prefab(
                            crate::assets::prefabs::hud_back_button_prefab::prefab,
                        )),
                )
                .child(
                    Object::prefab(crate::assets::prefabs::banner_prefab::prefab)
                        .with(SofrehBanner)
                        .with(Visibility::Hidden)
                        .child(
                            Object::new("Result")
                                .with(SofrehResult)
                                .with(Text::new(""))
                                .with(TextFont::from_font_size(28.0))
                                .with(TextColor(Color::WHITE)),
                        ),
                ),
        ],
    }
}
