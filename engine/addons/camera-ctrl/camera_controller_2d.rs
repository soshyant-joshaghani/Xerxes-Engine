//! Template: 2D follow camera. Eases its object toward the object marked [`CameraTarget2d`]
//! on XY (`smoothing` per second; 0 = snap).

use xerxes_engine::prelude::*;

#[derive(Component, Clone, Debug, PartialEq)]
pub struct CameraController2d {
    pub smoothing: f32,
}

/// The object 2D cameras follow.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct CameraTarget2d;

pub fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, follow.before(TransformSystems::Propagate));
}

fn follow(
    time: Res<Time>,
    target: Query<&Transform, (With<CameraTarget2d>, Without<CameraController2d>)>,
    mut cameras: Query<(&mut Transform, &CameraController2d)>,
) {
    let Ok(target) = target.single() else { return };
    for (mut transform, camera) in &mut cameras {
        let goal = target.translation.truncate();
        let at = transform.translation.truncate();
        let next = if camera.smoothing <= 0.0 {
            goal
        } else {
            at.lerp(goal, (camera.smoothing * time.delta_secs()).min(1.0))
        };
        transform.translation.x = next.x;
        transform.translation.y = next.y;
    }
}
