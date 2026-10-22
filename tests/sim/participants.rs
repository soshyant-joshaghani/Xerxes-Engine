//! Participants: bots fill every side offline, hot-seat and remote seats report through one
//! interface, spectators play nothing, and a seeded match replays the same.

use xerxes_sim::matches::{Match, Outcome, Phase, Report, Rules, TICKS_PER_SECOND};
use xerxes_sim::mode::{GameMode, Modifier, Sides, WinCondition};
use xerxes_sim::participants::{Bot, Kind, Participants};

const SEC: u64 = TICKS_PER_SECOND as u64;

/// A timed match with no countdown, so it is playing from the first tick.
fn timed(sides: Sides, win: WinCondition, seconds: f32) -> Rules {
    let mode = GameMode::grid(sides, win).with(Modifier::TimeLimit { seconds });
    Rules::for_mode(mode).unwrap().countdown(0)
}

fn started(rules: Rules) -> Match {
    let mut m = Match::new(rules);
    m.start();
    assert_eq!(m.phase(), Phase::Playing);
    m
}

/// Runs the match, stepping the participants before each tick, until it finishes or the cap.
fn play(m: &mut Match, p: &mut Participants, cap_ticks: u64) -> u64 {
    for tick in 0..cap_ticks {
        if m.phase() == Phase::Results {
            return tick;
        }
        p.step(m);
        m.tick();
    }
    cap_ticks
}

fn modes() -> Vec<(&'static str, Sides)> {
    vec![
        ("Coop", Sides::Coop),
        ("Duel", Sides::Duel),
        ("N-gon", Sides::NGon(3)),
    ]
}

const WINS: [WinCondition; 4] = [
    WinCondition::Score,
    WinCondition::Objective,
    WinCondition::Role,
    WinCondition::Survival,
];

#[test]
fn bots_fill_every_side_offline() {
    // Coop, Duel and N-gon, every win condition: all bots, and each match ends by its rules.
    for (name, sides) in modes() {
        for win in WINS {
            let rules = timed(sides, win, 60.0);
            let mut m = started(rules.clone());
            let mut p = Participants::new(&rules, 0, 600, 7);
            let ticks = play(&mut m, &mut p, 61 * SEC);
            assert_eq!(
                m.phase(),
                Phase::Results,
                "{name} {win:?} did not finish in {ticks} ticks"
            );
            assert!(m.result().is_some(), "{name} {win:?} has no result");
        }
    }
}

#[test]
fn bots_on_every_side_score() {
    // Each side has its own bots, so each side scores; none is left out.
    for sides in [Sides::Duel, Sides::NGon(3)] {
        let rules = timed(sides, WinCondition::Score, 30.0);
        let mut m = started(rules.clone());
        let mut p = Participants::new(&rules, 0, 600, 3);
        play(&mut m, &mut p, 31 * SEC);
        for side in 0..rules.sides.len() {
            assert!(m.score(side) > 0, "side {side} of {sides:?} never scored");
        }
    }
    // Coop: one side of two bots scores.
    let rules = timed(Sides::Coop, WinCondition::Score, 30.0);
    let mut m = started(rules.clone());
    let mut p = Participants::new(&rules, 0, 600, 3);
    play(&mut m, &mut p, 31 * SEC);
    assert!(m.score(0) > 0);
}

#[test]
fn a_seeded_match_replays_the_same() {
    let rules = timed(Sides::NGon(3), WinCondition::Objective, 90.0);
    let run = || {
        let mut m = started(rules.clone());
        let mut p = Participants::new(&rules, 0, 300, 42);
        play(&mut m, &mut p, 91 * SEC);
        m.result().cloned()
    };
    let first = run();
    assert!(first.is_some());
    assert_eq!(first, run());
}

#[test]
fn bot_seeds_differ_per_seat() {
    let rules = timed(Sides::Coop, WinCondition::Score, 60.0);
    let p = Participants::new(&rules, 0, 60, 1);
    let bots: Vec<Bot> = p
        .seats()
        .iter()
        .filter_map(|s| match s.kind {
            Kind::Bot(b) => Some(b),
            _ => None,
        })
        .collect();
    assert_eq!(bots.len(), 2);
    assert_ne!(bots[0], bots[1], "two bots must not play alike");
}

#[test]
fn a_bot_with_no_actions_never_scores() {
    let rules = timed(Sides::Duel, WinCondition::Score, 20.0);
    let mut m = started(rules.clone());
    let mut p = Participants::new(&rules, 0, 0, 9);
    play(&mut m, &mut p, 21 * SEC);
    assert_eq!(m.score(0), 0);
    assert_eq!(m.score(1), 0);
}

#[test]
fn hot_seat_locals_submit_for_their_own_seat() {
    // Two people on one device share a Coop side: both seats are local, no bots.
    let rules = timed(Sides::Coop, WinCondition::Score, 60.0);
    let mut m = started(rules.clone());
    let mut p = Participants::new(&rules, 2, 600, 5);
    assert!(p.seats().iter().all(|s| s.kind == Kind::Local));

    assert!(p.submit(0, Report::Score { side: 0, amount: 3 }));
    assert!(p.submit(1, Report::Score { side: 0, amount: 4 }));
    // Nothing applies until the step.
    assert_eq!(m.score(0), 0);
    assert_eq!(p.step(&mut m), 2);
    assert_eq!(m.score(0), 7);
}

#[test]
fn submit_checks_the_seat() {
    let rules = timed(Sides::Duel, WinCondition::Survival, 60.0);
    let mut p = Participants::new(&rules, 1, 0, 1); // seat 0 local, seat 1 a bot
    let spectator = p.add_spectator();

    assert!(p.submit(0, Report::Down { side: 0, player: 0 }));
    assert!(
        !p.submit(0, Report::Down { side: 1, player: 0 }),
        "wrong side"
    );
    assert!(
        !p.submit(1, Report::Down { side: 1, player: 0 }),
        "a bot's seat takes no input"
    );
    assert!(
        !p.submit(spectator, Report::Score { side: 0, amount: 1 }),
        "a spectator plays nothing"
    );
    assert!(
        !p.submit(9, Report::Score { side: 0, amount: 1 }),
        "no such seat"
    );
}

#[test]
fn a_local_seat_cannot_down_its_neighbour() {
    let rules = timed(Sides::Coop, WinCondition::Survival, 60.0);
    let mut p = Participants::new(&rules, 2, 0, 1);
    assert!(p.submit(0, Report::Down { side: 0, player: 0 }));
    assert!(
        !p.submit(0, Report::Down { side: 0, player: 1 }),
        "another player's seat"
    );
}

#[test]
fn remote_seats_report_like_local_ones() {
    let rules = timed(Sides::Duel, WinCondition::Score, 60.0);
    let mut m = started(rules.clone());
    let mut p = Participants::new(&rules, 0, 0, 1);
    // Offline the seat is a bot; the network makes it remote, and its reports now count.
    assert!(!p.submit(1, Report::Score { side: 1, amount: 2 }));
    assert!(p.set_kind(1, Kind::Remote));
    assert!(p.submit(1, Report::Score { side: 1, amount: 2 }));
    p.step(&mut m);
    assert_eq!(m.score(1), 2);
}

#[test]
fn spectators_play_no_side() {
    let rules = timed(Sides::Duel, WinCondition::Score, 60.0);
    let mut p = Participants::new(&rules, 0, 600, 1);
    let before = p.seats().len();
    let spectator = p.add_spectator();
    assert_eq!(spectator, before);
    assert_eq!(p.seats()[spectator].side, None);
    assert_eq!(p.seats()[spectator].kind, Kind::Spectator);
    // The sides keep their players: the spectator adds no one to play.
    let playing = p.seats().iter().filter(|s| s.side.is_some()).count();
    assert_eq!(
        playing,
        rules
            .sides
            .iter()
            .map(|s| s.players as usize)
            .sum::<usize>()
    );
}

#[test]
fn a_person_can_take_over_a_bot_seat() {
    // Mid-match a bot seat is handed to a person: its reports count from then on.
    let rules = timed(Sides::Duel, WinCondition::Objective, 60.0);
    let mut m = started(rules.clone());
    let mut p = Participants::new(&rules, 0, 0, 1);
    assert!(p.set_kind(0, Kind::Local));
    assert!(p.submit(0, Report::Progress { side: 0, amount: 1 }));
    p.step(&mut m);
    assert_eq!(m.phase(), Phase::Results, "the goal of 1 is met");
    assert_eq!(m.result().unwrap().standings[0].outcome, Outcome::Won);
}

#[test]
fn a_spectator_seat_cannot_be_set_to_play() {
    let rules = timed(Sides::Duel, WinCondition::Score, 60.0);
    let mut p = Participants::new(&rules, 0, 0, 1);
    let spectator = p.add_spectator();
    assert!(
        !p.set_kind(spectator, Kind::Local),
        "a spectator has no side to play"
    );
}

#[test]
fn the_game_can_decide_what_a_bot_reports() {
    // A bot's own brain (the game's) decides its report; a seat it returns None for stays quiet.
    let rules = timed(Sides::Duel, WinCondition::Score, 60.0);
    let mut m = started(rules.clone());
    let mut p = Participants::new(&rules, 0, 0, 1);
    let applied = p.step_with(&mut m, |index, _, out| {
        if index == 1 {
            out.push(Report::Score { side: 1, amount: 5 });
        }
    });
    assert_eq!(applied, 1);
    assert_eq!(m.score(0), 0);
    assert_eq!(m.score(1), 5);
}
