use crate::assets::logic::camera_controller_3d::CameraController3d;
use crate::assets::logic::hud::Hud;
use xerxes_engine::prelude::*;

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D3,
        objects: vec![
            Object::new("Camera")
                .with(Camera3d::default())
                .with(Transform::from_xyz(0.0, 12.0, 14.0))
                .with(CameraController3d {
                    offset: Vec3::new(0.0, 12.0, 14.0),
                }),
            Object::new("Sun")
                .with(DirectionalLight {
                    shadows_enabled: true,
                    ..default()
                })
                .with(Transform::from_xyz(-4.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y)),
            Object::new("Ground")
                .with(MeshDef::Plane { size: 20.0 })
                .with(MaterialDef {
                    color: Color::srgb(0.2, 0.25, 0.36),
                    texture: None,
                }),
            Object::prefab(crate::assets::prefabs::player_prefab::prefab)
                .with(Transform::from_xyz(0.0, 0.5, 0.0)),
            Object::new("Pillar")
                .with(Transform::from_xyz(4.0, 1.0, 0.0))
                .with(MeshDef::Sphere { radius: 1.0 })
                .with(MaterialDef {
                    color: Color::WHITE,
                    texture: Some("art/checker.png"),
                }),
            Object::new("HUD").with(Hud {
                title: "Sandbox: no goal, just play",
                hint: "WASD / arrows: move   R: respawn   P / Esc: pause   M: menu",
            }),
        ],
    }
}
