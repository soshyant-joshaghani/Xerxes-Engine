//! Zahhak's Bevy side. The field steps on the fixed clock inside the engine's `Play` set and before
//! the match steps (`MatchStep`), so each tick's reports count on that tick. You steer with the
//! `move` action; the bots steer with their brain (`Field::bot_heading`).
//!
//! Seats (`xerxes_sim::participants`): seat 0 is you (local), the others are bots. Your reports go
//! in with `submit`; the bots' reports are their brain's, handed to `step_with`.

use bevy::prelude::*;
use xerxes_engine::modules::flow::{CurrentLevel, Flow, LoadLevel, Play};
use xerxes_engine::modules::input::Actions;
use xerxes_engine::modules::matches::{
    ActiveMatch, MatchEvent, MatchPause, MatchStep, Phase, Report,
};
use xerxes_engine::modules::ui::TouchPad;
use xerxes_sim::participants::{Kind, Participants};

use super::rules::{Event, FIELD_RADIUS, Field};

/// Seed for the pellets and the field's layout (the same every run of a level).
const SEED: u64 = 0x5A_44_4B;
/// The size of a pellet and of a serpent's head and body on the screen.
const PELLET_R: f32 = 3.0;
const HEAD_R: f32 = 6.0;
const BODY_R: f32 = 4.5;

const WALL: Color = Color::srgb(0.878, 0.659, 0.243);
const PELLET: Color = Color::srgb(0.6, 0.85, 0.45);
/// The local serpent is cyan; opponents use distinct hues around the colour wheel.
fn serpent_color(seat: usize, seats: usize) -> Color {
    if seat == 0 {
        Color::srgb(0.12, 0.9, 1.0)
    } else {
        let opponents = seats.saturating_sub(1).max(1);
        let hue = 15.0 + (seat - 1) as f32 * 360.0 / opponents as f32;
        Color::hsl(hue, 0.72, 0.58)
    }
}

/// The field, held while the level's match runs.
#[derive(Resource)]
pub struct Serpents(pub Field);

/// Who fills each seat of the match.
#[derive(Resource)]
pub struct Seats(pub Participants);

/// The arena of a Zahhak level, written in its scene. Every Zahhak system acts only while a level
/// has this marker: the match is project-wide, so other levels run matches too.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ZahhakField;

/// Run condition: this level is a Zahhak level.
pub fn on_zahhak_level(field: Query<(), With<ZahhakField>>) -> bool {
    !field.is_empty()
}

/// The track pad over the content area: a drag steers the serpent from where the finger landed.
/// It has no resting ring, so nothing sits in a corner of the screen.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ZahhakTrackPad;

/// The Restart button of the header: reloads the level and starts a fresh match.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ZahhakRestart;

/// The status line: your length and the time left.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ZahhakStatus;

/// The sheet shown when the match is finished.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ZahhakBanner;

/// The line on that sheet: who is longest.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ZahhakResult;

pub struct ZahhakPlugin;

impl Plugin for ZahhakPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(Flow::Level), |mut commands: Commands| {
            commands.remove_resource::<Serpents>();
            commands.remove_resource::<Seats>();
        })
        .add_systems(
            Update,
            (
                sync_field,
                respawn_serpents,
                restart,
                draw_field,
                refresh_hud,
            )
                .chain()
                .run_if(in_state(Flow::Level))
                .run_if(on_zahhak_level),
        )
        .add_systems(
            FixedUpdate,
            step_field
                .in_set(Play)
                .before(MatchStep)
                .run_if(on_zahhak_level)
                .run_if(resource_exists::<ActiveMatch>)
                .run_if(resource_exists::<Serpents>)
                .run_if(resource_exists::<Seats>),
        );
    }
}

/// Builds the field and the seats when a match starts, and drops them when it is gone (a restart).
fn sync_field(
    mut commands: Commands,
    active: Option<Res<ActiveMatch>>,
    field: Option<Res<Serpents>>,
) {
    match (active, field) {
        (Some(active), None) => {
            let rules = active.0.rules();
            let seats = rules.sides.len();
            commands.insert_resource(Serpents(Field::new(seats, SEED)));
            // One person at seat 0; every other seat is a bot.
            commands.insert_resource(Seats(Participants::new(rules, 1, 0, SEED)));
        }
        (None, Some(_)) => {
            commands.remove_resource::<Serpents>();
            commands.remove_resource::<Seats>();
        }
        _ => {}
    }
}

/// The Restart button or the `respawn` key (R): reloads the level. The flow rebuilds the scene and
/// `sync_field` drops the old field once the match is gone.
fn restart(
    mut commands: Commands,
    actions: Res<Actions>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<ZahhakRestart>)>,
    level: Res<CurrentLevel>,
    mut load: MessageWriter<LoadLevel>,
) {
    let pressed = buttons.iter().any(|i| *i == Interaction::Pressed);
    if pressed || actions.just_pressed("respawn") {
        load.write(LoadLevel(level.0));
        commands.remove_resource::<ActiveMatch>();
    }
}

/// A serpent the match brings back is put back on the field (the match counts the respawn delay).
fn respawn_serpents(mut events: MessageReader<MatchEvent>, field: Option<ResMut<Serpents>>) {
    let Some(mut field) = field else {
        return;
    };
    for MatchEvent(event) in events.read() {
        if let xerxes_engine::modules::matches::Event::Respawned { side, .. } = event {
            field.0.respawn(*side);
        }
    }
}

/// One fixed tick: headings for every seat, the field's step, then the reports of what happened.
fn step_field(
    mut field: ResMut<Serpents>,
    mut seats: ResMut<Seats>,
    mut active: ResMut<ActiveMatch>,
    actions: Res<Actions>,
    pad: Query<&TouchPad, With<ZahhakTrackPad>>,
    pause: Res<MatchPause>,
) {
    if pause.0 || active.0.phase() != Phase::Playing {
        return;
    }
    // The keys and the stick (`move`) first; otherwise the track pad's drag direction, which keeps
    // the last heading while the finger rests.
    let keys = actions.vector("move").clamp_length_max(1.0);
    let drag = pad.iter().next().map_or(Vec2::ZERO, |p| p.direction);
    let steer = if keys.length() > 0.2 { keys } else { drag };
    // Boost is held by you only: the bots do not boost.
    let boost = actions.pressed("boost");
    for seat in 0..field.0.serpents.len() {
        let person = matches!(seats.0.seats()[seat].kind, Kind::Local);
        field.0.set_boost(seat, person && boost);
    }
    let n = field.0.serpents.len();
    let headings: Vec<f32> = (0..n)
        .map(|seat| match seats.0.seats()[seat].kind {
            Kind::Bot(_) => field.0.bot_heading(seat),
            Kind::Local if steer.length() > 0.2 => steer.y.atan2(steer.x),
            _ => field.0.serpent(seat).heading,
        })
        .collect();

    let mut bot_reports: Vec<Vec<Report>> = vec![Vec::new(); n];
    for event in field.0.step(&headings) {
        let (seat, reports) = match event {
            Event::Ate { seat } => (
                seat,
                vec![Report::Score {
                    side: seat,
                    amount: 1,
                }],
            ),
            Event::Shed { seat } => (
                seat,
                vec![Report::Score {
                    side: seat,
                    amount: -1,
                }],
            ),
            Event::Died { seat, score } => {
                let mut reports = Vec::new();
                if score != 0 {
                    reports.push(Report::Score {
                        side: seat,
                        amount: score,
                    });
                }
                reports.push(Report::Down {
                    side: seat,
                    player: 0,
                });
                (seat, reports)
            }
        };
        if matches!(seats.0.seats()[seat].kind, Kind::Bot(_)) {
            bot_reports[seat].extend(reports);
        } else {
            for report in reports {
                seats.0.submit(seat, report);
            }
        }
    }

    // The people's reports are queued by now; the bots' go in after them, in seat order.
    seats.0.step_with(&mut active.0, |seat, _, out| {
        out.append(&mut bot_reports[seat]);
    });
}

/// The field: the wall, the pellets, and each live serpent as a line of body with a head.
fn draw_field(mut gizmos: Gizmos, field: Option<Res<Serpents>>) {
    let Some(field) = field else {
        return;
    };
    gizmos.circle_2d(Vec2::ZERO, FIELD_RADIUS, WALL);
    for p in &field.0.pellets {
        gizmos.circle_2d(Vec2::new(p.x, p.y), PELLET_R, PELLET);
    }
    for (seat, serpent) in field.0.serpents.iter().enumerate() {
        if !serpent.alive {
            continue;
        }
        let color = serpent_color(seat, field.0.serpents.len());
        // A dot every few samples is the body; the head is drawn larger on top. The local worm
        // gets a white outline as well as its cyan color, so it stays identifiable in a crowd.
        for p in serpent.path.iter().step_by(3) {
            if seat == 0 {
                gizmos.circle_2d(Vec2::new(p.x, p.y), BODY_R + 2.5, Color::WHITE);
            }
            gizmos.circle_2d(Vec2::new(p.x, p.y), BODY_R, color);
        }
        let head = serpent.head();
        if seat == 0 {
            gizmos.circle_2d(Vec2::new(head.x, head.y), HEAD_R + 4.0, Color::WHITE);
        }
        gizmos.circle_2d(Vec2::new(head.x, head.y), HEAD_R, color);
    }
}

/// The status line and, once the match is over, who is longest.
fn refresh_hud(
    active: Option<Res<ActiveMatch>>,
    field: Option<Res<Serpents>>,
    mut status: Query<&mut Text, (With<ZahhakStatus>, Without<ZahhakResult>)>,
    mut banner: Query<&mut Visibility, With<ZahhakBanner>>,
    mut result: Query<&mut Text, (With<ZahhakResult>, Without<ZahhakStatus>)>,
) {
    let (Some(active), Some(field)) = (active, field) else {
        return;
    };
    let units = field.0.serpent(0).units;
    let alive = field.0.serpent(0).alive;
    let left = active
        .0
        .time_left()
        .map(|t| format!("{:.0} s left", t as f32 / 60.0))
        .unwrap_or_default();
    for mut text in &mut status {
        let state = if alive { "" } else { "  |  Respawning..." };
        **text = format!("YOU{state}  |  Length {units}  |  {left}");
    }

    let finished = active.0.phase() == Phase::Results;
    for mut visibility in &mut banner {
        *visibility = if finished {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Some(board) = active.0.result() {
        let line = match board.winners.as_slice() {
            [0] => format!("You are the longest: {units} units"),
            [winner] => format!("Serpent {} is the longest", winner + 1),
            _ => "A tie for the longest".to_string(),
        };
        for mut text in &mut result {
            **text = line.clone();
        }
    }
}
