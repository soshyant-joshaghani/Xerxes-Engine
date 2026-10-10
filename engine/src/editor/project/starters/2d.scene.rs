//! Template: an empty 2D scene: a camera and a backdrop sprite.

use xerxes_engine::prelude::*;

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Camera").with(Camera2d),
            Object::new("Backdrop")
                .with(SpriteDef {
                    image: None,
                    color: Color::srgb(0.11, 0.13, 0.17),
                    size: Some(Vec2::new(800.0, 500.0)),
                })
                .with(Transform::from_xyz(0.0, 0.0, -1.0)),
        ],
    }
}
