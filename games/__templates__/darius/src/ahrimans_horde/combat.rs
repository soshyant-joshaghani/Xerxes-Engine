//! Slice 1 combat: unlimited pistol, plant-to-fire, reload / dash, one rusher, portal on kill,
//! plus a floor gun and a health pickup.

use bevy::prelude::*;
use xerxes_engine::modules::flow::{Flow, LevelEntity, Play};
use xerxes_engine::modules::input::Actions;
use xerxes_engine::modules::matches::{ActiveMatch, MatchPause, MatchReport, Phase, Report};
use xerxes_engine::modules::physics::{
    Body2d, ContactEvent, Mover, PhysicsBody, PhysicsWorld, rect,
};

use super::world::Player;

const PLAYER_SPEED: f32 = 260.0;
const DASH_SPEED: f32 = 720.0;
const DASH_TIME: f32 = 0.12;
const MAG_SIZE: u32 = 6;
const BOW_MAG: u32 = 4;
const BOW_RESERVE: u32 = 12;
const FIRE_COOLDOWN: f32 = 0.18;
const RELOAD_TIME: f32 = 0.55;
const BULLET_SPEED: f32 = 520.0;
const BULLET_LIFE: f32 = 0.9;
const ENEMY_SPEED: f32 = 110.0;
const ENEMY_HP: i32 = 3;
const ENEMY_HIT_RANGE: f32 = 40.0;
const PICKUP_RANGE: f32 = 32.0;
const ROOM2: Vec2 = Vec2::new(280.0, -160.0);

const PISTOL_BULLET: Color = Color::srgb(0.98, 0.90, 0.35);
const BOW_BULLET: Color = Color::srgb(0.45, 0.88, 0.55);
const PORTAL: Color = Color::srgb(0.35, 0.82, 0.95);

#[derive(Resource)]
pub struct Run {
    aim: Vec2,
    mag: u32,
    reloading: f32,
    fire_cd: f32,
    dash: f32,
    iframe: f32,
    lives: u32,
    depth: u32,
    portal_open: bool,
    pending_portal: Option<Vec2>,
    cleared: bool,
    /// Floor gun claimed; Switch cycles pistol ↔ bow.
    has_bow: bool,
    bow_mag: u32,
    bow_reserve: u32,
    using_bow: bool,
}

impl Default for Run {
    fn default() -> Self {
        Self {
            aim: Vec2::X,
            mag: MAG_SIZE,
            reloading: 0.0,
            fire_cd: 0.0,
            dash: 0.0,
            iframe: 0.0,
            lives: 3,
            depth: 0,
            portal_open: false,
            pending_portal: None,
            cleared: false,
            has_bow: false,
            bow_mag: 0,
            bow_reserve: 0,
            using_bow: false,
        }
    }
}

#[derive(Component)]
struct Bullet {
    vel: Vec2,
    life: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Enemy {
    pub hp: i32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self { hp: ENEMY_HP }
    }
}

/// Floor weapon: Interact / Switch when near picks it up.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GunPickup;

/// +1 life on contact (clamped at 7).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HealthPickup;

#[derive(Component)]
pub struct Portal;

/// Label on a right-pad button; the combat systems rewrite it (Fire / Switch / Reload|Dash).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadSlot {
    Fire,
    Switch,
    ReloadDash,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PadLabel;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Run>()
            .add_systems(OnEnter(Flow::Level), reset_run)
            .add_systems(
                Update,
                (
                    // The courtyard moves only once the level's ready beat is over; the pad is
                    // drawn throughout, so the player can see what they are about to use.
                    (
                        combat_player,
                        move_bullets,
                        drive_enemies,
                        hit_player,
                        collect_pickups,
                        open_portal_on_clear,
                        enter_portal,
                    )
                        .chain()
                        .in_set(Play),
                    refresh_pad,
                )
                    .chain()
                    .run_if(in_state(Flow::Level))
                    .run_if(resource_exists::<Run>),
            );
    }
}

fn reset_run(
    mut commands: Commands,
    mut run: ResMut<Run>,
    bullets: Query<Entity, With<Bullet>>,
    portals: Query<Entity, With<Portal>>,
    // Runtime rushers were roots; the scene rusher is a child of Game and must stay.
    orphan_enemies: Query<Entity, (With<Enemy>, Without<ChildOf>)>,
) {
    *run = Run::default();
    for entity in bullets
        .iter()
        .chain(portals.iter())
        .chain(orphan_enemies.iter())
    {
        commands.entity(entity).despawn();
    }
}

fn playing(active: &Option<Res<ActiveMatch>>, pause: &MatchPause) -> bool {
    !pause.0
        && active
            .as_ref()
            .is_some_and(|m| m.0.phase() == Phase::Playing)
}

fn mag_cap(run: &Run) -> u32 {
    if run.using_bow { BOW_MAG } else { MAG_SIZE }
}

fn current_mag(run: &Run) -> u32 {
    if run.using_bow { run.bow_mag } else { run.mag }
}

fn set_current_mag(run: &mut Run, value: u32) {
    if run.using_bow {
        run.bow_mag = value;
    } else {
        run.mag = value;
    }
}

fn start_dash(run: &mut Run, stick: Vec2) {
    if stick.length_squared() > 0.04 {
        run.aim = stick.normalize();
    }
    run.dash = DASH_TIME;
    run.iframe = DASH_TIME;
}

/// Found guns with mag and reserve both empty leave the inventory (pistol stays forever).
fn drop_empty_guns(run: &mut Run) {
    if run.has_bow && run.bow_mag == 0 && run.bow_reserve == 0 {
        run.has_bow = false;
        run.using_bow = false;
        run.reloading = 0.0;
    }
}

fn try_reload(run: &mut Run) {
    let cap = mag_cap(run);
    if run.reloading > 0.0 || current_mag(run) >= cap {
        return;
    }
    if run.using_bow && run.bow_reserve == 0 {
        drop_empty_guns(run);
        return;
    }
    run.reloading = RELOAD_TIME;
}

fn finish_reload(run: &mut Run) {
    if run.using_bow {
        let need = BOW_MAG.saturating_sub(run.bow_mag);
        let take = need.min(run.bow_reserve);
        run.bow_reserve -= take;
        run.bow_mag += take;
        drop_empty_guns(run);
    } else {
        run.mag = MAG_SIZE;
    }
}

fn combat_player(
    time: Res<Time>,
    actions: Res<Actions>,
    pause: Res<MatchPause>,
    active: Option<Res<ActiveMatch>>,
    mut run: ResMut<Run>,
    mut commands: Commands,
    mut player: Query<(&Transform, &mut Mover), With<Player>>,
) {
    let Ok((transform, mut mover)) = player.single_mut() else {
        return;
    };
    if !playing(&active, &pause) {
        mover.wish = Vec2::ZERO;
        return;
    }

    let dt = time.delta_secs();
    run.fire_cd = (run.fire_cd - dt).max(0.0);
    run.iframe = (run.iframe - dt).max(0.0);
    if run.dash > 0.0 {
        run.dash = (run.dash - dt).max(0.0);
        mover.wish = run.aim * DASH_SPEED;
        return;
    }

    if run.reloading > 0.0 {
        run.reloading -= dt;
        if run.reloading <= 0.0 {
            run.reloading = 0.0;
            finish_reload(&mut run);
        }
    }

    let stick = actions.vector("move").clamp_length_max(1.0);
    // RMB = fire (also drives the stick when dragged on it). LMB on the stick = move.
    // While firing you stay planted; stick / WASD only update aim.
    let fire = actions.pressed("fire");
    let plant = fire;

    let at = transform.translation.truncate();
    if stick.length_squared() > 0.04 {
        run.aim = stick.normalize();
    }

    if plant {
        mover.wish = Vec2::ZERO;
        if fire {
            let mag = current_mag(&run);
            if run.reloading <= 0.0 && run.fire_cd <= 0.0 && mag > 0 {
                set_current_mag(&mut run, mag - 1);
                run.fire_cd = FIRE_COOLDOWN;
                let dir = run.aim;
                let origin = at + dir * 20.0;
                let color = if run.using_bow {
                    BOW_BULLET
                } else {
                    PISTOL_BULLET
                };
                let size = if run.using_bow {
                    Vec2::new(14.0, 5.0)
                } else {
                    Vec2::new(10.0, 6.0)
                };
                commands.spawn((
                    LevelEntity,
                    Bullet {
                        vel: dir * BULLET_SPEED,
                        life: BULLET_LIFE,
                    },
                    Sprite::from_color(color, size),
                    Transform::from_translation(origin.extend(3.0))
                        .with_rotation(Quat::from_rotation_z(dir.y.atan2(dir.x))),
                ));
                if current_mag(&run) == 0 {
                    try_reload(&mut run);
                    drop_empty_guns(&mut run);
                }
            } else if current_mag(&run) == 0 && run.reloading <= 0.0 {
                try_reload(&mut run);
                drop_empty_guns(&mut run);
            }
        }
    } else {
        mover.wish = stick * PLAYER_SPEED;
    }

    // R / pad / touch: reload when mag is not full, dash when it is.
    if actions.just_pressed("reload_dash") {
        if current_mag(&run) >= mag_cap(&run) && run.reloading <= 0.0 {
            start_dash(&mut run, stick);
        } else {
            try_reload(&mut run);
        }
    }

    // Switch weapon (gun is picked up by walking onto it).
    if actions.just_pressed("interact") && run.has_bow {
        run.using_bow = !run.using_bow;
        run.reloading = 0.0;
    }
}

fn move_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut run: ResMut<Run>,
    mut bullets: Query<(Entity, &mut Bullet, &mut Transform)>,
    mut enemies: Query<(Entity, &mut Enemy, &Transform), Without<Bullet>>,
    world: Option<Res<PhysicsWorld>>,
) {
    let dt = time.delta_secs();
    let Some(world) = world else {
        return;
    };
    for (entity, mut bullet, mut transform) in &mut bullets {
        bullet.life -= dt;
        if bullet.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let from = transform.translation.truncate();
        let step = bullet.vel * dt;
        let dist = step.length().max(0.001);
        let dir = step / dist;
        let next = from + step;
        let mut hit_enemy = false;
        for (enemy_e, mut enemy, enemy_tf) in &mut enemies {
            if enemy_tf.translation.truncate().distance(next) < 18.0 {
                enemy.hp -= 1;
                if enemy.hp <= 0 {
                    run.pending_portal = Some(enemy_tf.translation.truncate());
                    commands.entity(enemy_e).despawn();
                }
                hit_enemy = true;
                break;
            }
        }
        if hit_enemy {
            commands.entity(entity).despawn();
            continue;
        }
        if world
            .sim
            .ray([from.x, from.y], [dir.x, dir.y], dist + 2.0)
            .is_some()
        {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation = next.extend(transform.translation.z);
    }
}

fn drive_enemies(
    pause: Res<MatchPause>,
    active: Option<Res<ActiveMatch>>,
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<(&Transform, &mut Mover), With<Enemy>>,
) {
    if !playing(&active, &pause) {
        for (_, mut mover) in &mut enemies {
            mover.wish = Vec2::ZERO;
        }
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let target = player.translation.truncate();
    for (transform, mut mover) in &mut enemies {
        let to = target - transform.translation.truncate();
        mover.wish = if to.length_squared() > 1.0 {
            to.normalize() * ENEMY_SPEED
        } else {
            Vec2::ZERO
        };
    }
}

fn hit_player(
    time: Res<Time>,
    pause: Res<MatchPause>,
    active: Option<Res<ActiveMatch>>,
    mut run: ResMut<Run>,
    player: Query<&Transform, With<Player>>,
    enemies: Query<&Transform, With<Enemy>>,
    mut reports: MessageWriter<MatchReport>,
    mut cool: Local<f32>,
) {
    *cool = (*cool - time.delta_secs()).max(0.0);
    if !playing(&active, &pause) || run.iframe > 0.0 || run.cleared || *cool > 0.0 {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let at = player.translation.truncate();
    for enemy in &enemies {
        // Physics keeps centers farther apart than the sprites look — use a generous range.
        if enemy.translation.truncate().distance(at) < ENEMY_HIT_RANGE {
            run.lives = run.lives.saturating_sub(1);
            run.iframe = 0.55;
            *cool = 0.55;
            if run.lives == 0 {
                reports.write(MatchReport(Report::Down { side: 0, player: 0 }));
            }
            break;
        }
    }
}

fn collect_pickups(
    mut commands: Commands,
    mut run: ResMut<Run>,
    player: Query<&Transform, With<Player>>,
    hearts: Query<(Entity, &Transform), With<HealthPickup>>,
    guns: Query<(Entity, &Transform), With<GunPickup>>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let at = player.translation.truncate();
    for (entity, heart) in &hearts {
        if heart.translation.truncate().distance(at) < PICKUP_RANGE {
            commands.entity(entity).despawn();
            if run.lives < 7 {
                run.lives += 1;
            }
        }
    }
    // Green box = floor gun. New gun → equip; already owned → ammo only (no auto-switch).
    for (entity, gun) in &guns {
        if gun.translation.truncate().distance(at) < PICKUP_RANGE {
            commands.entity(entity).despawn();
            if run.has_bow {
                run.bow_reserve = (run.bow_reserve + BOW_RESERVE).min(48);
            } else {
                run.has_bow = true;
                run.bow_mag = BOW_MAG;
                run.bow_reserve = BOW_RESERVE;
                run.using_bow = true;
                run.reloading = 0.0;
            }
        }
    }
}

fn open_portal_on_clear(
    mut commands: Commands,
    mut run: ResMut<Run>,
    portals: Query<(), With<Portal>>,
) {
    if run.portal_open || run.cleared || !portals.is_empty() {
        return;
    }
    let Some(at) = run.pending_portal.take() else {
        return;
    };
    commands.spawn((
        LevelEntity,
        Portal,
        Sprite::from_color(PORTAL, Vec2::splat(36.0)),
        Transform::from_translation(at.extend(1.5)),
        Body2d::fixed(rect(Vec2::splat(36.0))).sensor(),
    ));
    run.portal_open = true;
}

fn enter_portal(
    mut commands: Commands,
    mut run: ResMut<Run>,
    mut contacts: MessageReader<ContactEvent>,
    player: Query<(Entity, &PhysicsBody), With<Player>>,
    portals: Query<Entity, With<Portal>>,
    mut world: Option<ResMut<PhysicsWorld>>,
    mut reports: MessageWriter<MatchReport>,
    mut transforms: Query<&mut Transform>,
) {
    if run.cleared {
        return;
    }
    let Ok((player_e, PhysicsBody(body))) = player.single() else {
        return;
    };
    for contact in contacts.read() {
        if !contact.started {
            continue;
        }
        let Some(portal) = contact.other(player_e).filter(|e| portals.contains(*e)) else {
            continue;
        };
        commands.entity(portal).despawn();
        run.portal_open = false;
        run.depth += 1;
        run.lives = (run.lives + 1).min(7);
        if let Some(world) = world.as_mut() {
            world.sim.set_position(*body, [ROOM2.x, ROOM2.y]);
            world.sim.set_velocity(*body, [0.0, 0.0]);
        }
        if let Ok(mut tf) = transforms.get_mut(player_e) {
            tf.translation.x = ROOM2.x;
            tf.translation.y = ROOM2.y;
        }
        run.cleared = true;
        reports.write(MatchReport(Report::Win { side: 0 }));
        break;
    }
}

fn refresh_pad(
    run: Res<Run>,
    labels: Query<(&PadSlot, &Children)>,
    mut texts: Query<&mut Text, With<PadLabel>>,
) {
    let cap = mag_cap(&run);
    let mag = current_mag(&run);
    for (slot, children) in &labels {
        let line = match slot {
            PadSlot::Fire => "Fire".to_string(),
            PadSlot::Switch => {
                if run.has_bow {
                    if run.using_bow {
                        "Pistol".into()
                    } else {
                        "Bow".into()
                    }
                } else {
                    "Switch".into()
                }
            }
            PadSlot::ReloadDash => {
                if run.reloading > 0.0 {
                    "Reload…".into()
                } else if mag >= cap {
                    "Dash".into()
                } else {
                    format!("Reload ({mag}/{cap})")
                }
            }
        };
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                if text.0 != line {
                    text.0 = line.clone();
                }
            }
        }
    }
}

/// Lives / depth / mag / cleared for the HUD.
pub fn run_status(run: &Run) -> (u32, u32, u32, bool) {
    (run.lives, run.depth, current_mag(run), run.cleared)
}

/// Inventory line for the header: each gun, ammo, and which one is equipped.
pub fn loadout_line(run: &Run) -> String {
    let mark = |equipped: bool| if equipped { ">" } else { " " };
    let pistol = if run.reloading > 0.0 && !run.using_bow {
        format!("{}Pistol reload…", mark(!run.using_bow))
    } else {
        format!("{}Pistol {}/{}∞", mark(!run.using_bow), run.mag, MAG_SIZE)
    };
    if !run.has_bow {
        return pistol;
    }
    let bow = if run.reloading > 0.0 && run.using_bow {
        format!("{}Bow reload…", mark(run.using_bow))
    } else {
        format!(
            "{}Bow {}/{}+{}",
            mark(run.using_bow),
            run.bow_mag,
            BOW_MAG,
            run.bow_reserve
        )
    };
    format!("{pistol}   {bow}")
}
