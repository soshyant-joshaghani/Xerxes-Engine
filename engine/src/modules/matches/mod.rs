//! A level's match, in Bevy: the [`Match`] of `xerxes_sim` for the level's game mode, started
//! when the level starts and stepped on the fixed clock.
//!
//! - The game says what happens with [`MatchReport`] messages (points, goal progress, a player
//!   down, a side winning); the match decides the rest.
//! - [`MatchEvent`] messages carry what the match did (countdown, started, round ended, finished)
//!   for HUD and sound.
//! - [`ActiveMatch`] holds the match: read its phase, progress, time left and result.
//! - [`MatchPause`] stops the clock (a pause menu).
//! - A game that needs more than the mode's defaults (a goal of 8, a 60 tick countdown) sets
//!   [`RulesTweak`] before the level starts.
//!
//! Modes without a win condition (sandbox, progression, narrative) get a match that never ends.
//! A composite or tournament level has no single match, so none is started.

use bevy::prelude::*;

use crate::modules::flow::{ActiveGameMode, Flow};

pub use xerxes_sim::matches::{
    Event, Match, MatchResult, Outcome, Phase, Report, Rules, Side, SideDef, Standing,
};

/// The level's match.
#[derive(Resource, Debug, Clone)]
pub struct ActiveMatch(pub Match);

/// While true the match clock stands still.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MatchPause(pub bool);

/// Changes the mode's default rules for this game (goal, countdown, score target...).
#[derive(Resource, Clone)]
pub struct RulesTweak(pub fn(Rules) -> Rules);

/// Tells the match something happened.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchReport(pub Report);

/// Something the match did.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchEvent(pub Event);

/// Starts a fresh match for the level (also: remove [`ActiveMatch`] to restart it).
pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MatchPause>()
            .add_message::<MatchReport>()
            .add_message::<MatchEvent>()
            .add_systems(OnExit(Flow::Level), end_match)
            .add_systems(
                Update,
                (begin_match, apply_reports)
                    .chain()
                    .run_if(resource_exists::<ActiveGameMode>),
            )
            .add_systems(
                FixedUpdate,
                step_match.run_if(resource_exists::<ActiveMatch>),
            );
    }
}

fn end_match(mut commands: Commands) {
    commands.remove_resource::<ActiveMatch>();
}

fn begin_match(
    mut commands: Commands,
    mode: Res<ActiveGameMode>,
    active: Option<Res<ActiveMatch>>,
    tweak: Option<Res<RulesTweak>>,
    mut reports: ResMut<Messages<MatchReport>>,
) {
    if active.is_some() {
        return;
    }
    // Reports from before this match belong to the last one.
    reports.clear();
    let Ok(rules) = Rules::for_mode(mode.0.clone()) else {
        return; // A composite or tournament: its parts have matches, not the level.
    };
    let rules = match tweak {
        Some(tweak) => (tweak.0)(rules),
        None => rules,
    };
    let mut m = Match::new(rules);
    m.start();
    commands.insert_resource(ActiveMatch(m));
}

fn apply_reports(
    active: Option<ResMut<ActiveMatch>>,
    mut reports: MessageReader<MatchReport>,
) {
    let Some(mut active) = active else {
        return;
    };
    for MatchReport(report) in reports.read() {
        active.0.report(*report);
    }
}

fn step_match(
    mut active: ResMut<ActiveMatch>,
    pause: Res<MatchPause>,
    mut events: MessageWriter<MatchEvent>,
) {
    if !pause.0 {
        active.0.tick();
    }
    for event in active.0.take_events() {
        events.write(MatchEvent(event));
    }
}
