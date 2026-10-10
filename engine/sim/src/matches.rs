//! The match runtime: the rules every game mode plays by, with no Bevy and no clock.
//!
//! A [`Match`] is a set of sides, each with players, running a [`GameMode`] (the Sides x Win
//! condition taxonomy plus modifiers). The game reports what happened ([`Report`]: points, goal
//! progress, a player going down, a side winning) and calls [`Match::tick`] once per fixed tick;
//! the match decides the phases (lobby, countdown, playing, round end, results), applies the
//! time limit, the respawn delay and the rounds, and ends with a [`MatchResult`]: who won, the
//! ranking and each side's [`Outcome`].
//!
//! It is plain data stepped by integers, so a client, the server and a replay that feed it the
//! same reports in the same ticks reach the same result.
//!
//! | Win condition | The round ends when | Winner |
//! |---|---|---|
//! | Score | the time limit passes, or a side reaches `score_target` | the highest score (a tie at the top is a draw) |
//! | Objective, Role | a side's progress reaches `goal`, or a side is reported as `Win` | that side; at the time limit `timeout_winner` (the defender), else nobody |
//! | Survival | one side or none is left standing (alone: when it is out); at the time limit | the side left, or every side still standing |
//!
//! Several rounds (`rounds_to_win` above 1, e.g. best of 3) repeat that; the match is won by the
//! side that wins that many rounds.

use crate::mode::{GameMode, ModeKind, Modifier, Sides, WinCondition};

/// The fixed rate the match counts in.
pub const TICKS_PER_SECOND: u32 = 60;

/// Seconds as ticks (at least one).
pub fn ticks(seconds: f32) -> u64 {
    ((seconds * TICKS_PER_SECOND as f32).round() as u64).max(1)
}

/// A side: an index into [`Rules::sides`].
pub type Side = usize;

/// One side of the match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SideDef {
    /// How many players stand on it (people and bots alike).
    pub players: u8,
    /// The game's own number for what this side is (attacker, defender, crew, impostor).
    pub role: u8,
}

impl SideDef {
    pub fn new(players: u8) -> Self {
        Self { players, role: 0 }
    }

    pub fn role(mut self, role: u8) -> Self {
        self.role = role;
        self
    }
}

/// What a match is played by.
#[derive(Debug, Clone, PartialEq)]
pub struct Rules {
    pub mode: GameMode,
    pub sides: Vec<SideDef>,
    /// Rounds a side must win to win the match (1: a single round; 2: best of 3).
    pub rounds_to_win: u32,
    /// Ticks of countdown between `start` and play.
    pub countdown: u32,
    /// Ticks between a round's end and the next round.
    pub round_pause: u32,
    /// Score: the first side to reach this ends the round.
    pub score_target: Option<i64>,
    /// Objective and Role: progress a side needs to complete the goal.
    pub goal: u32,
    /// Objective and Role: the side that wins when the time runs out (the defender).
    pub timeout_winner: Option<Side>,
}

impl Rules {
    /// Rules for `mode` with the sides it needs: one for Solo and Coop (Coop with two
    /// players), two for Duel, `n` for N-gon, one for the modes without sides.
    pub fn for_mode(mode: GameMode) -> Result<Self, String> {
        let sides = match &mode.kind {
            ModeKind::Grid { sides, .. } => match sides {
                Sides::Solo => vec![SideDef::new(1)],
                Sides::Coop => vec![SideDef::new(2)],
                Sides::Duel => vec![SideDef::new(1), SideDef::new(1)],
                Sides::NGon(n) => vec![SideDef::new(1); *n as usize],
            },
            _ => vec![SideDef::new(1)],
        };
        Self::new(mode, sides)
    }

    /// Rules with the sides you give. The count must fit the mode: one for Solo and Coop, two
    /// for Duel, three or more for N-gon.
    pub fn new(mode: GameMode, sides: Vec<SideDef>) -> Result<Self, String> {
        mode.validate()?;
        match &mode.kind {
            ModeKind::Composite(_) | ModeKind::Tournament(_) => {
                return Err("a composite or a tournament is not one match: play its parts".into());
            }
            ModeKind::Grid { sides: kind, .. } => {
                let fits = match kind {
                    Sides::Solo | Sides::Coop => sides.len() == 1,
                    Sides::Duel => sides.len() == 2,
                    Sides::NGon(n) => sides.len() == *n as usize,
                };
                if !fits {
                    return Err(format!("{kind:?} does not fit {} side(s)", sides.len()));
                }
                if matches!(kind, Sides::Solo) && sides[0].players != 1 {
                    return Err("Solo has exactly one player".into());
                }
            }
            _ => {}
        }
        if sides.is_empty() || sides.iter().any(|s| s.players == 0) {
            return Err("every side needs at least one player".into());
        }
        Ok(Self {
            mode,
            sides,
            rounds_to_win: 1,
            countdown: 3 * TICKS_PER_SECOND,
            round_pause: TICKS_PER_SECOND,
            score_target: None,
            goal: 1,
            timeout_winner: None,
        })
    }

    pub fn rounds_to_win(mut self, rounds: u32) -> Self {
        self.rounds_to_win = rounds.max(1);
        self
    }

    pub fn countdown(mut self, ticks: u32) -> Self {
        self.countdown = ticks;
        self
    }

    pub fn round_pause(mut self, ticks: u32) -> Self {
        self.round_pause = ticks;
        self
    }

    pub fn score_target(mut self, score: i64) -> Self {
        self.score_target = Some(score);
        self
    }

    pub fn goal(mut self, goal: u32) -> Self {
        self.goal = goal.max(1);
        self
    }

    pub fn timeout_winner(mut self, side: Side) -> Self {
        self.timeout_winner = Some(side);
        self
    }

    fn win(&self) -> Option<WinCondition> {
        match self.mode.kind {
            ModeKind::Grid { win, .. } => Some(win),
            _ => None,
        }
    }

    fn lone_side(&self) -> bool {
        self.sides.len() == 1
    }

    fn modifier<T>(&self, pick: impl Fn(&Modifier) -> Option<T>) -> Option<T> {
        self.mode.modifiers.iter().find_map(pick)
    }

    /// The round's time limit in ticks.
    pub fn time_limit(&self) -> Option<u64> {
        self.modifier(|m| match m {
            Modifier::TimeLimit { seconds } => Some(ticks(*seconds)),
            _ => None,
        })
    }

    /// Ticks a downed player waits before coming back; `None` when they stay down.
    pub fn respawn_delay(&self) -> Option<u64> {
        if self.has(|m| matches!(m, Modifier::Permadeath)) {
            return None;
        }
        self.modifier(|m| match m {
            Modifier::Respawn { seconds } => Some(ticks(*seconds)),
            _ => None,
        })
    }

    fn has(&self, test: impl Fn(&Modifier) -> bool) -> bool {
        self.mode.modifiers.iter().any(test)
    }

    /// Fog of war: players see only their side's part of the world.
    pub fn fog_of_war(&self) -> bool {
        self.has(|m| matches!(m, Modifier::FogOfWar))
    }

    pub fn escort(&self) -> bool {
        self.has(|m| matches!(m, Modifier::Escort))
    }

    pub fn moving_objective(&self) -> bool {
        self.has(|m| matches!(m, Modifier::MovingObjective))
    }

    /// What gravity is multiplied by (1, or less under LowGravity).
    pub fn gravity_scale(&self) -> f32 {
        if self.has(|m| matches!(m, Modifier::LowGravity)) {
            0.35
        } else {
            1.0
        }
    }
}

/// Where the match is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for `start`.
    Lobby,
    /// Counting down to play: ticks left.
    Countdown(u32),
    Playing,
    /// Between rounds: ticks left.
    RoundEnd(u32),
    Results,
}

/// Something the game tells the match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Report {
    /// Points for a side (Score; any mode may keep a score).
    Score { side: Side, amount: i64 },
    /// Progress toward the goal (Objective, Role).
    Progress { side: Side, amount: u32 },
    /// A side wins the round outright (a base destroyed, the bomb defused, the traitor voted out).
    Win { side: Side },
    /// A player is down (killed, fell out of the ring).
    Down { side: Side, player: u8 },
}

/// Something that happened, for the game to show and the server to send.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Countdown,
    Started {
        round: u32,
    },
    Scored {
        side: Side,
        total: i64,
    },
    Progressed {
        side: Side,
        total: u32,
    },
    Down {
        side: Side,
        player: u8,
    },
    Respawned {
        side: Side,
        player: u8,
    },
    /// Every player of the side is out of the round.
    Eliminated {
        side: Side,
    },
    /// The round is over; `winner` is the one side that won it (none: a draw or a failure).
    RoundEnded {
        round: u32,
        winner: Option<Side>,
    },
    Finished,
}

/// What the match meant for one side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Won,
    Lost,
    /// Tied for the lead (or the only ones left at the time limit).
    Draw,
    /// A solo or team run with no opponent: the score or the time is the result.
    Finished,
}

/// One side's line in the results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Standing {
    pub side: Side,
    /// 1 is best; equal results share a rank.
    pub rank: u32,
    pub outcome: Outcome,
    pub score: i64,
    pub progress: u32,
    pub round_wins: u32,
    /// Ticks the side lasted in the last round.
    pub survived: u64,
}

/// The end of a match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    /// By rank, then side.
    pub standings: Vec<Standing>,
    /// The sides that won (empty: nobody did).
    pub winners: Vec<Side>,
    /// Ticks played, all rounds.
    pub ticks: u64,
    pub rounds: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct PlayerState {
    down: bool,
    respawn_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
struct SideState {
    score: i64,
    progress: u32,
    round_wins: u32,
    players: Vec<PlayerState>,
    /// The tick the side was eliminated in this round.
    out_at: Option<u64>,
}

impl SideState {
    fn new(def: &SideDef, round_wins: u32) -> Self {
        Self {
            score: 0,
            progress: 0,
            round_wins,
            players: vec![
                PlayerState {
                    down: false,
                    respawn_at: None,
                };
                def.players as usize
            ],
            out_at: None,
        }
    }
}

/// A match in progress.
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    rules: Rules,
    phase: Phase,
    sides: Vec<SideState>,
    /// Ticks played in the current round.
    tick: u64,
    total: u64,
    round: u32,
    events: Vec<Event>,
    result: Option<MatchResult>,
}

impl Match {
    pub fn new(rules: Rules) -> Self {
        let sides = rules.sides.iter().map(|d| SideState::new(d, 0)).collect();
        Self {
            rules,
            phase: Phase::Lobby,
            sides,
            tick: 0,
            total: 0,
            round: 1,
            events: Vec::new(),
            result: None,
        }
    }

    pub fn rules(&self) -> &Rules {
        &self.rules
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// The round being played (from 1).
    pub fn round(&self) -> u32 {
        self.round
    }

    /// Ticks played in this round.
    pub fn elapsed(&self) -> u64 {
        self.tick
    }

    /// Ticks left before the time limit, if there is one.
    pub fn time_left(&self) -> Option<u64> {
        self.rules
            .time_limit()
            .map(|limit| limit.saturating_sub(self.tick))
    }

    pub fn score(&self, side: Side) -> i64 {
        self.sides[side].score
    }

    pub fn progress(&self, side: Side) -> u32 {
        self.sides[side].progress
    }

    pub fn round_wins(&self, side: Side) -> u32 {
        self.sides[side].round_wins
    }

    pub fn is_down(&self, side: Side, player: u8) -> bool {
        self.sides[side].players[player as usize].down
    }

    /// Whether the side still has a player in the round (up, or coming back).
    pub fn is_standing(&self, side: Side) -> bool {
        self.sides[side].out_at.is_none()
    }

    /// The size of a shrinking zone as a fraction of its start, 1 down to 0.2 over the time
    /// limit (or 90 seconds without one). Always 1 when the mode has no ShrinkingZone.
    pub fn zone(&self) -> f32 {
        if !self
            .rules
            .mode
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::ShrinkingZone))
        {
            return 1.0;
        }
        let span = self
            .rules
            .time_limit()
            .unwrap_or(90 * TICKS_PER_SECOND as u64) as f32;
        1.0 - 0.8 * (self.tick as f32 / span).min(1.0)
    }

    /// The result, once the phase is `Results`.
    pub fn result(&self) -> Option<&MatchResult> {
        self.result.as_ref()
    }

    /// What happened since the last call.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Leaves the lobby: the countdown starts (or play, with no countdown).
    pub fn start(&mut self) {
        if self.phase != Phase::Lobby {
            return;
        }
        self.begin_countdown_or_play();
    }

    fn begin_countdown_or_play(&mut self) {
        if self.rules.countdown > 0 {
            self.phase = Phase::Countdown(self.rules.countdown);
            self.events.push(Event::Countdown);
        } else {
            self.begin_play();
        }
    }

    fn begin_play(&mut self) {
        self.phase = Phase::Playing;
        self.tick = 0;
        self.events.push(Event::Started { round: self.round });
    }

    /// Takes what the game reports. Only counts while playing; returns whether it did.
    pub fn report(&mut self, report: Report) -> bool {
        if self.phase != Phase::Playing {
            return false;
        }
        let side = match report {
            Report::Score { side, .. }
            | Report::Progress { side, .. }
            | Report::Win { side }
            | Report::Down { side, .. } => side,
        };
        if side >= self.sides.len() {
            return false;
        }
        match report {
            Report::Score { amount, .. } => {
                let s = &mut self.sides[side];
                s.score += amount;
                self.events.push(Event::Scored {
                    side,
                    total: s.score,
                });
            }
            Report::Progress { amount, .. } => {
                let s = &mut self.sides[side];
                s.progress += amount;
                self.events.push(Event::Progressed {
                    side,
                    total: s.progress,
                });
            }
            Report::Win { .. } => {
                self.end_round(vec![side]);
                return true;
            }
            Report::Down { player, .. } => {
                let Some(p) = self.sides[side].players.get_mut(player as usize) else {
                    return false;
                };
                if p.down {
                    return false;
                }
                p.down = true;
                p.respawn_at = self.rules.respawn_delay().map(|d| self.tick + d);
                self.events.push(Event::Down { side, player });
                self.check_elimination(side);
            }
        }
        self.evaluate();
        true
    }

    fn check_elimination(&mut self, side: Side) {
        let s = &mut self.sides[side];
        if s.out_at.is_none() && s.players.iter().all(|p| p.down && p.respawn_at.is_none()) {
            s.out_at = Some(self.tick);
            self.events.push(Event::Eliminated { side });
        }
    }

    /// Advances one tick.
    pub fn tick(&mut self) {
        match self.phase {
            Phase::Lobby | Phase::Results => {}
            Phase::Countdown(left) => {
                if left <= 1 {
                    self.begin_play();
                } else {
                    self.phase = Phase::Countdown(left - 1);
                }
            }
            Phase::RoundEnd(left) => {
                if left <= 1 {
                    self.next_round();
                } else {
                    self.phase = Phase::RoundEnd(left - 1);
                }
            }
            Phase::Playing => {
                self.tick += 1;
                self.total += 1;
                self.respawn_due();
                self.evaluate();
            }
        }
    }

    fn respawn_due(&mut self) {
        for (side, s) in self.sides.iter_mut().enumerate() {
            for (player, p) in s.players.iter_mut().enumerate() {
                if p.respawn_at.is_some_and(|at| self.tick >= at) {
                    p.down = false;
                    p.respawn_at = None;
                    self.events.push(Event::Respawned {
                        side,
                        player: player as u8,
                    });
                }
            }
        }
    }

    /// Ends the round when the mode's end condition holds.
    fn evaluate(&mut self) {
        if self.phase != Phase::Playing {
            return;
        }
        let Some(win) = self.rules.win() else {
            return; // Sandbox, progression, narrative: no end.
        };
        let timed_out = self.time_left() == Some(0);
        match win {
            WinCondition::Score => {
                let reached = self
                    .rules
                    .score_target
                    .is_some_and(|t| self.sides.iter().any(|s| s.score >= t));
                if reached || timed_out {
                    let best = self.sides.iter().map(|s| s.score).max().unwrap_or(0);
                    let winners = (0..self.sides.len())
                        .filter(|&i| self.sides[i].score == best)
                        .collect();
                    self.end_round(winners);
                }
            }
            WinCondition::Objective | WinCondition::Role => {
                let done: Vec<Side> = (0..self.sides.len())
                    .filter(|&i| self.sides[i].progress >= self.rules.goal)
                    .collect();
                if !done.is_empty() {
                    self.end_round(done);
                } else if timed_out {
                    self.end_round(self.rules.timeout_winner.into_iter().collect());
                }
            }
            WinCondition::Survival => {
                let alive: Vec<Side> = (0..self.sides.len())
                    .filter(|&i| self.sides[i].out_at.is_none())
                    .collect();
                if self.rules.lone_side() {
                    if alive.is_empty() {
                        self.end_round(Vec::new());
                    } else if timed_out {
                        self.end_round(alive);
                    }
                } else if alive.len() <= 1 || timed_out {
                    self.end_round(alive);
                }
            }
        }
    }

    fn end_round(&mut self, winners: Vec<Side>) {
        if self.phase != Phase::Playing {
            return;
        }
        // One winner takes the round; a tie takes nobody's.
        let winner = (winners.len() == 1).then(|| winners[0]);
        if let Some(w) = winner {
            self.sides[w].round_wins += 1;
        }
        self.events.push(Event::RoundEnded {
            round: self.round,
            winner,
        });
        // Sides still standing have lasted to now.
        for s in &mut self.sides {
            if s.out_at.is_none() {
                s.out_at = Some(self.tick);
            }
        }
        let decided = self
            .sides
            .iter()
            .any(|s| s.round_wins >= self.rules.rounds_to_win);
        if decided || self.rules.rounds_to_win == 1 {
            self.finish(winners);
        } else {
            self.phase = Phase::RoundEnd(self.rules.round_pause.max(1));
        }
    }

    fn next_round(&mut self) {
        self.round += 1;
        let wins: Vec<u32> = self.sides.iter().map(|s| s.round_wins).collect();
        self.sides = self
            .rules
            .sides
            .iter()
            .zip(wins)
            .map(|(d, w)| SideState::new(d, w))
            .collect();
        self.begin_countdown_or_play();
    }

    fn finish(&mut self, round_winners: Vec<Side>) {
        let multi_round = self.rules.rounds_to_win > 1;
        let winners: Vec<Side> = if multi_round {
            (0..self.sides.len())
                .filter(|&i| self.sides[i].round_wins >= self.rules.rounds_to_win)
                .collect()
        } else {
            round_winners
        };
        // Order: rounds won, then what the win condition measures.
        let key = |s: &SideState| -> (u32, i64) {
            let measure = match self.rules.win() {
                Some(WinCondition::Score) => s.score,
                Some(WinCondition::Survival) => s.out_at.unwrap_or(0) as i64,
                _ => s.progress as i64,
            };
            (s.round_wins, measure)
        };
        let lone = self.rules.lone_side();
        let mut standings: Vec<Standing> = (0..self.sides.len())
            .map(|side| {
                let s = &self.sides[side];
                let outcome = if winners.contains(&side) {
                    if winners.len() > 1 && !lone {
                        Outcome::Draw
                    } else if lone && self.rules.win() == Some(WinCondition::Score) {
                        Outcome::Finished
                    } else {
                        Outcome::Won
                    }
                } else if lone && self.rules.win() == Some(WinCondition::Survival) {
                    Outcome::Finished
                } else {
                    Outcome::Lost
                };
                let rank = 1 + self.sides.iter().filter(|o| key(o) > key(s)).count() as u32;
                Standing {
                    side,
                    rank,
                    outcome,
                    score: s.score,
                    progress: s.progress,
                    round_wins: s.round_wins,
                    survived: s.out_at.unwrap_or(self.tick),
                }
            })
            .collect();
        standings.sort_by_key(|s| (s.rank, s.side));
        self.result = Some(MatchResult {
            standings,
            winners,
            ticks: self.total,
            rounds: self.round,
        });
        self.phase = Phase::Results;
        self.events.push(Event::Finished);
    }
}
