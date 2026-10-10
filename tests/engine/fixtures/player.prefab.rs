use crate::assets::logic::camera_controller_3d::CameraTarget3d;
use crate::assets::logic::character_controller_3d::CharacterController3d;
use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Player")
            .with(MeshDef::Cube { size: 1.0 })
            .with(MaterialDef {
                color: Color::srgb(0.95, 0.45, 0.12),
                texture: None,
            })
            .with(CharacterController3d {
                speed: 6.0,
                bounds: 9.5,
            })
            .with(CameraTarget3d),
    }
}
