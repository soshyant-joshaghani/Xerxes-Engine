//! Participants: who plays each seat of a match, and how their reports reach it.
//!
//! A seat is one player of one side, or a spectator (who plays no seat). Each seat is one of
//! four kinds:
//!
//! - [`Kind::Local`]: a person on this device. Hot-seat is several local seats on one device,
//!   each submitting for its own player.
//! - [`Kind::Bot`]: a simple player as data (a rate of actions and a seed). It needs no game
//!   code: it reports for its own seat (a point, a step of progress, or a down, by the win
//!   condition), so every side can be filled offline.
//! - [`Kind::Remote`]: a person on another device. The wire submits its reports the way a local
//!   seat submits them.
//! - [`Kind::Spectator`]: watches, and reports nothing.
//!
//! One interface feeds every seat: [`Participants::submit`] takes a report for a seat that a
//! person controls, and [`Participants::step`] lets the bots act and applies what was submitted
//! to the [`Match`]. The match keeps the rules; the participants keep who may report.
//!
//! Everything is integer and seeded, so the same seats, seeds and submissions play the same match.

use crate::matches::{Match, Report, Rules, Side, TICKS_PER_SECOND};
use crate::mode::{ModeKind, WinCondition};

/// One minute in ticks.
const MINUTE: u64 = 60 * TICKS_PER_SECOND as u64;

/// Who plays a seat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Local,
    Remote,
    Spectator,
    Bot(Bot),
}

/// A simple bot, as data: it acts `actions_per_minute` times a minute on average, drawing from a
/// seeded generator. The same seed plays the same match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bot {
    state: u64,
    actions_per_minute: u32,
}

impl Bot {
    /// A bot that acts `actions_per_minute` times a minute (at most 60 a second).
    pub fn new(seed: u64, actions_per_minute: u32) -> Self {
        Self {
            state: seed,
            actions_per_minute: actions_per_minute.min(MINUTE as u32),
        }
    }

    /// Advances the generator one tick and says whether the bot acts on this tick.
    fn roll(&mut self) -> bool {
        // splitmix64: a small, well-mixed generator with no platform dependence.
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        z % MINUTE < u64::from(self.actions_per_minute)
    }
}

/// One seat of the match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seat {
    /// The side the seat plays for; `None` for a spectator.
    pub side: Option<Side>,
    /// The player on that side (0 for a spectator).
    pub player: u8,
    pub kind: Kind,
}

/// Every seat of a match and what fills it.
#[derive(Debug, Clone, PartialEq)]
pub struct Participants {
    seats: Vec<Seat>,
    queued: Vec<Report>,
}

impl Participants {
    /// Seats for every player of every side of `rules`, in side order. The first `locals` seats
    /// are local (hot-seat when there are several); the rest are bots at `actions_per_minute`,
    /// each seeded from `seed` and its seat so no two bots play alike.
    pub fn new(rules: &Rules, locals: usize, actions_per_minute: u32, seed: u64) -> Self {
        let mut seats = Vec::new();
        for (side, def) in rules.sides.iter().enumerate() {
            for player in 0..def.players {
                let index = seats.len();
                let kind = if index < locals {
                    Kind::Local
                } else {
                    let bot_seed =
                        seed.wrapping_add((index as u64).wrapping_mul(0x2545_F491_4F6C_DD1D));
                    Kind::Bot(Bot::new(bot_seed, actions_per_minute))
                };
                seats.push(Seat {
                    side: Some(side),
                    player,
                    kind,
                });
            }
        }
        Self {
            seats,
            queued: Vec::new(),
        }
    }

    /// Every seat, in order (spectators last).
    pub fn seats(&self) -> &[Seat] {
        &self.seats
    }

    /// Changes what fills a seat (a remote player, a bot taken over by a person).
    pub fn set_kind(&mut self, seat: usize, kind: Kind) -> bool {
        match self.seats.get_mut(seat) {
            Some(s) if s.side.is_some() => {
                s.kind = kind;
                true
            }
            _ => false,
        }
    }

    /// Adds a spectator seat and returns its index. Spectators play no side.
    pub fn add_spectator(&mut self) -> usize {
        self.seats.push(Seat {
            side: None,
            player: 0,
            kind: Kind::Spectator,
        });
        self.seats.len() - 1
    }

    /// Takes a report from the person (or the network) on a seat. Only a local or remote seat
    /// may report, and only for its own side and player. Returns whether it was accepted; the
    /// match applies it at the next [`Participants::step`].
    pub fn submit(&mut self, seat: usize, report: Report) -> bool {
        let Some(s) = self.seats.get(seat) else {
            return false;
        };
        if !matches!(s.kind, Kind::Local | Kind::Remote) {
            return false;
        }
        let fits = match report {
            Report::Score { side, .. } | Report::Progress { side, .. } | Report::Win { side } => {
                Some(side) == s.side
            }
            Report::Down { side, player } => Some(side) == s.side && player == s.player,
        };
        if !fits {
            return false;
        }
        self.queued.push(report);
        true
    }

    /// One fixed tick of participation: every bot rolls and reports, then everything submitted
    /// since the last step is applied to `m` in seat order. Call it once per tick, before
    /// [`Match::tick`]. Returns how many reports the match took.
    pub fn step(&mut self, m: &mut Match) -> usize {
        let win = match m.rules().mode.kind {
            ModeKind::Grid { win, .. } => Some(win),
            _ => None,
        };
        self.step_with(m, |_, seat, out| {
            let (Some(side), Kind::Bot(bot)) = (seat.side, &mut seat.kind) else {
                return;
            };
            if bot.roll()
                && let Some(report) = bot_report(win, side, seat.player)
            {
                out.push(report);
            }
        })
    }

    /// Like [`Participants::step`], but the game decides what each bot reports on its tick (its
    /// own brain). `decide` runs for bot seats only, with the seat index, the seat and the list to
    /// push its reports into (none for a quiet tick). Reports apply in seat order.
    pub fn step_with(
        &mut self,
        m: &mut Match,
        mut decide: impl FnMut(usize, &mut Seat, &mut Vec<Report>),
    ) -> usize {
        let Self { seats, queued } = self;
        for (index, seat) in seats.iter_mut().enumerate() {
            if matches!(seat.kind, Kind::Bot(_)) {
                decide(index, seat, queued);
            }
        }
        queued.drain(..).filter(|r| m.report(*r)).count()
    }
}

/// What a bot does on its turn to act, for the win condition: a point, a step of progress, or
/// its own player going down. A mode with no win condition has no bot action.
fn bot_report(win: Option<WinCondition>, side: Side, player: u8) -> Option<Report> {
    match win? {
        WinCondition::Score => Some(Report::Score { side, amount: 1 }),
        WinCondition::Objective | WinCondition::Role => Some(Report::Progress { side, amount: 1 }),
        WinCondition::Survival => Some(Report::Down { side, player }),
    }
}
