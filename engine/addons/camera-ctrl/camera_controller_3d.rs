//! Template: 3D follow camera. Keeps its object at `offset` from the object marked
//! [`CameraTarget3d`], looking at it.

use xerxes_engine::prelude::*;

#[derive(Component, Clone, Debug, PartialEq)]
pub struct CameraController3d {
    pub offset: Vec3,
}

/// The object 3D cameras follow.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct CameraTarget3d;

pub fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, follow.before(TransformSystems::Propagate));
}

fn follow(
    target: Query<&Transform, (With<CameraTarget3d>, Without<CameraController3d>)>,
    mut cameras: Query<(&mut Transform, &CameraController3d)>,
) {
    let Ok(target) = target.single() else { return };
    for (mut transform, camera) in &mut cameras {
        *transform = Transform::from_translation(target.translation + camera.offset)
            .looking_at(target.translation, Vec3::Y);
    }
}
