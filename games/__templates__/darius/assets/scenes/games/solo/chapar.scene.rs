//! Chapar (Solo / Score): the royal courier on the endless relay road. GDD:
//! `__docs__/gdd/darius/solo-score-chapar.md`.
//!
//! The scene has three objects at its root, like Ahriman's Horde:
//!
//! - `Camera`: the game's camera (the layout module boxes it into the gap between the bars).
//! - `Game`: a world-space canvas, the layer the level lives on: sky, far mesas, the road with
//!   its dashes, the rider, and `Road`, the object obstacles are spawned under as the run needs
//!   them. The rider stays at one place; the road, dashes and mesas scroll past it (`Parallax`).
//! - `HUD`: a screen-space overlay canvas: the header bar (distance, buttons), the banner, and
//!   the footer with the two big touch buttons, Duck (left half) and Jump (right half).
//!
//! The game's logic (`src/chapar`) only drives these objects through marker components.
//!
//! Scenes are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

use crate::chapar::{
    Banner, BannerText, ChaparAction, DayText, DistanceText, GhostRider, HintText, HudButton, Parallax, PauseLabel,
    Rider, Road,
};
use crate::layout::{
    Band, BandContent, BandToggle, BandToggleLabel, FooterBand, HeaderBand,
};

const SKY: Color = Color::srgb(0.96, 0.78, 0.52);
const SUN: Color = Color::srgb(0.99, 0.91, 0.66);
const MESA: Color = Color::srgb(0.73, 0.50, 0.38);
const SAND: Color = Color::srgb(0.80, 0.62, 0.40);
const EDGE: Color = Color::srgb(0.58, 0.42, 0.27);
const DASH: Color = Color::srgb(0.90, 0.76, 0.55);
const HORSE: Color = Color::srgb(0.30, 0.18, 0.12);
const COURIER: Color = Color::srgb(0.16, 0.30, 0.62);
const SASH: Color = Color::srgb(0.95, 0.80, 0.30);
const GHOST_BODY: Color = Color::srgba(0.95, 0.97, 1.0, 0.45);
// Opaque: the header and footer bars hide the world behind them.
const PANEL: Color = Color::srgb(0.047, 0.055, 0.071);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const SCORE: Color = Color::srgb(0.984, 0.827, 0.302);
const PRIMARY: Color = Color::srgb(0.22, 0.741, 0.973);
const PRIMARY_TEXT: Color = Color::srgb(0.043, 0.071, 0.125);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);
const JUMP: Color = Color::srgb(0.878, 0.659, 0.243);
const DUCK: Color = Color::srgb(0.22, 0.26, 0.33);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Camera")
                .with(GameCamera2D)
                .with(Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::AutoMin {
                        min_width: 760.0,
                        min_height: 430.0,
                    },
                    ..OrthographicProjection::default_2d()
                })),
            Object::new("Game")
                .with(Canvas::world(1280, 540, 1.0))
                .child(
                    Object::new("Sky")
                        .with(SpriteDef {
                            image: None,
                            color: SKY,
                            size: Some(Vec2::new(6000.0, 1600.0)),
                        })
                        .with(Transform::from_xyz(0.0, 0.0, -10.0)),
                )
                .child(
                    Object::new("Sun")
                        .with(SpriteDef {
                            image: None,
                            color: SUN,
                            size: Some(Vec2::new(90.0, 90.0)),
                        })
                        .with(Transform::from_xyz(250.0, 130.0, -9.0)),
                )
                .child(
                    Object::new("Mesa 1")
                        .with(Parallax {
                            factor: 0.12,
                            offset: 0.0,
                            span: 2400.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: MESA,
                            size: Some(Vec2::new(170.0, 70.0)),
                        })
                        .with(Transform::from_xyz(0.0, -75.0, -5.0)),
                )
                .child(
                    Object::new("Mesa 2")
                        .with(Parallax {
                            factor: 0.12,
                            offset: 400.0,
                            span: 2400.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: MESA,
                            size: Some(Vec2::new(230.0, 110.0)),
                        })
                        .with(Transform::from_xyz(0.0, -55.0, -5.0)),
                )
                .child(
                    Object::new("Mesa 3")
                        .with(Parallax {
                            factor: 0.12,
                            offset: 800.0,
                            span: 2400.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: MESA,
                            size: Some(Vec2::new(140.0, 55.0)),
                        })
                        .with(Transform::from_xyz(0.0, -82.5, -5.0)),
                )
                .child(
                    Object::new("Mesa 4")
                        .with(Parallax {
                            factor: 0.12,
                            offset: 1200.0,
                            span: 2400.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: MESA,
                            size: Some(Vec2::new(260.0, 125.0)),
                        })
                        .with(Transform::from_xyz(0.0, -47.5, -5.0)),
                )
                .child(
                    Object::new("Mesa 5")
                        .with(Parallax {
                            factor: 0.12,
                            offset: 1600.0,
                            span: 2400.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: MESA,
                            size: Some(Vec2::new(190.0, 80.0)),
                        })
                        .with(Transform::from_xyz(0.0, -70.0, -5.0)),
                )
                .child(
                    Object::new("Mesa 6")
                        .with(Parallax {
                            factor: 0.12,
                            offset: 2000.0,
                            span: 2400.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: MESA,
                            size: Some(Vec2::new(210.0, 95.0)),
                        })
                        .with(Transform::from_xyz(0.0, -62.5, -5.0)),
                )
                .child(
                    Object::new("Road Surface")
                        .with(SpriteDef {
                            image: None,
                            color: SAND,
                            size: Some(Vec2::new(6000.0, 300.0)),
                        })
                        .with(Transform::from_xyz(0.0, -260.0, 0.0)),
                )
                .child(
                    Object::new("Road Edge")
                        .with(SpriteDef {
                            image: None,
                            color: EDGE,
                            size: Some(Vec2::new(6000.0, 8.0)),
                        })
                        .with(Transform::from_xyz(0.0, -114.0, 0.1)),
                )
                .child(
                    Object::new("Dash 1")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 0.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 2")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 160.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 3")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 320.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 4")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 480.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 5")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 640.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 6")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 800.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 7")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 960.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 8")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 1120.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 9")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 1280.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 10")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 1440.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 11")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 1600.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 12")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 1760.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 13")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 1920.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                .child(
                    Object::new("Dash 14")
                        .with(Parallax {
                            factor: 1.00,
                            offset: 2080.0,
                            span: 2240.0,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: DASH,
                            size: Some(Vec2::new(70.0, 6.0)),
                        })
                        .with(Transform::from_xyz(0.0, -146.0, 0.2)),
                )
                // Obstacles are spawned under this object as the run lays the road.
                .child(
                    Object::new("Road")
                        .with(Road)
                        .with(Transform::from_xyz(0.0, 0.0, 1.0)),
                )
                // The ghost of the day's best run: the rider again, faint. Shown by `place_ghost`
                // while there is a recording.
                .child(
                    Object::new("Ghost")
                        .with(GhostRider)
                        .with(Visibility::Hidden)
                        .with(Transform::from_xyz(-300.0, -110.0, 1.9))
                        .child(
                            Object::new("Ghost Horse")
                                .with(SpriteDef {
                                    image: None,
                                    color: GHOST_BODY,
                                    size: Some(Vec2::new(56.0, 28.0)),
                                })
                                .with(Transform::from_xyz(0.0, 14.0, 0.0)),
                        )
                        .child(
                            Object::new("Ghost Courier")
                                .with(SpriteDef {
                                    image: None,
                                    color: GHOST_BODY,
                                    size: Some(Vec2::new(16.0, 36.0)),
                                })
                                .with(Transform::from_xyz(4.0, 46.0, 0.1)),
                        ),
                )
                // The rider: horse and courier as one body. Its feet are the object's origin; it
                // is squashed from there while ducking.
                .child(
                    Object::new("Rider")
                        .with(Rider)
                        .with(Transform::from_xyz(-300.0, -110.0, 2.0))
                        .child(
                            Object::new("Horse")
                                .with(SpriteDef {
                                    image: None,
                                    color: HORSE,
                                    size: Some(Vec2::new(56.0, 28.0)),
                                })
                                .with(Transform::from_xyz(0.0, 14.0, 0.0)),
                        )
                        .child(
                            Object::new("Courier")
                                .with(SpriteDef {
                                    image: None,
                                    color: COURIER,
                                    size: Some(Vec2::new(16.0, 36.0)),
                                })
                                .with(Transform::from_xyz(4.0, 46.0, 0.1)),
                        )
                        .child(
                            Object::new("Sash")
                                .with(SpriteDef {
                                    image: None,
                                    color: SASH,
                                    size: Some(Vec2::new(16.0, 6.0)),
                                })
                                .with(Transform::from_xyz(4.0, 50.0, 0.2)),
                        ),
                ),
            Object::new("HUD")
                .with(Canvas::overlay())
                // Header: the top row. Title, hint and distance on the left; Pause / Restart /
                // Back on the right; the collapse toggle in the corner.
                .child(
                    Object::new("Header")
                        .with(HeaderBand)
                        .with(Node::default())
                        .with(ZIndex(1))
                        .with(RectTransform {
                            anchor_min: Vec2::new(0.0, 0.8),
                            anchor_max: Vec2::new(1.0, 1.0),
                            pivot: Vec2::new(0.5, 0.5),
                            position: Vec2::new(0.0, 0.0),
                            size: Vec2::new(0.0, 0.0),
                        })
                        .with(BackgroundColor(PANEL))
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
                                        .with(Text::new("Chapar"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Day")
                                        .with(DayText)
                                        .with(Text::new("Daily road"))
                                        .with(TextFont::from_font_size(12.0))
                                        .with(TextColor(SCORE)),
                                )
                                .child(
                                    Object::new("Hint")
                                        .with(HintText)
                                        .with(Text::new("Space / Up: jump (hold = higher)  |  S / Down: duck"))
                                        .with(TextFont::from_font_size(13.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Distance")
                                        .with(DistanceText)
                                        .with(Text::new("0 m   best 0"))
                                        .with(TextFont::from_font_size(18.0))
                                        .with(TextColor(SCORE)),
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
                                    column_gap: px(8),
                                    ..default()
                                })
                                .child(
                                    Object::new("Pause Button")
                                        .with(Button)
                                        .with(HudButton(ChaparAction::TogglePause))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(PRIMARY))
                                        .child(
                                            Object::new("Pause Button Label")
                                                .with(PauseLabel)
                                                .with(Text::new("Pause"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(PRIMARY_TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Restart Button")
                                        .with(Button)
                                        .with(HudButton(ChaparAction::Restart))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Restart Button Label")
                                                .with(Text::new("Restart"))
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
                )
                // The banner covers the whole screen while paused, counting down or finished.
                .child(
                    Object::new("Banner")
                        .with(Banner)
                        .with(RectTransform::stretch(0.0, 0.0, 0.0, 0.0))
                        .with(Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)))
                        .with(Visibility::Hidden)
                        .child(
                            Object::new("Banner Text")
                                .with(BannerText)
                                .with(Text::new("GET READY"))
                                .with(TextFont::from_font_size(28.0))
                                .with(TextColor(Color::WHITE)),
                        ),
                )
                // Footer: two big touch buttons, Duck on the left half and Jump on the right
                // (short of the corner toggle). They also answer the mouse.
                .child(
                    Object::new("Footer")
                        .with(FooterBand)
                        .with(Node::default())
                        .with(ZIndex(1))
                        .with(RectTransform {
                            anchor_min: Vec2::new(0.0, 0.0),
                            anchor_max: Vec2::new(1.0, 0.22),
                            pivot: Vec2::new(0.5, 0.5),
                            position: Vec2::new(0.0, 0.0),
                            size: Vec2::new(0.0, 0.0),
                        })
                        .with(BackgroundColor(PANEL))
                        .child(
                            Object::new("Duck Button")
                                .with(BandContent(Band::Footer))
                                .with(Button)
                                .with(TouchButton("duck"))
                                .with(RectTransform {
                                    anchor_min: Vec2::new(0.02, 0.1),
                                    anchor_max: Vec2::new(0.48, 0.9),
                                    pivot: Vec2::new(0.5, 0.5),
                                    position: Vec2::new(0.0, 0.0),
                                    size: Vec2::new(0.0, 0.0),
                                })
                                .with(Node {
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(px(12)),
                                    ..default()
                                })
                                .with(BackgroundColor(DUCK))
                                .child(
                                    Object::new("Duck Label")
                                        .with(Text::new("DUCK"))
                                        .with(TextFont::from_font_size(20.0))
                                        .with(TextColor(TEXT)),
                                ),
                        )
                        .child(
                            Object::new("Jump Button")
                                .with(BandContent(Band::Footer))
                                .with(Button)
                                .with(TouchButton("jump"))
                                .with(RectTransform {
                                    anchor_min: Vec2::new(0.5, 0.1),
                                    anchor_max: Vec2::new(0.9, 0.9),
                                    pivot: Vec2::new(0.5, 0.5),
                                    position: Vec2::new(0.0, 0.0),
                                    size: Vec2::new(0.0, 0.0),
                                })
                                .with(Node {
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(px(12)),
                                    ..default()
                                })
                                .with(BackgroundColor(JUMP))
                                .child(
                                    Object::new("Jump Label")
                                        .with(Text::new("JUMP"))
                                        .with(TextFont::from_font_size(20.0))
                                        .with(TextColor(PRIMARY_TEXT)),
                                ),
                        )
                        .child(
                            Object::new("Footer Toggle")
                                .with(Button)
                                .with(BandToggle(Band::Footer))
                                .with(RectTransform {
                                    anchor_min: Vec2::new(1.0, 1.0),
                                    anchor_max: Vec2::new(1.0, 1.0),
                                    pivot: Vec2::new(0.0, 1.0),
                                    position: Vec2::new(-46.0, -6.0),
                                    size: Vec2::new(36.0, 28.0),
                                })
                                .with(Node {
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(px(6)),
                                    ..default()
                                })
                                .with(BackgroundColor(GHOST))
                                .child(
                                    Object::new("Footer Toggle Label")
                                        .with(BandToggleLabel(Band::Footer))
                                        .with(Text::new("-"))
                                        .with(TextFont::from_font_size(16.0))
                                        .with(TextColor(TEXT)),
                                ),
                        ),
                ),
        ],
    }
}
