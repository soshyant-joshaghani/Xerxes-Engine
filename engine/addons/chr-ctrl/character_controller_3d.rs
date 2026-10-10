//! Template: 3D character controller. Moves its object on the ground plane (XZ) with the
//! `move_left/right/up/down` actions, inside ±`bounds`; the `respawn` action puts it back
//! where it started. The camera looks down -Z, so `move_up` moves away from it.
//! needs: pause

use xerxes_engine::prelude::*;

use super::pause::Paused;

#[derive(Component, Clone, Debug, PartialEq)]
pub struct CharacterController3d {
    pub speed: f32,
    pub bounds: f32,
}

/// Where the character was spawned.
#[derive(Component, Debug, Clone, Copy)]
struct SpawnPoint(Vec3);

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (remember_spawn, respawn, walk).chain());
}

fn remember_spawn(
    mut commands: Commands,
    added: Query<(Entity, &Transform), Added<CharacterController3d>>,
) {
    for (entity, transform) in &added {
        commands
            .entity(entity)
            .insert(SpawnPoint(transform.translation));
    }
}

fn respawn(actions: Res<Actions>, mut characters: Query<(&mut Transform, &SpawnPoint)>) {
    if actions.just_pressed("respawn") {
        for (mut transform, spawn) in &mut characters {
            transform.translation = spawn.0;
        }
    }
}

/// Screen-relative direction on the ground plane.
pub fn direction(actions: &Actions) -> Vec3 {
    Vec3::new(
        actions.axis("move_left", "move_right"),
        0.0,
        actions.axis("move_up", "move_down"),
    )
    .normalize_or_zero()
}

fn walk(
    actions: Res<Actions>,
    paused: Res<Paused>,
    time: Res<Time>,
    mut characters: Query<(&mut Transform, &CharacterController3d)>,
) {
    if paused.0 {
        return;
    }
    let direction = direction(&actions);
    for (mut transform, controller) in &mut characters {
        let next = transform.translation + direction * controller.speed * time.delta_secs();
        transform.translation.x = next.x.clamp(-controller.bounds, controller.bounds);
        transform.translation.z = next.z.clamp(-controller.bounds, controller.bounds);
    }
}
