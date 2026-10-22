//! Sofreh's Bevy side (step 7 of the plan): the kitchen on the screen, two local cooks, the clock and
//! the dishes in the header, and the result when the bell goes. The rules are in `rules.rs`; the
//! kitchen steps on the fixed clock inside the engine's `Play` set.
//!
//! Cook 1 is WASD and E; cook 2 is the arrows and Enter (`cook_wasd`, `cook_arrows` in the project's
//! input map). A cook steps one cell every [`STEP_TICKS`] while a direction is held; a press of the
//! interact key acts on the cell it faces.

use bevy::prelude::*;
use xerxes_engine::modules::flow::{Flow, Play};
use xerxes_engine::modules::input::Actions;

use super::rules::{COOK_TICKS, Dir, Held, Input, Kitchen};

/// The kitchen's cell size on the screen (pixels).
pub const CELL: f32 = 48.0;
/// Ticks a cook waits between two steps while a direction is held (an eighth of a second).
const STEP_TICKS: u32 = 8;
/// The match's time limit in ticks (4 minutes, from the GDD).
pub const TIME_LIMIT: u64 = 4 * 60 * 60;

const TEAM: [Color; 2] = [
    Color::srgb(0.878, 0.659, 0.243),
    Color::srgb(0.30, 0.66, 0.92),
];

/// The kitchen, held while the level is on.
#[derive(Resource)]
pub struct Table(pub Kitchen);

/// The interact presses since the last fixed tick, one per cook.
#[derive(Resource, Default)]
struct Latch {
    acts: [bool; 2],
}

/// Ticks each cook still waits before its next step.
#[derive(Resource, Default)]
struct Walk {
    timers: [u32; 2],
}

/// The kitchen's root in the scene: its cells and cooks hang from it.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SofrehBoard;

/// A cook, by index (0 is cook 1, 1 is cook 2).
#[derive(Component, Debug, Clone, Copy)]
pub struct SofrehCook(pub usize);

/// The grill's progress bar, scaled by how far the kebab has cooked.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SofrehGrill;

/// The status line: the dishes of each team and the time left.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SofrehStatus;

/// The sheet shown when the match is over.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SofrehBanner;

/// The line on that sheet: who won.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SofrehResult;

/// The screen position of a kitchen cell (the map's top row is the top of the screen).
pub fn cell_xy(x: usize, y: usize) -> Vec2 {
    Vec2::new((x as f32 - 4.0) * CELL, (2.0 - y as f32) * CELL)
}

pub struct SofrehPlugin;

impl Plugin for SofrehPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(Flow::Level), |mut commands: Commands| {
            commands.remove_resource::<Table>();
            commands.remove_resource::<Latch>();
            commands.remove_resource::<Walk>();
        })
        .add_systems(
            Update,
            (
                open.run_if(not(resource_exists::<Table>)),
                read_presses.run_if(resource_exists::<Table>),
                show_cooks.run_if(resource_exists::<Table>),
                show_grill.run_if(resource_exists::<Table>),
                show_status.run_if(resource_exists::<Table>),
            )
                .run_if(in_state(Flow::Level)),
        )
        .add_systems(
            FixedUpdate,
            step_kitchen
                .in_set(Play)
                .run_if(resource_exists::<Table>)
                .run_if(resource_exists::<Latch>),
        );
    }
}

/// Starts the kitchen on the first frame of a level that has its board.
fn open(mut commands: Commands, boards: Query<(), With<SofrehBoard>>) {
    if boards.is_empty() {
        return;
    }
    commands.insert_resource(Table(Kitchen::new(TIME_LIMIT)));
    commands.insert_resource(Latch::default());
    commands.insert_resource(Walk::default());
}

/// Latches each cook's interact press: a press is taken by one fixed tick.
fn read_presses(actions: Res<Actions>, mut latch: ResMut<Latch>) {
    latch.acts[0] |= actions.just_pressed("cook_wasd_act");
    latch.acts[1] |= actions.just_pressed("cook_arrows_act");
}

/// One fixed tick: each cook's input (an interact press first, else the held direction, one step
/// every [`STEP_TICKS`]), then the kitchen steps.
fn step_kitchen(
    mut table: ResMut<Table>,
    mut latch: ResMut<Latch>,
    mut walk: ResMut<Walk>,
    actions: Res<Actions>,
) {
    let axes = [actions.vector("cook_wasd"), actions.vector("cook_arrows")];
    let mut inputs = [None, None];
    for i in 0..2 {
        if std::mem::take(&mut latch.acts[i]) {
            inputs[i] = Some(Input::Interact);
            continue;
        }
        if walk.timers[i] > 0 {
            walk.timers[i] -= 1;
            continue;
        }
        let v = axes[i].clamp_length_max(1.0);
        if v.length() > 0.5 {
            let dir = if v.x.abs() > v.y.abs() {
                if v.x > 0.0 { Dir::Right } else { Dir::Left }
            } else if v.y > 0.0 {
                Dir::Up
            } else {
                Dir::Down
            };
            inputs[i] = Some(Input::Move(dir));
            walk.timers[i] = STEP_TICKS;
        }
    }
    table.0.step(&inputs);
}

/// Places each cook on its cell, and colours it by what it holds (its team's colour when empty).
fn show_cooks(table: Res<Table>, mut cooks: Query<(&SofrehCook, &mut Transform, &mut Sprite)>) {
    for (cook, mut transform, mut sprite) in &mut cooks {
        let c = table.0.cooks[cook.0];
        let at = cell_xy(c.x, c.y);
        transform.translation = Vec3::new(at.x, at.y, 2.0);
        sprite.color = match c.held {
            Held::Nothing => TEAM[c.team as usize],
            Held::Skewer => Color::srgb(0.55, 0.36, 0.20),
            Held::Cooked => Color::srgb(0.90, 0.40, 0.20),
            Held::Plated => Color::srgb(0.96, 0.96, 0.92),
        };
    }
}

/// Scales the grill's bar by how far its kebab has cooked (none when the grill is empty).
fn show_grill(table: Res<Table>, mut bars: Query<&mut Transform, With<SofrehGrill>>) {
    let progress = match table.0.grill {
        Some(ticks) => ticks as f32 / COOK_TICKS as f32,
        None => 0.0,
    };
    for mut transform in &mut bars {
        transform.scale.x = progress.clamp(0.0, 1.0);
    }
}

/// The status line: each team's dishes and the time left; once the bell has gone, who won.
fn show_status(
    table: Res<Table>,
    mut status: Query<&mut Text, (With<SofrehStatus>, Without<SofrehResult>)>,
    mut banner: Query<&mut Visibility, With<SofrehBanner>>,
    mut result: Query<&mut Text, (With<SofrehResult>, Without<SofrehStatus>)>,
) {
    let kitchen = &table.0;
    let left = kitchen.limit.saturating_sub(kitchen.ticks) / 60;
    let line = format!(
        "Team 1: {} dishes   Team 2: {} dishes   {} s left",
        kitchen.delivered[0], kitchen.delivered[1], left
    );
    for mut text in &mut status {
        **text = line.clone();
    }
    let over = kitchen.over();
    for mut visibility in &mut banner {
        *visibility = if over {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let verdict = match kitchen.winner() {
        Some(0) => "Team 1 wins the cook-off".to_string(),
        Some(_) => "Team 2 wins the cook-off".to_string(),
        None if kitchen.ticks >= kitchen.limit => "A tie: the next dish wins".to_string(),
        None => String::new(),
    };
    for mut text in &mut result {
        **text = verdict.clone();
    }
}
