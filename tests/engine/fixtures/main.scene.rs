use crate::assets::logic::camera_controller_3d::CameraController3d;
use crate::assets::logic::goal_zone_3d::GoalZone3d;
use crate::assets::logic::hud::Hud;
use xerxes_engine::prelude::*;

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D3,
        objects: vec![
            Object::new("Camera")
                .with(Camera3d::default())
                .with(Transform::from_xyz(0.0, 9.0, 12.0))
                .with(CameraController3d {
                    offset: Vec3::new(0.0, 9.0, 12.0),
                }),
            Object::new("Sun")
                .with(DirectionalLight {
                    shadows_enabled: true,
                    ..default()
                })
                .with(Transform::from_xyz(4.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y)),
            Object::new("Ground")
                .with(MeshDef::Plane { size: 20.0 })
                .with(MaterialDef {
                    color: Color::srgb(0.24, 0.42, 0.3),
                    texture: None,
                }),
            Object::prefab(crate::assets::prefabs::player_prefab::prefab)
                .with(Transform::from_xyz(0.0, 0.5, 0.0)),
            Object::new("Crate")
                .with(Transform::from_xyz(3.0, 0.5, -2.0))
                .with(MeshDef::Cube { size: 1.0 })
                .with(MaterialDef {
                    color: Color::WHITE,
                    texture: Some("art/checker.png"),
                }),
            Object::new("Bonus Crate")
                .with(Transform::from_xyz(-3.0, 0.5, -2.0))
                .with(MeshDef::Cube { size: 1.0 })
                .with(MaterialDef {
                    color: Color::WHITE,
                    texture: None,
                })
                .with(BundleTexture {
                    bundle: "extra",
                    texture: "art/stripes.png",
                }),
            Object::new("Goal")
                .with(Transform::from_xyz(0.0, 0.0, -8.0))
                .with(GoalZone3d { radius: 1.5 }),
            Object::new("HUD").with(Hud {
                title: "Courtyard: reach the green ring",
                hint: "WASD / arrows: move   R: respawn   P / Esc: pause   M: menu",
            }),
        ],
    }
}
