//! Siavash's run, without Bevy: a knight on an endless road of three lanes, running by himself.
//! The road is made from a seed, as enemies laid out ahead of the knight. The rules follow Blades
//! of Brim's attack by direction:
//!
//! - a **goon** stands on the road in one lane. Running straight into it without a change of
//!   direction loses. It dies to a **lane switch into its lane** while it is close ahead (the
//!   exact moment), to a **roll** (down on the ground) in its lane, or to a **dive** (down while
//!   in the air) over it.
//! - a **flyer** hangs over one lane. A **jump** through its lane kills it; rolling under it passes.
//!
//! Jump, roll (down on the ground) and dive (down in the air) are the moves; lane switches are the
//! steering. The score is metres run and enemies killed.
//!
//! The run steps on a fixed clock ([`TICKS_PER_SECOND`]) and nothing in it reads a clock or a
//! random source of its own, so the same seed and the same [`Input`]s always give the same run.
//! That is what a ghost, a replay and a daily board need.
//!
//! Units are metres and seconds. The knight's `y` is his height above the road. Lanes are numbered
//! 0 (left) to 2 (right).

/// The fixed clock the Bevy side steps the run on.
pub const TICKS_PER_SECOND: u32 = 60;
/// Seconds per tick.
pub const DT: f32 = 1.0 / TICKS_PER_SECOND as f32;

/// The most run-time one frame of a game may add ([`Pace`]).
pub const MAX_CATCH_UP: f32 = 0.05;

/// Turns real time into fixed ticks: the one place that says how a frame becomes steps of the run.
///
/// A frame never contributes more than [`MAX_CATCH_UP`] seconds, however long it actually took.
/// The slowest frames there are are the first ones of a level, where the knight's model is still
/// being built into the world; without the cap those seconds would be spent running him through
/// road the player was never shown, into the first goon. Past the cap the run goes slower than the
/// wall clock instead of skipping ahead, which is the right way round for a game where a step
/// missed is a death.
///
/// The room host steps the same run from the same clock, so this lives beside the rules rather
/// than in the Bevy side.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pace {
    /// Time taken from the frames so far that has not become a tick yet.
    leftover: f32,
}

impl Pace {
    /// How many ticks this frame runs. The remainder is kept for the next one.
    pub fn ticks(&mut self, delta: f32) -> u32 {
        if !delta.is_finite() || delta <= 0.0 {
            return 0;
        }
        self.leftover += delta.min(MAX_CATCH_UP);
        let ticks = (self.leftover / DT) as u32;
        self.leftover -= ticks as f32 * DT;
        ticks
    }

    /// Drops what has not been spent: a run that is over, or starting again.
    pub fn reset(&mut self) {
        self.leftover = 0.0;
    }
}

/// Lanes on the road, and the distance between two lane centres.
pub const LANES: u8 = 3;
pub const LANE_SPACING: f32 = 2.0;

/// The special ability, the default one: the **Shield**. Each run starts with this many charges, and
/// each activation lasts this many ticks (10 s), or until it takes one collision: the shield kills
/// that enemy and ends, and the knight does not die. Pickups add charges and other abilities later.
pub const ABILITY_CHARGES: u32 = 1;
pub const ABILITY_TICKS: u32 = 10 * TICKS_PER_SECOND;

/// The knight's height standing and rolling.
pub const KNIGHT_STAND: f32 = 1.8;
pub const KNIGHT_ROLL: f32 = 0.9;
/// A goon's height on the road.
pub const GOON_HEIGHT: f32 = 1.0;
/// A flyer's underside, 1.28 times the first version's 1.2 m: a knight whose head is above it (and who has not attacked it) hits it.
pub const FLYER_BOTTOM: f32 = 1.536;
/// A flyer's height: its body runs from [`FLYER_BOTTOM`] up to this many metres above it.
pub const FLYER_HEIGHT: f32 = 0.6;

const GRAVITY: f32 = 30.0;
/// A jump 1.28 times the first version's: on the 60 Hz clock the first apex is 1.275 m and this
/// one is 1.632 m. The height grows with
/// the speed, so the speed is about 10.15 (not 9 × √1.28, which the clock's rounding spoils).
const JUMP_SPEED: f32 = 10.146;
/// Downward speed of a dive (down in the air).
const DIVE_SPEED: f32 = 30.0;
/// A jump pressed this many ticks before landing still counts (0.12 s).
const JUMP_BUFFER: u32 = 7;
/// A jump pressed this many ticks after leaving the road still counts (0.10 s).
const COYOTE: u32 = 6;
/// How long a roll lasts (0.5 s).
const ROLL_TICKS: u32 = 30;
/// How close ahead of a goon the knight must switch into its lane, or land from a dive, for that to
/// strike it when they meet. Every kill is decided when the bodies meet, by what the knight did
/// just before: a dive lands in the air's last moments, so its landing counts for a stretch ahead.
const SWITCH_REACH: f32 = 3.0;
const DIVE_REACH: f32 = 4.0;

/// Running speed at the start and at the cap (m/s), and how much faster the road gets per metre.
/// Speed depends on the distance, not on a clock, so the generator knows how fast the knight will
/// be at any enemy and can keep every one fair.
const SPEED_START: f32 = 14.0;
const SPEED_MAX: f32 = 26.0;
const SPEED_PER_M: f32 = 0.01;

/// The road is clear for this many metres at the start.
const START_CLEAR: f32 = 30.0;
/// How far ahead of the knight enemies are made.
const HORIZON: f32 = 60.0;
/// How far behind the knight enemies are kept (so the Bevy side can still draw them leaving).
const KEEP_BEHIND: f32 = 10.0;
/// The shortest gap between two rows of enemies, before the speed adds its reading time.
const GAP_MIN: f32 = 12.0;
/// Coins come in a row of this many, this far apart (metres).
pub const COIN_ROW: u32 = 3;
pub const COIN_SPACING: f32 = 2.0;
/// Ammo a pickup gives.
pub const AMMO_PICKUP: u32 = 5;
/// The most ability charges a power-up can stack to.
pub const ABILITY_MAX_CHARGES: u32 = 2;
/// A pickup is taken when the knight is lower than this (a jump above it misses).
pub const PICKUP_REACH: f32 = 1.6;
/// Kills this close (metres) after the last one extend the chain.
pub const COMBO_REACH: f32 = 25.0;
/// The most a kill is worth in a chain.
pub const COMBO_MAX: u32 = 5;
/// Coins per step of the multiplier, and the most steps.
const COINS_PER_STEP: u32 = 5;
const MULTIPLIER_STEPS: u32 = 3;

/// What a pickup gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    /// A coin: score, times the multiplier.
    Coin,
    /// Ammo for the gun.
    Ammo,
    /// An ability charge.
    PowerUp,
}

/// A pickup on the road, at a distance in a lane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pickup {
    pub at: f32,
    pub lane: u8,
    pub kind: PickupKind,
    /// Taken by the knight (it stays listed until he is past it, so it can be drawn gone).
    pub taken: bool,
}

impl Pickup {
    pub fn new(at: f32, lane: u8, kind: PickupKind) -> Self {
        Self {
            at,
            lane,
            kind,
            taken: false,
        }
    }
}

/// What the player asked for in one tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Input {
    /// -1 to switch left, 0 to stay, +1 to switch right.
    pub lane: i8,
    /// A jump pressed this tick.
    pub jump: bool,
    /// Down pressed this tick: a roll on the ground, a dive in the air.
    pub down: bool,
    /// The special ability pressed this tick (it takes a charge, when there is one and none is active).
    pub ability: bool,
}

/// One tick's input as a flag byte (the ghost's format): bits 0-1 the lane switch (0 none, 1 left,
/// 2 right), bit 2 jump, bit 3 down, bit 4 ability.
pub fn pack_input(input: Input) -> u8 {
    let lane = match input.lane {
        -1 => 1,
        1 => 2,
        _ => 0,
    };
    lane | u8::from(input.jump) << 2 | u8::from(input.down) << 3 | u8::from(input.ability) << 4
}

/// The input a flag byte stands for (see [`pack_input`]).
pub fn unpack_input(flags: u8) -> Input {
    Input {
        lane: match flags & 3 {
            1 => -1,
            2 => 1,
            _ => 0,
        },
        jump: flags & 4 != 0,
        down: flags & 8 != 0,
        ability: flags & 16 != 0,
    }
}

/// A run is a ghost-able sim: its input is one [`Input`] a tick.
impl xerxes_sim::ghost::Sim for Run {
    type Input = Input;

    fn step(&mut self, input: Input) {
        Run::step(self, input);
    }
}

/// The two kinds of enemy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoeKind {
    /// On the road, in one lane.
    Goon,
    /// In the air, over one lane.
    Flyer,
}

/// An enemy, at a distance along the road (in metres), in a lane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Foe {
    pub at: f32,
    pub lane: u8,
    pub kind: FoeKind,
    /// Killed by an attack (it stays listed until the knight is past it, so it can be drawn gone).
    pub dead: bool,
    /// A shielded goon: only a dive (a knight falling onto it) strikes it.
    pub shielded: bool,
}

impl Foe {
    /// A live enemy at distance `at` in `lane`.
    pub fn new(at: f32, lane: u8, kind: FoeKind) -> Self {
        Self {
            at,
            lane,
            kind,
            dead: false,
            shielded: false,
        }
    }
}

/// Whether the run is still going, and what ended it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Running,
    Crashed(FoeKind),
}

/// One run: the knight, the road ahead of him and the score.
#[derive(Clone, Debug)]
pub struct Run {
    pub phase: Phase,
    /// The lane the knight is in (0 to 2).
    pub lane: u8,
    /// The knight's height above the road.
    pub y: f32,
    vy: f32,
    jump_buffer: u32,
    coyote: u32,
    roll: u32,
    diving: bool,
    /// Metres run.
    pub dist: f32,
    /// Running speed in m/s.
    pub speed: f32,
    /// The enemies on the road, nearest first.
    pub foes: Vec<Foe>,
    /// The pickups on the road, nearest first.
    pub pickups: Vec<Pickup>,
    /// Enemies killed by attacks.
    pub kills: u32,
    /// Coins taken, and ammo.
    pub coins: u32,
    pub ammo: u32,
    /// Points from coins (each coin is worth the multiplier at the time it is taken).
    coin_points: u32,
    /// The kills in the current chain (a kill close to the last one extends it).
    pub combo: u32,
    /// Where (metres) the last kill was, for the chain.
    last_kill_at: Option<f32>,
    /// Points from kills: each is worth its place in the chain, up to [`COMBO_MAX`].
    combo_points: u32,
    next_at: f32,
    /// Random state (splitmix64), seeded from the run's seed.
    rng: u64,
    /// Ticks stepped so far.
    pub tick: u32,
    /// Distance (metres run) at the knight's last lane switch, for a goon met close after it.
    switch_at: Option<f32>,
    /// Distance (metres run) where the knight last landed from a dive, for a goon met close after it.
    dive_landed_at: Option<f32>,
    /// Ability charges left in this run.
    pub ability_charges: u32,
    /// Ticks the ability stays on after its activation (0 when it is off).
    ability_ticks: u32,
}

impl Run {
    /// A fresh run on the road that `seed` makes.
    pub fn new(seed: u64) -> Self {
        let mut run = Self::blank(seed ^ 0x9E37_79B9_7F4A_7C15, START_CLEAR);
        run.generate();
        run
    }

    /// A run on exactly these enemies and no others (for tests and fixed courses).
    pub fn scripted(mut foes: Vec<Foe>) -> Self {
        foes.sort_by(|a, b| a.at.total_cmp(&b.at));
        let mut run = Self::blank(1, f32::INFINITY);
        run.foes = foes;
        run
    }

    fn blank(rng: u64, next_at: f32) -> Self {
        Self {
            phase: Phase::Running,
            lane: LANES / 2,
            y: 0.0,
            vy: 0.0,
            jump_buffer: 0,
            coyote: 0,
            roll: 0,
            diving: false,
            dist: 0.0,
            speed: SPEED_START,
            foes: Vec::new(),
            pickups: Vec::new(),
            kills: 0,
            coins: 0,
            ammo: 0,
            coin_points: 0,
            combo: 0,
            last_kill_at: None,
            combo_points: 0,
            next_at,
            rng,
            tick: 0,
            switch_at: None,
            dive_landed_at: None,
            ability_charges: ABILITY_CHARGES,
            ability_ticks: 0,
        }
    }

    /// Whether the special ability is on now.
    pub fn ability_active(&self) -> bool {
        self.ability_ticks > 0
    }

    /// Metres run, as the score shows them.
    pub fn metres(&self) -> u32 {
        self.dist as u32
    }

    /// The score: metres run, the points of the coins, and the points of the kills (their chain).
    pub fn score(&self) -> u32 {
        self.metres() + self.coin_points + self.combo_points
    }

    /// The multiplier for coins: 1, up one step for every five coins taken, to a fixed most.
    pub fn multiplier(&self) -> u32 {
        1 + (self.coins / COINS_PER_STEP).min(MULTIPLIER_STEPS)
    }

    /// Whether the knight is rolling (his head is lower).
    pub fn rolling(&self) -> bool {
        self.roll > 0
    }

    /// The knight's height now (standing or rolling).
    pub fn height(&self) -> f32 {
        if self.rolling() {
            KNIGHT_ROLL
        } else {
            KNIGHT_STAND
        }
    }

    /// The x position of a lane's centre, with the middle lane at 0.
    pub fn lane_x(lane: u8) -> f32 {
        (lane as f32 - (LANES / 2) as f32) * LANE_SPACING
    }

    /// Advances the run by one tick with this input. A crashed run does not move.
    pub fn step(&mut self, input: Input) {
        if matches!(self.phase, Phase::Crashed(_)) {
            return;
        }
        self.tick += 1;

        // The ability is on for exactly ABILITY_TICKS steps after it is taken.
        self.ability_ticks = self.ability_ticks.saturating_sub(1);
        if input.ability && self.ability_charges > 0 && !self.ability_active() {
            self.ability_charges -= 1;
            self.ability_ticks = ABILITY_TICKS;
        }

        let before = self.lane;
        if input.lane < 0 && self.lane > 0 {
            self.lane -= 1;
        } else if input.lane > 0 && self.lane < LANES - 1 {
            self.lane += 1;
        }
        let switched = self.lane != before;
        if switched {
            self.switch_at = Some(self.dist);
        }

        self.jump_buffer = if input.jump {
            JUMP_BUFFER
        } else {
            self.jump_buffer.saturating_sub(1)
        };
        let grounded = self.y <= 0.0;
        self.coyote = if grounded {
            COYOTE
        } else {
            self.coyote.saturating_sub(1)
        };
        if input.down {
            if grounded {
                if self.roll == 0 {
                    self.roll = ROLL_TICKS;
                }
            } else {
                self.diving = true;
                self.vy = -DIVE_SPEED;
            }
        }
        if self.jump_buffer > 0 && (grounded || self.coyote > 0) {
            self.vy = JUMP_SPEED;
            self.jump_buffer = 0;
            self.coyote = 0;
            self.roll = 0;
            self.diving = false;
        }
        self.roll = self.roll.saturating_sub(1);

        self.vy -= GRAVITY * DT;
        self.y += self.vy * DT;
        if self.y <= 0.0 {
            if self.diving {
                self.dive_landed_at = Some(self.dist);
            }
            self.y = 0.0;
            self.vy = 0.0;
            self.diving = false;
        }

        let prev = self.dist;
        self.dist += self.speed * DT;
        self.speed = speed_at(self.dist);
        self.generate();

        self.collect(prev);

        if matches!(self.phase, Phase::Running) {
            self.resolve_hits(prev);
        }

        let behind = self.dist - KEEP_BEHIND;
        self.foes.retain(|f| f.at >= behind);
        self.pickups.retain(|p| p.at >= behind);
    }

    /// Takes the pickups the knight passes in his lane while he is low enough to reach them.
    fn collect(&mut self, prev: f32) {
        let (lane, y, dist) = (self.lane, self.y, self.dist);
        if y >= PICKUP_REACH {
            return;
        }
        let mut taken = Vec::new();
        for p in self.pickups.iter_mut() {
            if !p.taken && p.lane == lane && p.at > prev && p.at <= dist {
                p.taken = true;
                taken.push(p.kind);
            }
        }
        for kind in taken {
            match kind {
                PickupKind::Coin => {
                    self.coin_points += self.multiplier();
                    self.coins += 1;
                }
                PickupKind::Ammo => self.ammo += AMMO_PICKUP,
                PickupKind::PowerUp => {
                    self.ability_charges = (self.ability_charges + 1).min(ABILITY_MAX_CHARGES);
                }
            }
        }
    }

    /// Settles the enemies the knight meets this tick, in his lane, when the bodies meet. What the
    /// knight is doing at that moment decides it. A goon stands from the road up to [`GOON_HEIGHT`]:
    /// a knight whose feet are below its top meets it. He strikes it with a roll on the ground, a
    /// dive still falling, or a lane switch into its lane made close ahead of it (see
    /// [`SWITCH_REACH`]); otherwise the run ends. A jump above it passes. A flyer hangs from
    /// [`FLYER_BOTTOM`] up to [`FLYER_BOTTOM`] + [`FLYER_HEIGHT`]: a knight in the air strikes it;
    /// on the road he meets it standing and the run ends; a rolling one passes under.
    /// The shield takes a collision it does not win: the enemy dies and the shield ends.
    fn resolve_hits(&mut self, prev: f32) {
        let dist = self.dist;
        let (lane, y) = (self.lane, self.y);
        let top = y + self.height();
        let airborne = y > 0.0;
        let rolling = self.rolling();
        let diving = self.diving && airborne;
        let switch_at = self.switch_at;
        let dive_landed_at = self.dive_landed_at;
        // The shield takes the first collision it does not win this tick. A collision after it (in
        // the same tick) ends the run as usual.
        let mut shield = self.ability_active();
        let mut crash = None;
        for foe in self.foes.iter_mut() {
            if foe.dead || foe.lane != lane || !(foe.at > prev && foe.at <= dist) {
                continue;
            }
            let meets = match foe.kind {
                FoeKind::Goon => y < GOON_HEIGHT,
                FoeKind::Flyer => top > FLYER_BOTTOM && y < FLYER_BOTTOM + FLYER_HEIGHT,
            };
            if !meets {
                continue;
            }
            let switched_close =
                switch_at.is_some_and(|s| s <= foe.at && foe.at - s <= SWITCH_REACH);
            let dived_close =
                dive_landed_at.is_some_and(|s| s <= foe.at && foe.at - s <= DIVE_REACH);
            let struck = match foe.kind {
                // A shielded goon takes only the dive: a roll or a lane switch does not count.
                FoeKind::Goon if foe.shielded => diving,
                FoeKind::Goon => rolling || diving || switched_close || dived_close,
                FoeKind::Flyer => airborne,
            };
            if struck {
                foe.dead = true;
                self.kills += 1;
                chain(&mut self.combo, &mut self.last_kill_at, &mut self.combo_points, foe.at);
            } else if shield {
                shield = false;
                foe.dead = true;
                self.kills += 1;
                self.ability_ticks = 0;
                chain(&mut self.combo, &mut self.last_kill_at, &mut self.combo_points, foe.at);
            } else {
                crash = crash.or(Some(foe.kind));
            }
        }
        if let Some(kind) = crash {
            self.phase = Phase::Crashed(kind);
        }
    }

    /// Makes rows of enemies until the road is made `HORIZON` metres ahead of the knight. A row has
    /// one enemy, or two in different lanes, so there is always a lane to go to.
    fn generate(&mut self) {
        while self.next_at < self.dist + HORIZON {
            let at = self.next_at;
            let count = if self.next_u64() % 3 == 0 { 2 } else { 1 };
            let first = (self.next_u64() % LANES as u64) as u8;
            for n in 0..count {
                let lane = (first + n) % LANES;
                let kind = if self.next_u64() % 2 == 0 {
                    FoeKind::Goon
                } else {
                    FoeKind::Flyer
                };
                let mut foe = Foe::new(at, lane, kind);
                // One goon in four is shielded (only a dive strikes it).
                if kind == FoeKind::Goon && self.next_u64() % 4 == 0 {
                    foe.shielded = true;
                }
                self.foes.push(foe);
            }
            self.make_pickup(at, first, count);
            // The next row comes after the reading time: a fixed minimum, plus the time to react at
            // the speed the knight will have there, plus a little seeded slack.
            let gap = GAP_MIN + 0.9 * speed_at(at) + 6.0 * self.unit();
            self.next_at = at + gap;
        }
    }

    /// Maybe lays a pickup at this row, in a lane the row's enemies leave free. A third of the rows
    /// have one: six in ten are a row of coins, two in ten ammo, two in ten a power-up.
    fn make_pickup(&mut self, at: f32, first: u8, count: u8) {
        if self.next_u64() % 3 != 0 {
            return;
        }
        let free: Vec<u8> = (0..LANES)
            .filter(|l| !(0..count).any(|n| (first + n) % LANES == *l))
            .collect();
        if free.is_empty() {
            return;
        }
        let lane = free[(self.next_u64() % free.len() as u64) as usize];
        match self.next_u64() % 10 {
            0..=5 => {
                for i in 0..COIN_ROW {
                    self.pickups.push(Pickup::new(
                        at + i as f32 * COIN_SPACING,
                        lane,
                        PickupKind::Coin,
                    ));
                }
            }
            6..=7 => self.pickups.push(Pickup::new(at, lane, PickupKind::Ammo)),
            _ => self.pickups.push(Pickup::new(at, lane, PickupKind::PowerUp)),
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in [0, 1).
    fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// Counts a kill at `at` into the chain: close to the last kill it extends the chain, otherwise it
/// starts one. A kill is worth its place in the chain, up to [`COMBO_MAX`].
fn chain(combo: &mut u32, last: &mut Option<f32>, points: &mut u32, at: f32) {
    let close = last.is_some_and(|k| at - k <= COMBO_REACH);
    *combo = if close { *combo + 1 } else { 1 };
    *last = Some(at);
    *points += (*combo).min(COMBO_MAX);
}

/// The running speed at a distance along the road.
pub fn speed_at(dist: f32) -> f32 {
    (SPEED_START + SPEED_PER_M * dist).min(SPEED_MAX)
}

#[cfg(test)]
mod tests {
    use super::{
        ABILITY_MAX_CHARGES, AMMO_PICKUP, Foe, FoeKind, Input, LANES, Phase, Pickup, PickupKind, Run,
    };

    fn run_with(pickups: Vec<Pickup>) -> Run {
        let mut run = Run::scripted(Vec::new());
        run.pickups = pickups;
        run
    }

    #[test]
    fn a_coin_in_the_lane_is_taken_and_scores() {
        let mut run = run_with(vec![Pickup::new(0.1, 1, PickupKind::Coin)]);
        run.step(Input::default());
        assert_eq!(run.coins, 1);
        assert_eq!(run.score(), run.metres() + 1);
        assert!(run.pickups[0].taken);
    }

    #[test]
    fn a_jump_above_a_pickup_misses_it() {
        let mut run = run_with(vec![Pickup::new(0.1, 1, PickupKind::Coin)]);
        run.y = 3.0;
        run.step(Input::default());
        assert_eq!(run.coins, 0);
        assert!(!run.pickups[0].taken);
    }

    #[test]
    fn a_pickup_in_another_lane_is_not_taken() {
        let mut run = run_with(vec![Pickup::new(0.1, 0, PickupKind::Ammo)]);
        run.step(Input::default());
        assert_eq!(run.ammo, 0);
    }

    #[test]
    fn five_coins_make_the_next_coins_worth_double() {
        // Six coins in one tick: five at 1 each, then the sixth at 2 (the multiplier after five).
        let coins = (0..6)
            .map(|i| Pickup::new(0.1 + i as f32 * 0.01, 1, PickupKind::Coin))
            .collect();
        let mut run = run_with(coins);
        run.step(Input::default());
        assert_eq!(run.coins, 6);
        assert_eq!(run.multiplier(), 2);
        assert_eq!(run.coin_points, 5 + 2);
    }

    #[test]
    fn ammo_and_power_ups_are_given_and_the_charges_stop_at_the_most() {
        let mut run = run_with(vec![
            Pickup::new(0.1, 1, PickupKind::Ammo),
            Pickup::new(0.2, 1, PickupKind::PowerUp),
        ]);
        run.ability_charges = ABILITY_MAX_CHARGES;
        run.step(Input::default());
        assert_eq!(run.ammo, AMMO_PICKUP);
        assert_eq!(run.ability_charges, ABILITY_MAX_CHARGES, "stacked to the most");
    }

    #[test]
    fn fresh_runs_lay_pickups_on_the_road() {
        // A row has a pickup one time in three, so any one seed may lay none nearby; the roads of
        // a few seeds together must.
        let runs: Vec<Run> = (0..20).map(Run::new).collect();
        assert!(runs.iter().any(|r| !r.pickups.is_empty()));
        assert!(runs.iter().flat_map(|r| r.pickups.iter()).all(|p| p.lane < LANES));
    }

    #[test]
    fn kills_close_together_make_a_chain_and_a_far_one_starts_again() {
        use super::{chain, COMBO_REACH};
        let (mut combo, mut last, mut points) = (0, None, 0);
        chain(&mut combo, &mut last, &mut points, 100.0);
        chain(&mut combo, &mut last, &mut points, 100.0 + COMBO_REACH);
        assert_eq!(combo, 2);
        assert_eq!(points, 1 + 2, "the first kill is worth 1, the second 2");
        chain(&mut combo, &mut last, &mut points, 500.0);
        assert_eq!(combo, 1, "a far kill starts a new chain");
    }

    #[test]
    fn a_chain_is_worth_at_most_the_cap_per_kill() {
        use super::{chain, COMBO_MAX};
        let (mut combo, mut last, mut points) = (0, None, 0);
        for i in 0..10 {
            chain(&mut combo, &mut last, &mut points, i as f32);
        }
        assert_eq!(combo, 10);
        // 1 + 2 + 3 + 4 + 5 (cap) + 5 * 5 more
        assert_eq!(points, 1 + 2 + 3 + 4 + 5 + 5 * (10 - 5));
        assert!(COMBO_MAX == 5);
    }

    fn shielded_goon_at(at: f32, lane: u8) -> Foe {
        let mut foe = Foe::new(at, lane, FoeKind::Goon);
        foe.shielded = true;
        foe
    }

    #[test]
    fn a_roll_does_not_strike_a_shielded_goon() {
        let mut run = Run::scripted(vec![shielded_goon_at(0.1, 1)]);
        run.step(Input {
            down: true,
            ..Input::default()
        });
        assert_eq!(run.phase, Phase::Crashed(FoeKind::Goon));
        assert_eq!(run.kills, 0);
    }

    #[test]
    fn a_lane_switch_does_not_strike_a_shielded_goon() {
        let mut run = Run::scripted(vec![shielded_goon_at(0.1, 2)]);
        run.step(Input {
            lane: 1,
            ..Input::default()
        });
        assert_eq!(run.phase, Phase::Crashed(FoeKind::Goon));
    }

    #[test]
    fn a_dive_strikes_a_shielded_goon() {
        let mut run = Run::scripted(vec![shielded_goon_at(0.1, 1)]);
        run.y = 0.9;
        run.step(Input {
            down: true,
            ..Input::default()
        });
        assert_eq!(run.phase, Phase::Running);
        assert_eq!(run.kills, 1);
    }

    #[test]
    fn some_goons_are_shielded_on_the_road() {
        let shielded = (0..50)
            .map(|seed| Run::new(seed).foes.iter().filter(|f| f.shielded).count())
            .sum::<usize>();
        assert!(shielded > 0, "some goons are shielded");
    }

    #[test]
    fn an_input_survives_the_flag_byte() {
        use super::{Input, pack_input, unpack_input};
        for input in [
            Input::default(),
            Input { lane: -1, ..Input::default() },
            Input { lane: 1, jump: true, ..Input::default() },
            Input { down: true, ability: true, ..Input::default() },
        ] {
            assert_eq!(unpack_input(pack_input(input)), input);
        }
    }

    #[test]
    fn a_ghost_replays_the_recorded_run_exactly() {
        use super::{Input, Run, pack_input, unpack_input};
        use xerxes_sim::ghost::{Ghost, Recording};

        // Ride a seed for a while, with a pattern of presses, and record the inputs.
        let seed = 9;
        let mut run = Run::new(seed);
        let mut inputs = Vec::new();
        for t in 0..600u32 {
            let input = Input {
                lane: match t / 90 % 3 {
                    0 => 0,
                    1 => 1,
                    _ => -1,
                },
                jump: t % 45 == 10,
                down: t % 70 == 40,
                ability: t == 300,
            };
            inputs.push(pack_input(input));
            run.step(input);
        }
        let recording = Recording {
            key: seed,
            score: run.metres(),
            inputs,
        };
        // The text round-trips, and the ghost on the same road lands where the run did.
        let text = recording.encode("cyrus1");
        let back = Recording::decode("cyrus1", &text).expect("a recording");
        let mut ghost = Ghost::new(Run::new(back.key), &back, unpack_input);
        while !ghost.finished() {
            ghost.step();
        }
        assert_eq!(ghost.sim().dist, run.dist);
        assert_eq!(ghost.sim().y, run.y);
        assert_eq!(ghost.sim().lane, run.lane);
        assert_eq!(ghost.sim().kills, run.kills);
    }

    #[test]
    fn a_slow_frame_cannot_fast_forward_the_run() {
        use super::{DT, MAX_CATCH_UP, Pace};

        let mut pace = Pace::default();
        // A frame of one tick is one tick, and the leftovers of short frames add up to one.
        assert_eq!(pace.ticks(DT), 1);
        assert_eq!(pace.ticks(DT / 2.0), 0);
        assert_eq!(pace.ticks(DT / 2.0), 1);

        // A frame that took two seconds (a level still being built, a phone that stalled) runs
        // the few ticks the cap allows, not two seconds of road.
        let mut pace = Pace::default();
        let ticks = pace.ticks(2.0);
        assert_eq!(
            ticks,
            (MAX_CATCH_UP / DT) as u32,
            "the cap's worth of ticks"
        );
        assert!(
            ticks < (2.0 / DT) as u32,
            "{ticks} ticks is far fewer than the frame's own two seconds"
        );
        // And the two seconds are not kept to be spent later.
        assert_eq!(pace.ticks(0.0), 0);
        assert!(pace.ticks(DT) <= 1);

        // Nothing silly gets through.
        let mut pace = Pace::default();
        assert_eq!(pace.ticks(-1.0), 0);
        assert_eq!(pace.ticks(f32::NAN), 0);
        pace.ticks(DT * 1.5);
        pace.reset();
        assert_eq!(pace.ticks(DT / 2.0), 0, "a reset drops the leftover");
    }

    use super::*;

    fn none() -> Input {
        Input::default()
    }
    fn jump() -> Input {
        Input {
            jump: true,
            ..Default::default()
        }
    }
    fn down() -> Input {
        Input {
            down: true,
            ..Default::default()
        }
    }
    fn lane(by: i8) -> Input {
        Input {
            lane: by,
            ..Default::default()
        }
    }

    /// The distances from `from` to `to` in steps of `step`, both ends included.
    fn sweep(from: f32, to: f32, step: f32) -> Vec<f32> {
        let mut out = Vec::new();
        let mut s = from;
        while s <= to + 1e-4 {
            out.push(s);
            s += step;
        }
        out
    }

    /// Steps with no input for `ticks` ticks.
    fn wait(run: &mut Run, ticks: u32) {
        for _ in 0..ticks {
            run.step(none());
        }
    }

    /// Runs until the enemy is `s` metres ahead, applies `act` for one tick, then runs on past it.
    fn act_at(foe: Foe, s: f32, act: Input) -> Run {
        let mut run = Run::scripted(vec![foe]);
        while run.foes[0].at - run.dist > s {
            run.step(none());
        }
        run.step(act);
        wait(&mut run, 400);
        run
    }

    // --- the road: a straight run into an enemy loses, whatever the enemy ----------------------

    #[test]
    fn running_straight_into_a_goon_loses() {
        let mut run = Run::scripted(vec![Foe::new(10.0, 1, FoeKind::Goon)]);
        wait(&mut run, 400);
        assert_eq!(run.phase, Phase::Crashed(FoeKind::Goon));
        assert_eq!(run.kills, 0);
    }

    #[test]
    fn a_standing_knight_is_hit_by_a_flyer() {
        let mut run = Run::scripted(vec![Foe::new(12.0, 1, FoeKind::Flyer)]);
        wait(&mut run, 400);
        assert_eq!(run.phase, Phase::Crashed(FoeKind::Flyer));
    }

    #[test]
    fn enemies_in_another_lane_are_never_a_problem() {
        let mut run = Run::scripted(vec![
            Foe::new(10.0, 0, FoeKind::Goon),
            Foe::new(14.0, 2, FoeKind::Flyer),
        ]);
        wait(&mut run, 400);
        assert_eq!(run.phase, Phase::Running);
        assert_eq!(run.kills, 0);
    }

    // --- switching lane into a goon's lane: strikes it only at the exact moment ---------------

    #[test]
    fn switching_into_a_goon_within_reach_strikes_it() {
        for s in sweep(0.25, SWITCH_REACH, 0.25) {
            let run = act_at(Foe::new(15.0, 0, FoeKind::Goon), s, lane(-1));
            assert_eq!(run.kills, 1, "switch at {s} m should strike the goon");
            assert_eq!(
                run.phase,
                Phase::Running,
                "switch at {s} m should not crash"
            );
        }
    }

    #[test]
    fn switching_too_early_does_not_strike_and_the_goon_hits() {
        for s in sweep(SWITCH_REACH + 0.5, 8.0, 0.5) {
            let run = act_at(Foe::new(15.0, 0, FoeKind::Goon), s, lane(-1));
            assert_eq!(run.kills, 0, "switch at {s} m is too early to strike");
            assert_eq!(
                run.phase,
                Phase::Crashed(FoeKind::Goon),
                "switch at {s} m should lose"
            );
        }
    }

    // --- rolling on the ground strikes a goon in its lane -------------------------------------

    #[test]
    fn rolling_on_the_ground_strikes_a_goon_in_its_lane() {
        // The roll lasts about 7 m: it is still on when the goon is met, up to that far ahead.
        for s in sweep(0.25, 6.0, 0.25) {
            let run = act_at(Foe::new(15.0, 1, FoeKind::Goon), s, down());
            assert_eq!(run.kills, 1, "a roll at {s} m should strike the goon");
            assert_eq!(
                run.phase,
                Phase::Running,
                "a roll at {s} m should not crash"
            );
        }
    }

    #[test]
    fn a_goon_is_not_killed_before_the_bodies_meet() {
        let mut run = Run::scripted(vec![Foe::new(15.0, 1, FoeKind::Goon)]);
        while run.foes[0].at - run.dist > 5.0 {
            run.step(none());
        }
        run.step(down());
        while run.foes[0].at > run.dist {
            run.step(none());
            if run.foes[0].at > run.dist {
                assert_eq!(
                    run.kills,
                    0,
                    "the goon died {} m before the knight met it",
                    run.foes[0].at - run.dist
                );
            }
        }
        assert_eq!(run.kills, 1, "the roll strikes it when they meet");
    }

    // --- diving in the air strikes a goon in its lane -----------------------------------------

    #[test]
    fn a_jump_and_dive_that_lands_just_before_a_goon_strikes_it() {
        // Where a jump-and-dive lands, with no enemy on the road.
        let mut probe = Run::scripted(vec![]);
        probe.step(jump());
        wait(&mut probe, 10);
        probe.step(down());
        while probe.y > 0.0 || probe.tick < 20 {
            probe.step(none());
        }
        let landed = probe.dist;
        assert!(
            probe.dive_landed_at.is_some(),
            "the dive should have landed"
        );

        // The same inputs, with a goon a little past the landing: the dive has landed close before it.
        for (gap, want) in [(2.0, 1), (8.0, 0)] {
            let mut run = Run::scripted(vec![Foe::new(landed + gap, 1, FoeKind::Goon)]);
            run.step(jump());
            wait(&mut run, 10);
            run.step(down());
            wait(&mut run, 400);
            assert_eq!(run.kills, want, "a goon {gap} m past the dive's landing");
        }
    }

    #[test]
    fn a_dive_still_falling_when_the_goon_is_met_strikes_it() {
        // Jump well before the goon, then dive at a point ahead of it: the dive strikes the goon only
        // if the knight is still falling through the goon's height when they meet.
        let mut struck = 0;
        for s in sweep(0.0, 2.0, 0.05) {
            let mut run = Run::scripted(vec![Foe::new(15.0, 1, FoeKind::Goon)]);
            while run.foes[0].at - run.dist > 8.0 {
                run.step(none());
            }
            run.step(jump());
            while run.foes[0].at - run.dist > s {
                run.step(none());
            }
            run.step(down());
            wait(&mut run, 400);
            if run.kills == 1 {
                struck += 1;
                assert_eq!(
                    run.phase,
                    Phase::Running,
                    "a dive at {s} m that strikes should not crash"
                );
            }
        }
        assert!(
            struck > 0,
            "some dive timing should meet the goon while falling"
        );
    }

    // --- jumping: over a goon, and through a flyer -------------------------------------------

    #[test]
    fn a_jump_over_a_goon_clears_it_without_a_strike() {
        let mut cleared = 0;
        for s in sweep(0.5, 12.0, 0.25) {
            let run = act_at(Foe::new(15.0, 1, FoeKind::Goon), s, jump());
            if run.phase == Phase::Running {
                cleared += 1;
                assert_eq!(
                    run.kills, 0,
                    "jumping over a goon strikes nothing (at {s} m)"
                );
            }
        }
        // A jump is high enough to clear a goon for a stretch of the approach, not one exact frame.
        assert!(
            cleared >= 16,
            "only {cleared} of the approaches cleared the goon"
        );
    }

    #[test]
    fn a_jump_too_late_hits_the_goon() {
        // Taken off this close, the knight is still under the goon's top when he passes it.
        let run = act_at(Foe::new(15.0, 1, FoeKind::Goon), 0.5, jump());
        assert_eq!(run.phase, Phase::Crashed(FoeKind::Goon));
    }

    #[test]
    fn a_jump_through_a_flyer_strikes_it() {
        for s in sweep(1.0, 9.0, 0.25) {
            let run = act_at(Foe::new(15.0, 1, FoeKind::Flyer), s, jump());
            assert_eq!(
                run.kills, 1,
                "a jump at {s} m should strike the flyer in the air"
            );
            assert_eq!(
                run.phase,
                Phase::Running,
                "a jump at {s} m should not crash"
            );
        }
    }

    #[test]
    fn a_jump_that_lands_before_the_flyer_hits_it() {
        // Taken off this far ahead, the knight has landed when he reaches the flyer.
        let run = act_at(Foe::new(15.0, 1, FoeKind::Flyer), 11.0, jump());
        assert_eq!(run.kills, 0);
        assert_eq!(run.phase, Phase::Crashed(FoeKind::Flyer));
    }

    // --- rolling under a flyer passes it ------------------------------------------------------

    #[test]
    fn rolling_under_a_flyer_passes_it() {
        for s in sweep(0.25, 6.0, 0.25) {
            let run = act_at(Foe::new(15.0, 1, FoeKind::Flyer), s, down());
            assert_eq!(
                run.phase,
                Phase::Running,
                "a roll at {s} m should pass under the flyer"
            );
            assert_eq!(run.kills, 0, "a roll does not strike a flyer");
        }
    }

    // --- the jump itself -----------------------------------------------------------------------

    #[test]
    fn a_jump_is_1_28_times_the_first_height() {
        let mut run = Run::scripted(vec![]);
        run.step(jump());
        let mut top = 0.0f32;
        for _ in 0..90 {
            run.step(none());
            top = top.max(run.y);
        }
        // The first version's apex on this clock is 1.275 m; 1.28 times that is 1.632 m.
        assert!(
            (top - 1.632).abs() < 0.01,
            "apex {top} should be about 1.632"
        );
    }

    #[test]
    fn a_jump_pressed_just_before_landing_is_kept() {
        let mut run = Run::scripted(vec![]);
        run.step(jump());
        let mut pressed = false;
        for _ in 0..200 {
            let early = !pressed && run.vy < 0.0 && run.y < 0.3;
            pressed |= early;
            run.step(Input {
                jump: early,
                ..Default::default()
            });
            if pressed && run.y == 0.0 {
                break;
            }
        }
        assert!(pressed, "the test should press on the way down");
        run.step(none());
        assert!(
            run.y > 0.0,
            "the buffered jump should lift the knight again"
        );
    }

    // --- the special ability: one charge a run, on for 10 seconds -----------------------------

    fn ability() -> Input {
        Input {
            ability: true,
            ..Default::default()
        }
    }

    #[test]
    fn a_run_starts_with_one_charge_and_the_ability_is_off() {
        let run = Run::new(1);
        assert_eq!(run.ability_charges, ABILITY_CHARGES);
        assert!(!run.ability_active());
    }

    #[test]
    fn taking_the_ability_turns_it_on_for_ten_seconds() {
        let mut run = Run::scripted(vec![]);
        run.step(ability());
        assert!(run.ability_active());
        assert_eq!(run.ability_charges, 0);
        for _ in 1..ABILITY_TICKS {
            run.step(none());
        }
        assert!(
            run.ability_active(),
            "still on for the whole of the ten seconds"
        );
        run.step(none());
        assert!(!run.ability_active(), "off once the ten seconds are over");
    }

    #[test]
    fn with_no_charge_left_the_ability_does_nothing() {
        let mut run = Run::scripted(vec![]);
        run.step(ability());
        wait(&mut run, ABILITY_TICKS);
        assert!(!run.ability_active());
        run.step(ability());
        assert!(
            !run.ability_active(),
            "the one charge is spent for this run"
        );
        assert_eq!(run.ability_charges, 0);
    }

    #[test]
    fn the_shield_takes_a_collision_and_the_knight_lives() {
        for foe in [
            Foe::new(10.0, 1, FoeKind::Goon),
            Foe::new(12.0, 1, FoeKind::Flyer),
        ] {
            let mut run = Run::scripted(vec![foe]);
            run.step(ability());
            wait(&mut run, 400);
            assert_eq!(
                run.phase,
                Phase::Running,
                "{:?}: the shield should save the knight",
                foe.kind
            );
            assert_eq!(
                run.kills, 1,
                "{:?}: the shield kills the enemy it takes",
                foe.kind
            );
            assert!(
                !run.ability_active(),
                "{:?}: the shield ends on the collision",
                foe.kind
            );
        }
    }

    #[test]
    fn the_shield_is_used_up_by_one_collision() {
        let mut run = Run::scripted(vec![
            Foe::new(10.0, 1, FoeKind::Goon),
            Foe::new(20.0, 1, FoeKind::Goon),
        ]);
        run.step(ability());
        wait(&mut run, 600);
        assert_eq!(run.kills, 1);
        assert_eq!(
            run.phase,
            Phase::Crashed(FoeKind::Goon),
            "the second goon ends the run"
        );
    }

    #[test]
    fn the_shield_ends_after_ten_seconds_and_a_later_collision_ends_the_run() {
        let mut run = Run::scripted(vec![Foe::new(200.0, 1, FoeKind::Goon)]);
        run.step(ability());
        wait(&mut run, ABILITY_TICKS);
        assert!(!run.ability_active());
        wait(&mut run, 400);
        assert_eq!(run.phase, Phase::Crashed(FoeKind::Goon));
        assert_eq!(run.kills, 0);
    }

    #[test]
    fn pressing_it_again_while_on_does_not_use_a_second_charge() {
        let mut run = Run::scripted(vec![]);
        run.step(ability());
        run.step(ability());
        assert!(run.ability_active());
        assert_eq!(run.ability_charges, 0);
    }

    // --- lanes ---------------------------------------------------------------------------------

    #[test]
    fn lanes_stop_at_the_edges() {
        let mut run = Run::scripted(vec![]);
        for _ in 0..5 {
            run.step(lane(-1));
        }
        assert_eq!(run.lane, 0);
        for _ in 0..5 {
            run.step(lane(1));
        }
        assert_eq!(run.lane, LANES - 1);
    }

    // --- the road is the same on every device and fair --------------------------------------

    #[test]
    fn the_same_seed_gives_the_same_road_and_the_same_run() {
        let mut a = Run::new(7);
        let mut b = Run::new(7);
        assert_eq!(a.foes, b.foes);
        for _ in 0..2000 {
            a.step(none());
            b.step(none());
        }
        assert_eq!(a.phase, b.phase);
        assert_eq!(a.dist, b.dist);
        assert_eq!(a.foes, b.foes);
    }

    #[test]
    fn the_road_is_clear_at_the_start_and_fair_between_rows() {
        for seed in 1..50u64 {
            let run = Run::new(seed);
            assert!(
                run.foes[0].at >= START_CLEAR,
                "seed {seed}: first enemy too close"
            );
            let mut rows: Vec<f32> = run.foes.iter().map(|f| f.at).collect();
            rows.dedup();
            for pair in rows.windows(2) {
                let gap = pair[1] - pair[0];
                assert!(
                    gap >= GAP_MIN + 0.9 * speed_at(pair[0]),
                    "seed {seed}: gap {gap} too short"
                );
            }
        }
    }

    #[test]
    fn a_row_never_fills_every_lane() {
        for seed in 1..200u64 {
            let run = Run::new(seed);
            for foe in &run.foes {
                let in_row = run.foes.iter().filter(|f| f.at == foe.at).count();
                assert!(
                    in_row < LANES as usize,
                    "seed {seed}: a row blocks every lane"
                );
            }
        }
    }

    #[test]
    fn the_run_gets_faster_with_distance_up_to_the_cap() {
        assert_eq!(speed_at(0.0), SPEED_START);
        assert!(speed_at(500.0) > SPEED_START);
        assert_eq!(speed_at(1.0e6), SPEED_MAX);
    }

    #[test]
    fn a_long_random_run_never_breaks_its_own_rules() {
        // Any input sequence: the run stays finite, moves forward, and never leaves the lanes.
        for seed in 1..20u64 {
            let mut run = Run::new(seed);
            let mut rng = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            for _ in 0..20_000 {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                let input = Input {
                    lane: ((rng % 3) as i8) - 1,
                    jump: rng % 5 == 0,
                    down: rng % 7 == 0,
                    ability: rng % 11 == 0,
                };
                let before = run.dist;
                run.step(input);
                assert!(
                    run.dist >= before,
                    "seed {seed}: the distance went backwards"
                );
                assert!(run.lane < LANES, "seed {seed}: left the road");
                assert!(run.y.is_finite(), "seed {seed}: the height is not a number");
                if matches!(run.phase, Phase::Crashed(_)) {
                    break;
                }
            }
        }
    }
}
