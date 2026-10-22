//! Courtyard: darics still score; combat (pistol / enemy / portal) lives in `combat.rs`.

use bevy::prelude::*;
use xerxes_engine::modules::flow::{CurrentLevel, Flow, LoadLevel, Play};
use xerxes_engine::modules::input::Actions;
use xerxes_engine::modules::matches::{ActiveMatch, MatchPause, MatchReport, Report};
use xerxes_engine::modules::physics::{ContactEvent, PhysicsWorld};

/// Darics in the scene (optional loot; clearing the room is the enemy + portal).
pub const DARIC_COUNT: usize = 8;

/// What the HUD asks the world to do.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub enum Action {
    TogglePause,
    Restart,
}

pub struct TopDownPlugin;

impl Plugin for TopDownPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Action>()
            .add_systems(OnEnter(Flow::Level), setup_level)
            .add_systems(
                Update,
                (
                    (apply_actions.in_set(Play), toggle_pause_key),
                    collect_darics.in_set(Play),
                )
                    .chain()
                    .run_if(in_state(Flow::Level)),
            );
    }
}

/// The player object of the scene.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Player;

/// A daric object of the scene.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Daric;

fn setup_level(mut pause: ResMut<MatchPause>, mut world: Option<ResMut<PhysicsWorld>>) {
    pause.0 = false;
    if let Some(world) = world.as_mut() {
        world.sim.set_gravity([0.0, 0.0]);
    }
}

fn apply_actions(
    mut commands: Commands,
    mut actions: MessageReader<Action>,
    mut pause: ResMut<MatchPause>,
    level: Res<CurrentLevel>,
    mut load: MessageWriter<LoadLevel>,
) {
    for action in actions.read() {
        match action {
            Action::TogglePause => pause.0 = !pause.0,
            Action::Restart => {
                load.write(LoadLevel(level.0));
                commands.remove_resource::<ActiveMatch>();
                pause.0 = false;
            }
        }
    }
}

fn toggle_pause_key(actions: Res<Actions>, mut pause: ResMut<MatchPause>) {
    if actions.just_pressed("pause") {
        pause.0 = !pause.0;
    }
}

fn collect_darics(
    mut commands: Commands,
    mut reports: MessageWriter<MatchReport>,
    mut contacts: MessageReader<ContactEvent>,
    player: Query<Entity, With<Player>>,
    darics: Query<(), With<Daric>>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    for contact in contacts.read() {
        if !contact.started {
            continue;
        }
        if let Some(daric) = contact.other(player).filter(|e| darics.contains(*e)) {
            commands.entity(daric).despawn();
            reports.write(MatchReport(Report::Score { side: 0, amount: 1 }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xerxes_engine::modules::assets::spawn_scene;

    #[test]
    fn the_scene_has_the_player_the_darics_and_the_hud() {
        let mut world = World::new();
        spawn_scene(
            &mut world,
            &crate::assets::scenes::games::solo::ahrimans_horde_scene::scene(),
        );
        let darics: Vec<Vec3> = world
            .query_filtered::<&Transform, With<Daric>>()
            .iter(&world)
            .map(|t| t.translation)
            .collect();
        assert_eq!(darics.len(), DARIC_COUNT);
        for c in darics {
            assert!(c.x.abs() < 390.0 - 7.0 && c.y.abs() < 240.0 - 7.0, "{c:?}");
        }
        assert_eq!(world.query::<&Player>().iter(&world).count(), 1);
        assert_eq!(
            world
                .query::<&crate::ahrimans_horde::HudButton>()
                .iter(&world)
                .count(),
            2
        );
    }
}
