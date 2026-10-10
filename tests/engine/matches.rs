//! The match module in Bevy: a level's mode becomes a match, reports go in, events and the
//! result come out, pausing stops the clock, and removing the match restarts it.

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use xerxes_engine::modules::flow::{ActiveGameMode, Flow, GameMode, Modifier, Sides, WinCondition};
use xerxes_engine::modules::matches::{
    ActiveMatch, MatchEvent, MatchPause, MatchPlugin, MatchReport, Outcome, Phase, RulesTweak,
};
use xerxes_sim::matches::{Event, Report, Rules};

fn tweak(rules: Rules) -> Rules {
    rules.goal(3).countdown(30)
}

fn app(mode: GameMode) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Flow>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_micros(
            16_667,
        )))
        .insert_resource(ActiveGameMode(mode))
        .insert_resource(RulesTweak(tweak))
        .add_plugins(MatchPlugin);
    app.update();
    app
}

fn run(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

fn phase(app: &App) -> Phase {
    app.world().resource::<ActiveMatch>().0.phase()
}

fn report(app: &mut App, report: Report) {
    app.world_mut().write_message(MatchReport(report));
}

fn events(app: &mut App) -> Vec<Event> {
    let messages = app.world().resource::<Messages<MatchEvent>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).map(|e| e.0).collect()
}

#[test]
fn a_level_gets_a_match_that_counts_down_plays_and_ends_on_the_goal() {
    let mut app = app(GameMode::grid(Sides::Solo, WinCondition::Objective));
    run(&mut app, 2);
    assert!(
        matches!(phase(&app), Phase::Countdown(_)),
        "the tweak gave a 30 tick countdown: {:?}",
        phase(&app)
    );
    run(&mut app, 40);
    assert_eq!(phase(&app), Phase::Playing);

    for _ in 0..3 {
        report(&mut app, Report::Progress { side: 0, amount: 1 });
    }
    run(&mut app, 2);
    assert_eq!(phase(&app), Phase::Results);
    let active = app.world().resource::<ActiveMatch>();
    assert_eq!(
        active.0.result().unwrap().standings[0].outcome,
        Outcome::Won
    );
    assert!(events(&mut app).contains(&Event::Finished));
}

#[test]
fn pausing_stops_the_clock_and_a_removed_match_starts_again() {
    let mode = GameMode::grid(Sides::Solo, WinCondition::Score)
        .with(Modifier::TimeLimit { seconds: 100.0 });
    let mut app = app(mode);
    run(&mut app, 45);
    let before = app.world().resource::<ActiveMatch>().0.elapsed();
    assert!(before > 0);

    app.world_mut().resource_mut::<MatchPause>().0 = true;
    run(&mut app, 10);
    assert_eq!(app.world().resource::<ActiveMatch>().0.elapsed(), before);
    app.world_mut().resource_mut::<MatchPause>().0 = false;

    report(&mut app, Report::Score { side: 0, amount: 9 });
    run(&mut app, 2);
    assert_eq!(app.world().resource::<ActiveMatch>().0.score(0), 9);

    app.world_mut().remove_resource::<ActiveMatch>();
    run(&mut app, 2);
    let fresh = app.world().resource::<ActiveMatch>();
    assert_eq!(fresh.0.score(0), 0, "a new match, not the old one");
}

#[test]
fn a_composite_level_has_no_single_match() {
    use xerxes_sim::mode::ModeKind;
    let mode = GameMode::new(ModeKind::Composite(vec![ModeKind::Sandbox]));
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Flow>()
        .insert_resource(ActiveGameMode(mode))
        .add_plugins(MatchPlugin);
    // A composite is allowed (its parts are), but Rules refuses it, so no match starts.
    run(&mut app, 3);
    assert!(app.world().get_resource::<ActiveMatch>().is_none());
}
