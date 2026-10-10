//! Template: 2D top-down character controller. Moves its object on XY with the
//! `move_left/right/up/down` actions; the `respawn` action puts it back where it started.
//! needs: pause

use xerxes_engine::prelude::*;

use super::pause::Paused;

#[derive(Component, Clone, Debug, PartialEq)]
pub struct CharacterController2d {
    pub speed: f32,
}

#[derive(Component, Debug, Clone, Copy)]
struct SpawnPoint2d(Vec3);

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (remember_spawn, respawn, walk).chain());
}

fn remember_spawn(
    mut commands: Commands,
    added: Query<(Entity, &Transform), Added<CharacterController2d>>,
) {
    for (entity, transform) in &added {
        commands
            .entity(entity)
            .insert(SpawnPoint2d(transform.translation));
    }
}

fn respawn(actions: Res<Actions>, mut characters: Query<(&mut Transform, &SpawnPoint2d)>) {
    if actions.just_pressed("respawn") {
        for (mut transform, spawn) in &mut characters {
            transform.translation = spawn.0;
        }
    }
}

fn walk(
    actions: Res<Actions>,
    paused: Res<Paused>,
    time: Res<Time>,
    mut characters: Query<(&mut Transform, &CharacterController2d)>,
) {
    if paused.0 {
        return;
    }
    let direction = Vec2::new(
        actions.axis("move_left", "move_right"),
        actions.axis("move_down", "move_up"),
    )
    .normalize_or_zero();
    for (mut transform, controller) in &mut characters {
        transform.translation += (direction * controller.speed * time.delta_secs()).extend(0.0);
    }
}
