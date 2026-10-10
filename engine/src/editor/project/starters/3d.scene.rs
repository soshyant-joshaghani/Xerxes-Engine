//! Template: an empty 3D scene: a camera, a sun and a ground plane.

use xerxes_engine::prelude::*;

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D3,
        objects: vec![
            Object::new("Camera")
                .with(Camera3d::default())
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
                    color: Color::srgb(0.24, 0.42, 0.3),
                    texture: None,
                }),
        ],
    }
}
