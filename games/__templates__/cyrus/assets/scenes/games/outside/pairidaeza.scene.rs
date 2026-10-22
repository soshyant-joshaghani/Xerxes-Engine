//! Pairidaeza (Sandbox): 3D physics lab (slice 0). Spawn cubes and spheres, drag them,
//! tilt gravity. Garden-builder slices come later. GDD:
//! `__docs__/gdd/cyrus/sandbox-pairidaeza.md`.
//!
//! Scenes are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::modules::physics::{Body3d, Shape3d};
use xerxes_engine::prelude::*;

use crate::pairidaeza::{
    GravityLabel, GravitySlider, HintText, LabCamera, ModeLabel, Plot, SpawnKind, Toy, ToyKind,
    WorkshopAction, WorkshopButton,
};

const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const DIM: Color = Color::srgb(0.62, 0.65, 0.70);
const STATUS: Color = Color::srgb(0.984, 0.827, 0.302);
const PRIMARY: Color = Color::srgb(0.22, 0.741, 0.973);
const PRIMARY_TEXT: Color = Color::srgb(0.043, 0.071, 0.125);
const PANEL: Color = Color::srgba(0.047, 0.055, 0.071, 0.92);
const GROUND: Color = Color::srgb(0.45, 0.52, 0.38);
const CUBE: Color = Color::srgb(0.72, 0.48, 0.28);
const BALL: Color = Color::srgb(0.30, 0.66, 0.92);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D3,
        objects: vec![
            Object::prefab(crate::assets::prefabs::camera_prefab::prefab).with(LabCamera)
                .with(Transform::from_xyz(6.0, 7.0, 9.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y)),
            Object::new("Sun")
                .with(DirectionalLight {
                    illuminance: 12_000.0,
                    shadows_enabled: true,
                    ..default()
                })
                .with(Transform::from_xyz(4.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y)),
            Object::new("Plot")
                .with(Plot)
                .child(
                    Object::new("Ground")
                        .with(MeshDef::Plane { size: 16.0 })
                        .with(MaterialDef {
                            color: GROUND,
                            texture: None,
                        })
                        .with(Body3d::fixed(Shape3d::Box {
                            half: [8.0, 0.02, 8.0],
                        })),
                )
                // Walls: unit cube scaled; Body3d half is the unscaled cube, then * scale.
                .child(
                    Object::prefab(crate::assets::prefabs::games::outside::pairidaeza::wall_prefab::prefab).named("Wall North").with(Transform::from_xyz(0.0, 0.75, -7.5).with_scale(Vec3::new(15.0, 1.5, 0.3))))
                .child(
                    Object::prefab(crate::assets::prefabs::games::outside::pairidaeza::wall_prefab::prefab).named("Wall South").with(Transform::from_xyz(0.0, 0.75, 7.5).with_scale(Vec3::new(15.0, 1.5, 0.3))))
                .child(
                    Object::prefab(crate::assets::prefabs::games::outside::pairidaeza::wall_prefab::prefab).named("Wall West").with(Transform::from_xyz(-7.5, 0.75, 0.0).with_scale(Vec3::new(0.3, 1.5, 15.0))))
                .child(
                    Object::prefab(crate::assets::prefabs::games::outside::pairidaeza::wall_prefab::prefab).named("Wall East").with(Transform::from_xyz(7.5, 0.75, 0.0).with_scale(Vec3::new(0.3, 1.5, 15.0))))
                .child(
                    Object::new("Starter Cube")
                        .with(Toy {
                            kind: ToyKind::Cube,
                            half_extent: 0.4,
                        })
                        .with(MeshDef::Cube { size: 0.8 })
                        .with(MaterialDef {
                            color: CUBE,
                            texture: None,
                        })
                        .with(Transform::from_xyz(-1.2, 1.0, 0.0))
                        .with(Body3d::dynamic(Shape3d::Box {
                            half: [0.4, 0.4, 0.4],
                        })),
                )
                .child(
                    Object::new("Starter Ball")
                        .with(Toy {
                            kind: ToyKind::Ball,
                            half_extent: 0.4,
                        })
                        .with(MeshDef::Sphere { radius: 0.4 })
                        .with(MaterialDef {
                            color: BALL,
                            texture: None,
                        })
                        .with(Transform::from_xyz(1.2, 1.2, 0.5))
                        .with(Body3d::dynamic(Shape3d::Ball { radius: 0.4 })),
                ),
            Object::new("HUD")
                .with(Canvas::overlay())
                .with(Node {
                    width: percent(100),
                    height: percent(100),
                    ..default()
                })
                .child(
                    Object::new("Bar")
                        .with(Node {
                            position_type: PositionType::Absolute,
                            top: px(12),
                            left: px(12),
                            right: px(12),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(px(14), px(10)),
                            column_gap: px(12),
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
                                        .with(Text::new("Pairidaeza"))
                                        .with(TextFont::from_font_size(18.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Hint")
                                        .with(HintText)
                                        .with(Text::new(
                                            "Grab: drag toys  |  Camera: orbit / pan / zoom  |  Spawn  |  Gravity",
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
                                                .with(Text::new("Gravity 1.00x"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(STATUS)),
                                        )
                                        .child(
                                            Object::new("Gravity Slider")
                                                .with(GravitySlider)
                                                .with(Slider::new(0.0, 1.5, 1.0))
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
                                .with(Node {
                                    align_items: AlignItems::Center,
                                    column_gap: px(8),
                                    ..default()
                                })
                                .child(
                                    Object::prefab(crate::assets::prefabs::hud_button_prefab::prefab).named("Mode Button").with(WorkshopButton(WorkshopAction::ToggleMode))
                                        .child(
                                            Object::new("Mode Label")
                                                .with(ModeLabel)
                                                .with(Text::new("Grab"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(
                                    Object::prefab(crate::assets::prefabs::hud_button_prefab::prefab).named("Cube Button").with(WorkshopButton(WorkshopAction::Spawn(SpawnKind::Cube))).with(BackgroundColor(PRIMARY))
                                        .child(
                                            Object::new("Cube Label")
                                                .with(Text::new("Cube"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(PRIMARY_TEXT)),
                                        ),
                                )
                                .child(
                                    Object::prefab(crate::assets::prefabs::hud_button_prefab::prefab).named("Ball Button").with(WorkshopButton(WorkshopAction::Spawn(SpawnKind::Ball))).with(BackgroundColor(PRIMARY))
                                        .child(
                                            Object::new("Ball Label")
                                                .with(Text::new("Ball"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(PRIMARY_TEXT)),
                                        ),
                                )
                                .child(
                                    Object::prefab(crate::assets::prefabs::hud_button_prefab::prefab).named("Reset Button").with(WorkshopButton(WorkshopAction::Reset))
                                        .child(
                                            Object::new("Reset Label")
                                                .with(Text::new("Reset"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(Object::prefab(crate::assets::prefabs::hud_back_button_prefab::prefab)),
                        ),
                ),
        ],
    }
}
