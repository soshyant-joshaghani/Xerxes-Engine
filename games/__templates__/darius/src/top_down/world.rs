//! The rules of the arena: a player square moved by the physics module (it slides along the
//! scene's walls), coins that are sensors, a score. The arena itself (floor, walls, camera) is
//! the scene `assets/scenes/arena.scene.rs`. Units are world pixels.

use bevy::prelude::*;
use xerxes_engine::modules::flow::{Flow, LevelEntity};
use xerxes_engine::modules::matches::{ActiveMatch, MatchPause, MatchReport, Phase, Report};
use xerxes_engine::modules::physics::{Body2d, ContactEvent, Mover, rect};

const PLAYER_SIZE: f32 = 28.0;
const PLAYER_SPEED: f32 = 260.0;
const COIN_SIZE: f32 = 14.0;
const SPAWN: Vec2 = Vec2::new(-340.0, -190.0);

const COINS: [Vec2; 8] = [
    Vec2::new(-340.0, 190.0),
    Vec2::new(-250.0, -150.0),
    Vec2::new(-60.0, -180.0),
    Vec2::new(0.0, 60.0),
    Vec2::new(100.0, 215.0),
    Vec2::new(240.0, -190.0),
    Vec2::new(340.0, 190.0),
    Vec2::new(300.0, 0.0),
];

pub const COIN_COUNT: usize = COINS.len();

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
                    (apply_actions, toggle_pause_key),
                    (steer_player, collect_coins).chain(),
                )
                    .chain()
                    .run_if(in_state(Flow::Level)),
            );
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Coin;

/// Starts a level: the player and the coins. The arena is the scene's, the match the engine's.
fn setup_level(mut commands: Commands, mut pause: ResMut<MatchPause>) {
    pause.0 = false;
    spawn_actors(&mut commands);
}

fn spawn_actors(commands: &mut Commands) {
    commands.spawn((
        LevelEntity,
        Player,
        Body2d::kinematic(rect(Vec2::splat(PLAYER_SIZE))),
        Mover::default(),
        Sprite::from_color(Color::srgb(0.22, 0.74, 0.97), Vec2::splat(PLAYER_SIZE)),
        Transform::from_translation(SPAWN.extend(2.0)),
    ));
    for at in COINS {
        commands.spawn((
            LevelEntity,
            Coin,
            Body2d::fixed(rect(Vec2::splat(COIN_SIZE))).sensor(),
            Sprite::from_color(Color::srgb(0.98, 0.82, 0.30), Vec2::splat(COIN_SIZE)),
            Transform::from_translation(at.extend(1.0)),
        ));
    }
}

fn apply_actions(
    mut commands: Commands,
    mut actions: MessageReader<Action>,
    mut pause: ResMut<MatchPause>,
    actors: Query<Entity, Or<(With<Player>, With<Coin>)>>,
) {
    for action in actions.read() {
        match action {
            Action::TogglePause => pause.0 = !pause.0,
            Action::Restart => {
                for actor in &actors {
                    commands.entity(actor).despawn();
                }
                spawn_actors(&mut commands);
                // No match: the engine starts a fresh one.
                commands.remove_resource::<ActiveMatch>();
                pause.0 = false;
            }
        }
    }
}

fn toggle_pause_key(keys: Res<ButtonInput<KeyCode>>, mut pause: ResMut<MatchPause>) {
    if keys.just_pressed(KeyCode::Escape) {
        pause.0 = !pause.0;
    }
}

fn input_direction(keys: &ButtonInput<KeyCode>) -> Vec2 {
    let axis = |neg: [KeyCode; 2], pos: [KeyCode; 2]| {
        keys.any_pressed(pos) as i8 as f32 - keys.any_pressed(neg) as i8 as f32
    };
    let x = axis(
        [KeyCode::KeyA, KeyCode::ArrowLeft],
        [KeyCode::KeyD, KeyCode::ArrowRight],
    );
    let y = axis(
        [KeyCode::KeyS, KeyCode::ArrowDown],
        [KeyCode::KeyW, KeyCode::ArrowUp],
    );
    Vec2::new(x, y).normalize_or_zero()
}

/// The keys say where the player wants to go; the physics module walks it and stops it at walls.
fn steer_player(
    keys: Res<ButtonInput<KeyCode>>,
    pause: Res<MatchPause>,
    active: Option<Res<ActiveMatch>>,
    mut mover: Single<&mut Mover, With<Player>>,
) {
    let playing = active.is_some_and(|m| m.0.phase() == Phase::Playing);
    mover.wish = if playing && !pause.0 {
        input_direction(&keys) * PLAYER_SPEED
    } else {
        Vec2::ZERO
    };
}

fn collect_coins(
    mut commands: Commands,
    mut reports: MessageWriter<MatchReport>,
    mut contacts: MessageReader<ContactEvent>,
    player: Single<Entity, With<Player>>,
    coins: Query<(), With<Coin>>,
) {
    for contact in contacts.read() {
        if !contact.started {
            continue;
        }
        if let Some(coin) = contact.other(*player).filter(|e| coins.contains(*e)) {
            commands.entity(coin).despawn();
            reports.write(MatchReport(Report::Progress { side: 0, amount: 1 }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coins_are_inside_the_walls() {
        // The arena is 800 x 500 with 20 thick walls (assets/scenes/arena.scene.rs).
        for coin in COINS {
            assert!(
                coin.x.abs() < 390.0 - COIN_SIZE / 2.0 && coin.y.abs() < 240.0 - COIN_SIZE / 2.0
            );
        }
        assert!(SPAWN.x.abs() < 370.0 && SPAWN.y.abs() < 220.0);
    }
}
