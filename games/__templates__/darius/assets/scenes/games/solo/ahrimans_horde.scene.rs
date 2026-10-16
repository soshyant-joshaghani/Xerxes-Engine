//! Ahriman's Horde (Solo · Survival): courtyard slice 1 — pistol, Fire / Switch / Reload-Dash pad,
//! one rusher; kill it and step into the warp. Darics are optional loot. GDD:
//! `__docs__/gdd/darius/solo-survival-ahrimans-horde.md`.
//!
//! The scene has three objects at its root, like a Unity scene with a camera and
//! two canvases:
//!
//! - `Camera`: the game's camera.
//! - `Game`: a world-space canvas, the layer the level lives on (floor, walls, the player and
//!   the darics). It is 800 x 500 pixels, one pixel to one world unit.
//! - `HUD`: a screen-space overlay canvas, drawn over everything. Its elements are anchored
//!   with `RectTransform` (the panel to the top-left corner, the banner stretched over all).
//!
//! Its size comes from the project's window settings (Project Settings), not from the canvas.
//! The game's logic (`src/ahrimans_horde`) only drives these objects through marker components.
//!
//! Scenes are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::modules::physics::{Body2d, Mover};
use xerxes_engine::prelude::*;

use crate::ahrimans_horde::{
    Action, Banner, BannerText, ControlSwitch, ControlSwitchLabel, Daric, Dpad, Enemy, GunPickup,
    HealthPickup, HintText, HudButton, PadLabel, PadSlot, PauseLabel, Player, ScoreText,
};
use crate::layout::{
    Band, BandContent, BandToggle, BandToggleLabel, FooterBand, HeaderBand,
};

const FLOOR: Color = Color::srgb(0.11, 0.13, 0.17);
const WALL: Color = Color::srgb(0.32, 0.36, 0.45);
const PLAYER: Color = Color::srgb(0.22, 0.74, 0.97);
const COIN: Color = Color::srgb(0.98, 0.82, 0.30);
const GUN: Color = Color::srgb(0.45, 0.78, 0.55);
const HEART: Color = Color::srgb(0.92, 0.35, 0.48);
const RUSHER: Color = Color::srgb(0.78, 0.28, 0.32);
// Opaque: the header and footer bars hide the world behind them.
const PANEL: Color = Color::srgb(0.047, 0.055, 0.071);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const SCORE: Color = Color::srgb(0.984, 0.827, 0.302);
const PRIMARY: Color = Color::srgb(0.22, 0.741, 0.973);
const PRIMARY_TEXT: Color = Color::srgb(0.043, 0.071, 0.125);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Camera")
                .with(GameCamera2D)
                .with(Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::AutoMin {
                        min_width: 880.0,
                        min_height: 580.0,
                    },
                    ..OrthographicProjection::default_2d()
                })),
            Object::new("Game")
                .with(Canvas::world(800, 500, 1.0))
                .child(Object::new("Floor").with(SpriteDef {
                    image: None,
                    color: FLOOR,
                    size: Some(Vec2::new(800.0, 500.0)),
                }))
                .child(
                    Object::new("Wall North")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(800.0, 20.0)),
                        })
                        .with(Transform::from_xyz(0.0, 240.0, 0.5))
                        .with(Body2d::fixed_fit()),
                )
                .child(
                    Object::new("Wall South")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(800.0, 20.0)),
                        })
                        .with(Transform::from_xyz(0.0, -240.0, 0.5))
                        .with(Body2d::fixed_fit()),
                )
                .child(
                    Object::new("Wall West")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(20.0, 500.0)),
                        })
                        .with(Transform::from_xyz(-390.0, 0.0, 0.5))
                        .with(Body2d::fixed_fit()),
                )
                .child(
                    Object::new("Wall East")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(20.0, 500.0)),
                        })
                        .with(Transform::from_xyz(390.0, 0.0, 0.5))
                        .with(Body2d::fixed_fit()),
                )
                .child(
                    Object::new("Pillar West")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(40.0, 220.0)),
                        })
                        .with(Transform::from_xyz(-150.0, 60.0, 0.5))
                        .with(Body2d::fixed_fit()),
                )
                .child(
                    Object::new("Pillar East")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(40.0, 220.0)),
                        })
                        .with(Transform::from_xyz(150.0, -60.0, 0.5))
                        .with(Body2d::fixed_fit()),
                )
                .child(
                    Object::new("Bar North")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(160.0, 30.0)),
                        })
                        .with(Transform::from_xyz(0.0, 170.0, 0.5))
                        .with(Body2d::fixed_fit()),
                )
                .child(
                    Object::new("Bar East")
                        .with(SpriteDef {
                            image: None,
                            color: WALL,
                            size: Some(Vec2::new(100.0, 30.0)),
                        })
                        .with(Transform::from_xyz(280.0, 150.0, 0.5))
                        .with(Body2d::fixed_fit()),
                )
                .child(
                    Object::new("Player")
                        .with(Player)
                        .with(SpriteDef {
                            image: None,
                            color: PLAYER,
                            size: Some(Vec2::new(28.0, 28.0)),
                        })
                        .with(Transform::from_xyz(-340.0, -190.0, 2.0))
                        .with(Body2d::kinematic_fit())
                        .with(Mover::default()),
                )
                // In the scene so Restart reloads it with the player (runtime spawns were left behind).
                .child(
                    Object::new("Rusher")
                        .with(Enemy::default())
                        .with(SpriteDef {
                            image: None,
                            color: RUSHER,
                            size: Some(Vec2::new(30.0, 30.0)),
                        })
                        .with(Transform::from_xyz(200.0, 40.0, 2.0))
                        .with(Body2d::kinematic_fit())
                        .with(Mover::default()),
                )
                .child(
                    Object::new("Gun Pickup")
                        .with(GunPickup)
                        .with(SpriteDef {
                            image: None,
                            color: GUN,
                            size: Some(Vec2::new(22.0, 14.0)),
                        })
                        .with(Transform::from_xyz(-80.0, -40.0, 1.0))
                        .with(Body2d::fixed_fit().sensor()),
                )
                .child(
                    Object::new("Health Pickup")
                        .with(HealthPickup)
                        .with(SpriteDef {
                            image: None,
                            color: HEART,
                            size: Some(Vec2::new(18.0, 18.0)),
                        })
                        .with(Transform::from_xyz(40.0, 120.0, 1.0))
                        .with(Body2d::fixed_fit().sensor()),
                )
                .child(
                    Object::new("Daric 1")
                        .with(Daric)
                        .with(SpriteDef {
                            image: None,
                            color: COIN,
                            size: Some(Vec2::new(14.0, 14.0)),
                        })
                        .with(Transform::from_xyz(-340.0, 190.0, 1.0))
                        .with(Body2d::fixed_fit().sensor()),
                )
                .child(
                    Object::new("Daric 2")
                        .with(Daric)
                        .with(SpriteDef {
                            image: None,
                            color: COIN,
                            size: Some(Vec2::new(14.0, 14.0)),
                        })
                        .with(Transform::from_xyz(-250.0, -150.0, 1.0))
                        .with(Body2d::fixed_fit().sensor()),
                )
                .child(
                    Object::new("Daric 3")
                        .with(Daric)
                        .with(SpriteDef {
                            image: None,
                            color: COIN,
                            size: Some(Vec2::new(14.0, 14.0)),
                        })
                        .with(Transform::from_xyz(-60.0, -180.0, 1.0))
                        .with(Body2d::fixed_fit().sensor()),
                )
                .child(
                    Object::new("Daric 4")
                        .with(Daric)
                        .with(SpriteDef {
                            image: None,
                            color: COIN,
                            size: Some(Vec2::new(14.0, 14.0)),
                        })
                        .with(Transform::from_xyz(0.0, 60.0, 1.0))
                        .with(Body2d::fixed_fit().sensor()),
                )
                .child(
                    Object::new("Daric 5")
                        .with(Daric)
                        .with(SpriteDef {
                            image: None,
                            color: COIN,
                            size: Some(Vec2::new(14.0, 14.0)),
                        })
                        .with(Transform::from_xyz(100.0, 215.0, 1.0))
                        .with(Body2d::fixed_fit().sensor()),
                )
                .child(
                    Object::new("Daric 6")
                        .with(Daric)
                        .with(SpriteDef {
                            image: None,
                            color: COIN,
                            size: Some(Vec2::new(14.0, 14.0)),
                        })
                        .with(Transform::from_xyz(240.0, -190.0, 1.0))
                        .with(Body2d::fixed_fit().sensor()),
                )
                .child(
                    Object::new("Daric 7")
                        .with(Daric)
                        .with(SpriteDef {
                            image: None,
                            color: COIN,
                            size: Some(Vec2::new(14.0, 14.0)),
                        })
                        .with(Transform::from_xyz(340.0, 190.0, 1.0))
                        .with(Body2d::fixed_fit().sensor()),
                )
                .child(
                    Object::new("Daric 8")
                        .with(Daric)
                        .with(SpriteDef {
                            image: None,
                            color: COIN,
                            size: Some(Vec2::new(14.0, 14.0)),
                        })
                        .with(Transform::from_xyz(300.0, 0.0, 1.0))
                        .with(Body2d::fixed_fit().sensor()),
                ),
            // One column of three rows: the header bar at the top, the game in the middle (the
            // camera is boxed into it by `crate::layout`), the controls in the footer. The two
            // bands' heights and the game's viewport are driven by `crate::layout` from the window
            // size and the collapse state; each band collapses to a thin strip with its toggle.
            // Every child inside a band is placed with an explicit `RectTransform` anchor + offset.
            Object::new("HUD")
                .with(Canvas::overlay())
                // Header: the top row. Title, hint and score on the left; Pause / Restart / Back on
                // the right; the collapse toggle in the corner.
                .child(
                    Object::new("Header")
                        .with(HeaderBand)
                        .with(Node::default())
                        // Above the banner, so the bar stays solid and clickable while the banner
                        // only dims the game in the middle.
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
                                        .with(Text::new("Ahriman's Horde"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Hint")
                                        .with(HintText)
                                        .with(Text::new(
                                            "Stick / WASD  ·  Space or RMB fire  ·  R reload/dash  ·  F switch  ·  Esc",
                                        ))
                                        .with(TextFont::from_font_size(13.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Score")
                                        .with(ScoreText)
                                        .with(Text::new(
                                            "Lives 3  ·  Depth 0  ·  Darics 0\n>Pistol 6/6∞",
                                        ))
                                        .with(TextFont::from_font_size(16.0))
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
                                        .with(HudButton(Action::TogglePause))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(PRIMARY))
                                        .child(
                                            Object::new("Pause Label")
                                                .with(PauseLabel)
                                                .with(Text::new("Pause"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(PRIMARY_TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Restart Button")
                                        .with(Button)
                                        .with(HudButton(Action::Restart))
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
                            Object::new("Control Switch")
                                .with(Button)
                                .with(ControlSwitch)
                                .with(BandContent(Band::Header))
                                // Header, left of the collapse toggle.
                                .with(RectTransform::anchored(
                                    Anchor::TopRight,
                                    Vec2::new(-120.0, -8.0),
                                    Vec2::new(102.0, 28.0),
                                ))
                                .with(Node {
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(px(6)),
                                    ..default()
                                })
                                .with(BackgroundColor(GHOST))
                                .child(
                                    Object::new("Control Switch Label")
                                        .with(ControlSwitchLabel)
                                        .with(Text::new("D-Pad"))
                                        .with(TextFont::from_font_size(13.0))
                                        .with(TextColor(TEXT)),
                                ),
                        )
                        .child(
                            Object::new("Header Toggle")
                                .with(Button)
                                .with(BandToggle(Band::Header))
                                // Top-right corner, 10px in and 8px down. The engine folds the
                                // pivot into the layout box, so the hit-box matches where it draws.
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
                // The banner covers the whole screen while paused or finished.
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
                                .with(Text::new("PAUSED"))
                                .with(TextFont::from_font_size(28.0))
                                .with(TextColor(Color::WHITE)),
                        ),
                )
                // Footer: the controls row. A floating joystick fills it, or a four-way D-pad; the
                // switch button (top-left) flips between them, the toggle (top-right) collapses it.
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
                        // The joystick fills the LEFT part of the footer only ("drag in the lower
                        // left"); the right part is left clear for the switch and the toggle, so a
                        // tap on them is never swallowed by the stick's touch zone. `with_mouse`
                        // lets a desktop try it too.
                        .child(
                            Object::new("Move Stick")
                                .with(
                                    VirtualJoystick::new(52.0)
                                        .dead_zone(0.12)
                                        .with_mouse()
                                        .source("move"),
                                )
                                .with(RectTransform {
                                    anchor_min: Vec2::new(0.0, 0.0),
                                    anchor_max: Vec2::new(0.42, 1.0),
                                    pivot: Vec2::new(0.5, 0.5),
                                    position: Vec2::new(0.0, 0.0),
                                    size: Vec2::new(0.0, 0.0),
                                }),
                        )
                        // Right pad: Fire / Switch / Reload-Dash (labels updated at runtime).
                        // BandContent so collapse hides it with the rest of the footer.
                        .child(
                            Object::new("Action Pad")
                                .with(BandContent(Band::Footer))
                                .with(RectTransform {
                                    anchor_min: Vec2::new(0.55, 0.08),
                                    anchor_max: Vec2::new(0.98, 0.92),
                                    pivot: Vec2::new(0.5, 0.5),
                                    position: Vec2::new(0.0, 0.0),
                                    size: Vec2::new(0.0, 0.0),
                                })
                                .with(Node {
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::SpaceEvenly,
                                    align_items: AlignItems::Stretch,
                                    row_gap: px(6),
                                    ..default()
                                })
                                .child(
                                    Object::new("Fire Button")
                                        .with(Button)
                                        .with(TouchButton("fire"))
                                        .with(PadSlot::Fire)
                                        .with(Node {
                                            flex_grow: 1.0,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border_radius: BorderRadius::all(px(8)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(PRIMARY))
                                        .child(
                                            Object::new("Fire Label")
                                                .with(PadLabel)
                                                .with(Text::new("Fire"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(PRIMARY_TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Switch Button")
                                        .with(Button)
                                        .with(TouchButton("interact"))
                                        .with(PadSlot::Switch)
                                        .with(Node {
                                            flex_grow: 1.0,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border_radius: BorderRadius::all(px(8)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Switch Label")
                                                .with(PadLabel)
                                                .with(Text::new("Switch"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Reload Dash Button")
                                        .with(Button)
                                        .with(TouchButton("reload_dash"))
                                        .with(PadSlot::ReloadDash)
                                        .with(Node {
                                            flex_grow: 1.0,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border_radius: BorderRadius::all(px(8)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Reload Dash Label")
                                                .with(PadLabel)
                                                .with(Text::new("Dash"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                ),
                        )
                        // The D-pad: Left, an Up / Down column, Right. Shown instead of the stick.
                        .child(
                            Object::new("Dpad")
                                .with(Dpad)
                                .with(RectTransform::anchored(
                                    Anchor::Middle,
                                    Vec2::new(0.0, 0.0),
                                    Vec2::new(0.0, 0.0),
                                ))
                                .with(Node {
                                    align_items: AlignItems::Center,
                                    column_gap: px(8),
                                    ..default()
                                })
                                .child(
                                    Object::new("Left Button")
                                        .with(Button)
                                        .with(TouchButton("dpad_left"))
                                        .with(Node {
                                            padding: UiRect::axes(px(16), px(10)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Left Label")
                                                .with(Text::new("Left"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Dpad Mid")
                                        .with(Node {
                                            flex_direction: FlexDirection::Column,
                                            row_gap: px(8),
                                            ..default()
                                        })
                                        .child(
                                            Object::new("Up Button")
                                                .with(Button)
                                                .with(TouchButton("dpad_up"))
                                                .with(Node {
                                                    padding: UiRect::axes(px(16), px(10)),
                                                    border_radius: BorderRadius::all(px(6)),
                                                    ..default()
                                                })
                                                .with(BackgroundColor(GHOST))
                                                .child(
                                                    Object::new("Up Label")
                                                        .with(Text::new("Up"))
                                                        .with(TextFont::from_font_size(13.0))
                                                        .with(TextColor(TEXT)),
                                                ),
                                        )
                                        .child(
                                            Object::new("Down Button")
                                                .with(Button)
                                                .with(TouchButton("dpad_down"))
                                                .with(Node {
                                                    padding: UiRect::axes(px(16), px(10)),
                                                    border_radius: BorderRadius::all(px(6)),
                                                    ..default()
                                                })
                                                .with(BackgroundColor(GHOST))
                                                .child(
                                                    Object::new("Down Label")
                                                        .with(Text::new("Down"))
                                                        .with(TextFont::from_font_size(13.0))
                                                        .with(TextColor(TEXT)),
                                                ),
                                        ),
                                )
                                .child(
                                    Object::new("Right Button")
                                        .with(Button)
                                        .with(TouchButton("dpad_right"))
                                        .with(Node {
                                            padding: UiRect::axes(px(16), px(10)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Right Label")
                                                .with(Text::new("Right"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
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
