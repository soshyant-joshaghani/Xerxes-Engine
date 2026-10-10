//! The arena: camera, floor, the four walls and the obstacles. The player, the coins and the
//! HUD are spawned by the top-down logic when the level starts.

use bevy::camera::ScalingMode;
use xerxes_engine::modules::physics::{Body2d, rect};
use xerxes_engine::prelude::*;


const WALL_COLOR: Color = Color::srgb(0.32, 0.36, 0.45);

fn block(name: &str, center: Vec2, size: Vec2) -> Object {
    Object::new(name)
        .with(SpriteDef {
            image: None,
            color: WALL_COLOR,
            size: Some(size),
        })
        .with(Transform::from_xyz(center.x, center.y, 0.5))
        .with(Body2d::fixed(rect(size)))
}

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Camera")
                .with(Camera2d)
                .with(Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::AutoMin {
                        min_width: 880.0,
                        min_height: 580.0,
                    },
                    ..OrthographicProjection::default_2d()
                })),
            Object::new("Floor").with(SpriteDef {
                image: None,
                color: Color::srgb(0.11, 0.13, 0.17),
                size: Some(Vec2::new(800.0, 500.0)),
            }),
            block("Wall North", Vec2::new(0.0, 240.0), Vec2::new(800.0, 20.0)),
            block("Wall South", Vec2::new(0.0, -240.0), Vec2::new(800.0, 20.0)),
            block("Wall West", Vec2::new(-390.0, 0.0), Vec2::new(20.0, 500.0)),
            block("Wall East", Vec2::new(390.0, 0.0), Vec2::new(20.0, 500.0)),
            block("Pillar West", Vec2::new(-150.0, 60.0), Vec2::new(40.0, 220.0)),
            block("Pillar East", Vec2::new(150.0, -60.0), Vec2::new(40.0, 220.0)),
            block("Bar North", Vec2::new(0.0, 170.0), Vec2::new(160.0, 30.0)),
            block("Bar East", Vec2::new(280.0, 150.0), Vec2::new(100.0, 30.0)),
        ],
    }
}
