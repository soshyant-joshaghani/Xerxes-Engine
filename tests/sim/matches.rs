//! The match runtime: phases, and every cell of the taxonomy played to its result.

use xerxes_sim::matches::{Event, Match, Outcome, Phase, Report, Rules, SideDef, TICKS_PER_SECOND};
use xerxes_sim::mode::{GameMode, ModeKind, Modifier, Sides, WinCondition};

const SEC: u64 = TICKS_PER_SECOND as u64;

fn playing(mode: GameMode) -> Match {
    let rules = Rules::for_mode(mode).unwrap().countdown(0);
    start(Match::new(rules))
}

fn start(mut m: Match) -> Match {
    m.start();
    assert_eq!(m.phase(), Phase::Playing);
    m
}

fn run(m: &mut Match, ticks: u64) {
    for _ in 0..ticks {
        m.tick();
    }
}

fn score(side: usize, amount: i64) -> Report {
    Report::Score { side, amount }
}

#[test]
fn phases_go_lobby_countdown_playing() {
    let rules = Rules::for_mode(GameMode::grid(Sides::Solo, WinCondition::Score)).unwrap();
    let mut m = Match::new(rules);
    assert_eq!(m.phase(), Phase::Lobby);
    // Nothing counts before play.
    assert!(!m.report(score(0, 5)));
    m.tick();
    assert_eq!(m.phase(), Phase::Lobby);
    m.start();
    assert!(matches!(m.phase(), Phase::Countdown(_)));
    assert!(!m.report(score(0, 5)));
    run(&mut m, 3 * SEC);
    assert_eq!(m.phase(), Phase::Playing);
    assert_eq!(
        m.take_events(),
        [Event::Countdown, Event::Started { round: 1 }]
    );
}

#[test]
fn solo_score_is_a_time_trial_that_finishes_with_its_score() {
    let mut m = playing(
        GameMode::grid(Sides::Solo, WinCondition::Score)
            .with(Modifier::TimeLimit { seconds: 30.0 }),
    );
    m.report(score(0, 120));
    m.report(score(0, 30));
    assert_eq!(m.time_left(), Some(30 * SEC));
    run(&mut m, 29 * SEC);
    assert_eq!(m.phase(), Phase::Playing);
    run(&mut m, SEC);
    assert_eq!(m.phase(), Phase::Results);
    let r = m.result().unwrap();
    assert_eq!(r.standings[0].score, 150);
    assert_eq!(r.standings[0].outcome, Outcome::Finished);
    assert_eq!(r.ticks, 30 * SEC);
}

#[test]
fn solo_objective_passes_at_the_goal_and_fails_at_the_time_limit() {
    let mode = GameMode::grid(Sides::Solo, WinCondition::Objective)
        .with(Modifier::TimeLimit { seconds: 10.0 });
    let rules = Rules::for_mode(mode).unwrap().countdown(0).goal(3);

    let mut m = start(Match::new(rules.clone()));
    for _ in 0..3 {
        m.report(Report::Progress { side: 0, amount: 1 });
    }
    assert_eq!(m.result().unwrap().standings[0].outcome, Outcome::Won);

    let mut m = start(Match::new(rules));
    m.report(Report::Progress { side: 0, amount: 2 });
    run(&mut m, 10 * SEC);
    let r = m.result().unwrap();
    assert_eq!(r.standings[0].outcome, Outcome::Lost);
    assert!(r.winners.is_empty());
}

#[test]
fn solo_survival_ends_when_down_and_is_won_by_lasting_the_limit() {
    let mut m = playing(GameMode::grid(Sides::Solo, WinCondition::Survival));
    run(&mut m, 5 * SEC);
    m.report(Report::Down { side: 0, player: 0 });
    let r = m.result().unwrap();
    assert_eq!(r.standings[0].outcome, Outcome::Finished);
    assert_eq!(
        r.standings[0].survived,
        5 * SEC,
        "the run is ranked by time survived"
    );

    let mode = GameMode::grid(Sides::Solo, WinCondition::Survival)
        .with(Modifier::TimeLimit { seconds: 20.0 });
    let mut m = playing(mode);
    run(&mut m, 20 * SEC);
    assert_eq!(m.result().unwrap().standings[0].outcome, Outcome::Won);
}

#[test]
fn coop_shares_one_fate() {
    let rules = Rules::for_mode(GameMode::grid(Sides::Coop, WinCondition::Survival))
        .unwrap()
        .countdown(0);
    assert_eq!(rules.sides[0].players, 2);
    let mut m = start(Match::new(rules));
    m.report(Report::Down { side: 0, player: 0 });
    assert_eq!(m.phase(), Phase::Playing, "one is still up");
    run(&mut m, 2 * SEC);
    m.report(Report::Down { side: 0, player: 1 });
    assert_eq!(m.phase(), Phase::Results);
    assert_eq!(m.result().unwrap().standings[0].survived, 2 * SEC);
}

#[test]
fn duel_score_picks_the_higher_score_and_calls_a_tie_a_draw() {
    let mode = GameMode::grid(Sides::Duel, WinCondition::Score);
    let rules = Rules::for_mode(mode).unwrap().countdown(0).score_target(7);
    let mut m = start(Match::new(rules));
    for _ in 0..3 {
        m.report(score(1, 1));
    }
    for _ in 0..7 {
        m.report(score(0, 1));
    }
    let r = m.result().unwrap();
    assert_eq!(r.winners, [0]);
    assert_eq!(
        (r.standings[0].side, r.standings[0].outcome),
        (0, Outcome::Won)
    );
    assert_eq!(
        (
            r.standings[1].side,
            r.standings[1].rank,
            r.standings[1].outcome
        ),
        (1, 2, Outcome::Lost)
    );

    let timed =
        GameMode::grid(Sides::Duel, WinCondition::Score).with(Modifier::TimeLimit { seconds: 5.0 });
    let mut m = start(Match::new(Rules::for_mode(timed).unwrap().countdown(0)));
    m.report(score(0, 2));
    m.report(score(1, 2));
    run(&mut m, 5 * SEC);
    let r = m.result().unwrap();
    assert!(
        r.standings
            .iter()
            .all(|s| s.outcome == Outcome::Draw && s.rank == 1)
    );
}

#[test]
fn duel_objective_goes_to_the_first_to_finish_and_respawns_bring_players_back() {
    let mode = GameMode::grid(Sides::Duel, WinCondition::Objective)
        .with(Modifier::Respawn { seconds: 3.0 })
        .with(Modifier::Escort);
    let rules = Rules::for_mode(mode).unwrap().countdown(0).goal(100);
    assert!(rules.escort() && !rules.fog_of_war());
    let mut m = start(Match::new(rules));
    m.report(Report::Down { side: 0, player: 0 });
    assert!(m.is_down(0, 0));
    assert!(m.is_standing(0), "it will respawn");
    run(&mut m, 3 * SEC);
    assert!(!m.is_down(0, 0));
    assert!(
        m.take_events()
            .contains(&Event::Respawned { side: 0, player: 0 })
    );
    m.report(Report::Progress {
        side: 1,
        amount: 100,
    });
    assert_eq!(m.result().unwrap().winners, [1]);
}

#[test]
fn duel_role_gives_the_defender_the_win_on_timeout() {
    let mode = GameMode::grid(Sides::Duel, WinCondition::Role)
        .asymmetric()
        .with(Modifier::TimeLimit { seconds: 60.0 });
    // Side 0 attacks (role 0), side 1 defends (role 1).
    let rules = Rules::new(mode, vec![SideDef::new(1), SideDef::new(1).role(1)])
        .unwrap()
        .countdown(0)
        .timeout_winner(1);
    let mut m = start(Match::new(rules.clone()));
    run(&mut m, 60 * SEC);
    assert_eq!(m.result().unwrap().winners, [1]);

    // The attacker winning outright ends it at once.
    let mut m = start(Match::new(rules));
    m.report(Report::Win { side: 0 });
    assert_eq!(m.result().unwrap().winners, [0]);
}

#[test]
fn duel_survival_is_best_of_three_rounds() {
    let mode = GameMode::grid(Sides::Duel, WinCondition::Survival);
    let rules = Rules::for_mode(mode)
        .unwrap()
        .countdown(0)
        .round_pause(10)
        .rounds_to_win(2);
    let mut m = start(Match::new(rules));
    // Round 1: side 1 falls out.
    m.report(Report::Down { side: 1, player: 0 });
    assert_eq!(m.phase(), Phase::RoundEnd(10));
    assert_eq!(m.round_wins(0), 1);
    run(&mut m, 10);
    assert_eq!((m.phase(), m.round()), (Phase::Playing, 2));
    assert!(!m.is_down(1, 0), "a new round puts everyone back");
    // Round 2: side 1 wins it.
    m.report(Report::Down { side: 0, player: 0 });
    run(&mut m, 10);
    // Round 3: side 0 takes the match.
    m.report(Report::Down { side: 1, player: 0 });
    let r = m.result().unwrap();
    assert_eq!(r.winners, [0]);
    assert_eq!(r.rounds, 3);
    assert_eq!(
        (r.standings[0].round_wins, r.standings[1].round_wins),
        (2, 1)
    );
}

#[test]
fn ngon_survival_ranks_by_who_lasted_and_ends_with_one_left() {
    let mode = GameMode::grid(Sides::NGon(4), WinCondition::Survival)
        .with(Modifier::ShrinkingZone)
        .with(Modifier::Permadeath);
    let mut m = playing(mode);
    assert!((m.zone() - 1.0).abs() < 1e-6);
    run(&mut m, 45 * SEC);
    assert!(
        (m.zone() - 0.6).abs() < 0.01,
        "half-way through 90 s: {}",
        m.zone()
    );
    m.report(Report::Down { side: 2, player: 0 });
    run(&mut m, SEC);
    m.report(Report::Down { side: 0, player: 0 });
    run(&mut m, SEC);
    m.report(Report::Down { side: 3, player: 0 });
    let r = m.result().unwrap();
    assert_eq!(r.winners, [1]);
    let order: Vec<_> = r.standings.iter().map(|s| s.side).collect();
    assert_eq!(order, [1, 3, 0, 2], "last standing first, first out last");
    let ranks: Vec<_> = r.standings.iter().map(|s| s.rank).collect();
    assert_eq!(ranks, [1, 2, 3, 4]);
}

#[test]
fn ngon_score_ranks_with_shared_places() {
    let mode = GameMode::grid(Sides::NGon(3), WinCondition::Score)
        .with(Modifier::TimeLimit { seconds: 1.0 })
        .with(Modifier::LowGravity);
    let rules = Rules::for_mode(mode).unwrap().countdown(0);
    assert!(rules.gravity_scale() < 1.0);
    let mut m = start(Match::new(rules));
    m.report(score(0, 10));
    m.report(score(1, 10));
    m.report(score(2, 4));
    run(&mut m, SEC);
    let r = m.result().unwrap();
    let ranks: Vec<_> = r.standings.iter().map(|s| s.rank).collect();
    assert_eq!(ranks, [1, 1, 3]);
    assert_eq!(r.standings[2].outcome, Outcome::Lost);
}

#[test]
fn modes_without_a_win_condition_never_end_and_wrappers_are_refused() {
    let mut m = playing(GameMode::sandbox());
    m.report(score(0, 1_000));
    run(&mut m, 600 * SEC);
    assert_eq!(m.phase(), Phase::Playing);
    assert!(m.result().is_none());

    let tournament = GameMode::new(ModeKind::Tournament(Box::new(ModeKind::Grid {
        sides: Sides::Duel,
        win: WinCondition::Survival,
    })));
    assert!(Rules::for_mode(tournament).is_err());
}

#[test]
fn rules_reject_the_wrong_number_of_sides() {
    let duel = GameMode::grid(Sides::Duel, WinCondition::Score);
    assert!(Rules::new(duel.clone(), vec![SideDef::new(1)]).is_err());
    assert!(Rules::new(duel, vec![SideDef::new(0), SideDef::new(1)]).is_err());
    assert!(Rules::for_mode(GameMode::grid(Sides::Solo, WinCondition::Role)).is_err());
}

#[test]
fn the_same_reports_give_the_same_result() {
    let play = || {
        let mode = GameMode::grid(Sides::Duel, WinCondition::Score)
            .with(Modifier::TimeLimit { seconds: 4.0 });
        let mut m = playing(mode);
        for i in 0..4 * SEC {
            if i % 50 == 0 {
                m.report(score((i / 50 % 2) as usize, 1));
            }
            m.tick();
        }
        m.result().unwrap().clone()
    };
    assert_eq!(play(), play());
}
