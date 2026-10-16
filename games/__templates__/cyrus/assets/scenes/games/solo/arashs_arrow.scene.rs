//! Arash's Arrow (Solo / Objective). Coming soon as a timed speedrun (R8, see GDD). For now
//! its slice 1, a 3D diorama of Qanat's first room: tap soil to lay tiles, drag to orbit, Open the
//! sluice. GDD: `__docs__/gdd/cyrus/solo-objective-arashs-arrow.md`.

use xerxes_engine::prelude::*;

use crate::arashs_arrow::{
    GateAction, GateBoard, GateButton, GateCamera, GateCursor, ResultBanner, ResultText, StatusText,
};

const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const DIM: Color = Color::srgb(0.62, 0.65, 0.70);
const STATUS: Color = Color::srgb(0.984, 0.827, 0.302);
const PRIMARY: Color = Color::srgb(0.22, 0.741, 0.973);
const PRIMARY_TEXT: Color = Color::srgb(0.043, 0.071, 0.125);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);
const PANEL: Color = Color::srgba(0.047, 0.055, 0.071, 0.92);
const BANNER: Color = Color::srgba(0.067, 0.086, 0.114, 0.92);
const CURSOR: Color = Color::srgb(0.98, 0.82, 0.30);
const GROUND: Color = Color::srgb(0.45, 0.40, 0.30);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D3,
        objects: vec![
            Object::new("Camera")
                .with(GameCamera3D)
                .with(GateCamera)
                .with(
                    Transform::from_xyz(0.0, 11.0, 14.0)
                        .looking_at(Vec3::new(0.0, 0.4, 0.0), Vec3::Y),
                ),
            Object::new("Sun")
                .with(DirectionalLight {
                    illuminance: 12_000.0,
                    shadows_enabled: true,
                    ..default()
                })
                .with(Transform::from_xyz(6.0, 14.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y)),
            Object::new("Board")
                .with(GateBoard)
                .child(
                    Object::new("Base")
                        .with(MeshDef::Plane { size: 16.0 })
                        .with(MaterialDef {
                            color: GROUND,
                            texture: None,
                        })
                        .with(Transform::from_xyz(0.0, -0.02, 0.0)),
                )
                .child(
                    Object::new("Cursor")
                        .with(GateCursor)
                        .with(MeshDef::Cube { size: 1.0 })
                        .with(MaterialDef {
                            color: CURSOR,
                            texture: None,
                        })
                        .with(
                            Transform::from_xyz(0.0, 0.8, 0.0)
                                .with_scale(Vec3::new(1.05, 0.08, 1.05)),
                        ),
                ),
            Object::new("HUD")
                .with(Canvas::overlay())
                .with(Node {
                    width: percent(100),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(px(12)),
                    ..default()
                })
                .child(
                    Object::new("Top")
                        .with(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            column_gap: px(12),
                            padding: UiRect::axes(px(14), px(10)),
                            border_radius: BorderRadius::all(px(8)),
                            ..default()
                        })
                        .with(BackgroundColor(PANEL))
                        .child(
                            Object::new("Info")
                                .with(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(4),
                                    ..default()
                                })
                                .child(
                                    Object::new("Title")
                                        .with(Text::new("Arash's Arrow"))
                                        .with(TextFont::from_font_size(18.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Hint")
                                        .with(Text::new(
                                            "Tap soil to lay tiles  |  drag to orbit  |  Open sluice",
                                        ))
                                        .with(TextFont::from_font_size(11.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Status")
                                        .with(StatusText)
                                        .with(Text::new("Tiles 0  |  par 12"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(STATUS)),
                                ),
                        )
                        .child(
                            Object::new("Buttons")
                                .with(Node {
                                    align_items: AlignItems::Center,
                                    column_gap: px(8),
                                    flex_wrap: FlexWrap::Wrap,
                                    ..default()
                                })
                                .child(
                                    Object::new("Open")
                                        .with(Button)
                                        .with(GateButton(GateAction::Open))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(PRIMARY))
                                        .child(
                                            Object::new("Open Label")
                                                .with(Text::new("Open"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(PRIMARY_TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Undo")
                                        .with(Button)
                                        .with(GateButton(GateAction::Undo))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Undo Label")
                                                .with(Text::new("Undo"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Clear")
                                        .with(Button)
                                        .with(GateButton(GateAction::Clear))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Clear Label")
                                                .with(Text::new("Clear"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Restart")
                                        .with(Button)
                                        .with(GateButton(GateAction::Restart))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Restart Label")
                                                .with(Text::new("Restart"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Back")
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
                                                .with(Text::new("Menu"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                ),
                        ),
                )
                .child(
                    Object::new("Result Banner")
                        .with(ResultBanner)
                        .with(Visibility::Hidden)
                        .with(Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            right: px(0),
                            top: percent(40),
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .child(
                            Object::new("Result Card")
                                .with(Node {
                                    padding: UiRect::axes(px(24), px(16)),
                                    border_radius: BorderRadius::all(px(10)),
                                    ..default()
                                })
                                .with(BackgroundColor(BANNER))
                                .child(
                                    Object::new("Result Text")
                                        .with(ResultText)
                                        .with(Text::new(""))
                                        .with(TextFont::from_font_size(18.0))
                                        .with(TextColor(STATUS)),
                                ),
                        ),
                ),
        ],
    }
}
