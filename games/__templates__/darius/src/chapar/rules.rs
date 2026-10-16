//! Chapar's rules, without Bevy: a rider on an endless road, running by himself. The player only
//! jumps (hold for height) and ducks. The road is made from a seed, as obstacles laid out ahead of
//! the rider: blocks to jump, gates to duck under, pits to jump across. A crash or a fall ends the
//! run. The score is metres plus darics and grazes, times a chain multiplier (a relay token
//! bursts it). Now and then a relay station stands on the road with a fresh horse held high over
//! it: jump (and hold) to take it, and the next stretch is run faster.
//!
//! It runs on a fixed clock ([`TICKS_PER_SECOND`], the Bevy side steps it once per fixed tick)
//! and nothing in it reads a clock or a random source of its own, so the same seed and the same
//! [`Input`]s always give the same run. That is what a ghost, a replay and a daily board need.
//!
//! Units are pixels and seconds; `x` is how far the road has been run, `y` is the rider's height
//! above the road. The rules live in the game while Chapar is their only user; the platformer
//! controller is extracted when Mobarezeh needs the same thing.

/// The fixed clock the Bevy side steps the run on.
pub const TICKS_PER_SECOND: u32 = 60;
const DT: f32 = 1.0 / TICKS_PER_SECOND as f32;

/// Pixels in a metre of score.
pub const PX_PER_METRE: f32 = 20.0;
/// Points added per metre run, per graze, and per daric, each times the current multiplier.
const POINTS_PER_METRE: i64 = 1;
const POINTS_PER_GRAZE: i64 = 50;
const POINTS_PER_DARIC: i64 = 10;
/// Chain multiplier: starts at 1, grows with consecutive darics, caps here; a miss resets to 1.
const MULT_MAX: u32 = 8;
/// Taking a relay token adds this many steps to the multiplier (still capped).
const STATION_BURST: u32 = 3;
/// How close a clear pass must be to count as a graze (block top / gate underside), in pixels.
const GRAZE_CLEAR: f32 = 12.0;
/// A daric's half-size and default height above the road (a hop reaches it; a run under does not).
pub const DARIC_R: f32 = 14.0;
pub const DARIC_Y: f32 = 52.0;

/// Running speed at the start and at the cap (px/s), and how much faster the road gets per pixel
/// run. Speed depends on the distance, not on a clock, so the generator knows how fast the rider
/// will be at any obstacle and can keep every one fair.
const SPEED_START: f32 = 380.0;
const SPEED_MAX: f32 = 760.0;
const SPEED_PER_PX: f32 = 0.0125;

const GRAVITY: f32 = 2600.0;
const JUMP_SPEED: f32 = 720.0;
/// Gravity factor while rising with the jump held (a higher jump) and after letting go (a hop).
const RISE_HELD: f32 = 0.6;
const RISE_CUT: f32 = 2.0;
/// Gravity factor when ducking in the air (a fast fall).
const FAST_FALL: f32 = 2.2;
/// A jump pressed this long before landing, or this long after leaving the road, still counts.
const BUFFER: f32 = 0.12;
const COYOTE: f32 = 0.10;

/// The rider (horse and courier as one body): its drawn width, its hit width (a little
/// narrower, so a graze is forgiven) and its height standing and ducking.
pub const RIDER_WIDTH: f32 = 56.0;
const HIT_WIDTH: f32 = 40.0;
pub const STAND_HEIGHT: f32 = 64.0;
pub const DUCK_HEIGHT: f32 = 30.0;

/// The underside of a gate: ducking (30) passes, standing (64) hits.
pub const GATE_CLEAR: f32 = 40.0;
/// How much of a pit's edge still counts as road.
const PIT_EDGE: f32 = 8.0;
/// How far below the road the rider falls before the run is over.
const FALL_OUT: f32 = 60.0;

/// The road is clear this far from the start, then the first obstacle comes.
const START_CLEAR: f32 = 900.0;
/// Pits and gates start after this distance; the first stretch is blocks only.
const HARD_FROM: f32 = 1800.0;
/// How far ahead of the rider obstacles are made.
const HORIZON: f32 = 2400.0;
/// How far behind the rider obstacles are kept (so the Bevy side can still draw them leaving).
const KEEP_BEHIND: f32 = 700.0;

/// A relay station: this much road, with a token held at `TOKEN_LOW..TOKEN_HIGH` above it that a
/// held jump reaches (a hop does not). Taking it makes the stretch after the station a sprint.
pub const STATION_W: f32 = 240.0;
pub const TOKEN_W: f32 = 56.0;
pub const TOKEN_LOW: f32 = 130.0;
pub const TOKEN_HIGH: f32 = 180.0;
/// How long the sprint is, and how much faster the rider runs in it.
pub const SPRINT_LEN: f32 = 1100.0;
pub const SPRINT_FACTOR: f32 = 1.5;
/// The first station comes after this much road; then one every so often.
const FIRST_STATION: f32 = 2600.0;
const STATION_GAP: f32 = 2400.0;
const STATION_GAP_VARY: f32 = 1600.0;

/// What the player holds this tick. The Bevy side samples it once per fixed tick; a ghost is just
/// a recording of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Input {
    /// Jump went down since the last tick.
    pub jump_pressed: bool,
    /// Jump is down.
    pub jump_held: bool,
    /// Duck is down.
    pub duck_held: bool,
}

/// What an obstacle is, and so how it is passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A block on the road: jump over it.
    Block,
    /// A gate across the road: duck under it.
    Gate,
    /// A gap in the road: jump across it.
    Pit,
}

/// Something on the road, from its left edge `x` and `w` wide. A block is `h` tall; a gate's
/// underside is `h` (= [`GATE_CLEAR`]) from the road; a pit has no height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Obstacle {
    /// Counts up from 0 in the order obstacles are made, so a view can tell which ones it
    /// already shows.
    pub id: u32,
    pub kind: Kind,
    pub x: f32,
    pub w: f32,
    pub h: f32,
}

/// A relay station, from its left edge `x`. The sprint it gives starts where the station ends,
/// whether or not the token is taken there, so the road after it can be made fair in advance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Station {
    pub id: u32,
    pub x: f32,
    pub collected: bool,
}

/// A gold daric floating above the road: jump through it to collect. Darics of one `line` are a
/// chain; missing one (passing its `x` without taking it) resets the score multiplier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Daric {
    pub id: u32,
    pub line: u32,
    pub x: f32,
    pub y: f32,
    pub collected: bool,
}

impl Station {
    /// Where the token is centred.
    pub fn token_x(&self) -> f32 {
        self.x + STATION_W / 2.0
    }

    pub fn sprint_start(&self) -> f32 {
        self.x + STATION_W
    }

    pub fn sprint_end(&self) -> f32 {
        self.sprint_start() + SPRINT_LEN
    }
}

/// What ended the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cause {
    /// Ran into a block.
    Block,
    /// Ran into a gate (not ducking).
    Gate,
    /// Fell into a pit.
    Pit,
}

/// Where the run is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Running,
    Over(Cause),
}

/// A small deterministic random source (SplitMix64): the road must be the same on every device.
#[derive(Debug, Clone)]
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `[0, 1)`.
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// The running speed at a distance.
pub fn speed_at(x: f32) -> f32 {
    (SPEED_START + SPEED_PER_PX * x).min(SPEED_MAX)
}

/// A run: the rider, and the road ahead made from the seed.
#[derive(Debug, Clone)]
pub struct Run {
    seed: u64,
    rng: Rng,
    obstacles: Vec<Obstacle>,
    next_id: u32,
    stations: Vec<Station>,
    next_station_id: u32,
    darics: Vec<Daric>,
    next_daric_id: u32,
    next_line_id: u32,
    /// Road is laid up to a station once the cursor passes this.
    next_station_at: f32,
    /// Where the last obstacle made ends.
    cursor: f32,
    x: f32,
    y: f32,
    vy: f32,
    grounded: bool,
    ducking: bool,
    coyote: f32,
    buffer: f32,
    phase: Phase,
    ticks: u32,
    /// Running score (metres + darics + grazes, each times the multiplier of the moment).
    score: i64,
    /// Metres already folded into [`Self::score`].
    metres_scored: u32,
    /// Chain multiplier (at least 1).
    multiplier: u32,
    /// Consecutive darics taken without a miss.
    chain: u32,
    darics_taken: u32,
    grazes: u32,
    /// Obstacle ids that already awarded a graze this pass.
    grazed: Vec<u32>,
    /// While overlapping an obstacle, how close the clearance got (for graze on clear).
    near: Vec<(u32, f32)>,
}

impl Run {
    /// A fresh run on the road of `seed` (a day's seed gives the day's road).
    pub fn new(seed: u64) -> Self {
        let mut run = Self {
            seed,
            rng: Rng(seed),
            obstacles: Vec::new(),
            next_id: 0,
            stations: Vec::new(),
            next_station_id: 0,
            darics: Vec::new(),
            next_daric_id: 0,
            next_line_id: 0,
            next_station_at: FIRST_STATION,
            cursor: START_CLEAR,
            x: 0.0,
            y: 0.0,
            vy: 0.0,
            grounded: true,
            ducking: false,
            coyote: COYOTE,
            buffer: 0.0,
            phase: Phase::Running,
            ticks: 0,
            score: 0,
            metres_scored: 0,
            multiplier: 1,
            chain: 0,
            darics_taken: 0,
            grazes: 0,
            grazed: Vec::new(),
            near: Vec::new(),
        };
        run.lay_road();
        run
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// How far the road has been run, in pixels.
    pub fn x(&self) -> f32 {
        self.x
    }

    /// The rider's height above the road.
    pub fn y(&self) -> f32 {
        self.y
    }

    /// The rider's vertical speed (up is positive), for drawing between ticks.
    pub fn vy(&self) -> f32 {
        self.vy
    }

    /// The speed now: the road's, times the sprint factor while sprinting.
    pub fn speed(&self) -> f32 {
        let base = speed_at(self.x);
        if self.sprinting() {
            base * SPRINT_FACTOR
        } else {
            base
        }
    }

    /// Whether the rider is in the sprint of a station whose token it took.
    pub fn sprinting(&self) -> bool {
        self.stations
            .iter()
            .any(|s| s.collected && self.x >= s.sprint_start() && self.x < s.sprint_end())
    }

    /// The stations made so far that are still near the rider.
    pub fn stations(&self) -> &[Station] {
        &self.stations
    }

    /// How far the road has been run, in metres (shown beside the score).
    pub fn metres(&self) -> u32 {
        (self.x / PX_PER_METRE) as u32
    }

    /// The running score (Solo x Score).
    pub fn score(&self) -> i64 {
        self.score
    }

    /// The chain multiplier right now (at least 1).
    pub fn multiplier(&self) -> u32 {
        self.multiplier
    }

    pub fn darics_taken(&self) -> u32 {
        self.darics_taken
    }

    pub fn grazes(&self) -> u32 {
        self.grazes
    }

    /// Darics still near the rider (for drawing).
    pub fn darics(&self) -> &[Daric] {
        &self.darics
    }

    pub fn grounded(&self) -> bool {
        self.grounded
    }

    pub fn ducking(&self) -> bool {
        self.ducking
    }

    /// The rider's height now (the drawn body shrinks when ducking).
    pub fn height(&self) -> f32 {
        if self.ducking {
            DUCK_HEIGHT
        } else {
            STAND_HEIGHT
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn is_over(&self) -> bool {
        matches!(self.phase, Phase::Over(_))
    }

    /// Fixed ticks the run has lasted.
    pub fn ticks(&self) -> u32 {
        self.ticks
    }

    /// The obstacles made so far that are still near the rider (some are behind it).
    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }

    /// Whether the road is under the rider at `x`.
    fn supported(&self, x: f32) -> bool {
        !self
            .obstacles
            .iter()
            .any(|o| o.kind == Kind::Pit && x > o.x + PIT_EDGE && x < o.x + o.w - PIT_EDGE)
    }

    /// Advances the run one fixed tick. Nothing happens once it is over.
    pub fn step(&mut self, input: Input) {
        if self.is_over() {
            return;
        }
        self.ticks += 1;

        // A jump pressed a moment early is kept until the rider can use it.
        if input.jump_pressed {
            self.buffer = BUFFER;
        } else {
            self.buffer = (self.buffer - DT).max(0.0);
        }
        if self.grounded {
            self.coyote = COYOTE;
        } else {
            self.coyote = (self.coyote - DT).max(0.0);
        }
        if self.buffer > 0.0 && (self.grounded || self.coyote > 0.0) {
            self.vy = JUMP_SPEED;
            self.grounded = false;
            self.coyote = 0.0;
            self.buffer = 0.0;
        }

        // Hold for a higher jump, let go for a hop; duck in the air to fall fast.
        let factor = if self.vy > 0.0 {
            if input.jump_held { RISE_HELD } else { RISE_CUT }
        } else if input.duck_held && !self.grounded {
            FAST_FALL
        } else {
            1.0
        };
        if !self.grounded {
            self.vy -= GRAVITY * factor * DT;
            self.y += self.vy * DT;
        }
        self.x += self.speed() * DT;

        // Landing, or going over an edge.
        if self.y <= 0.0 && self.vy <= 0.0 {
            if self.supported(self.x) {
                self.y = 0.0;
                self.vy = 0.0;
                self.grounded = true;
            } else {
                if self.grounded {
                    self.grounded = false;
                }
                if self.y < -FALL_OUT {
                    self.phase = Phase::Over(Cause::Pit);
                }
            }
        }
        if self.grounded && !self.supported(self.x) {
            // Rode off the edge: now falling.
            self.grounded = false;
        }
        self.ducking = input.duck_held && self.grounded;

        self.collide();
        self.score_tick();
        self.lay_road();
        self.forget_behind();
    }

    /// Metres, darics and grazes fold into [`Self::score`] each tick (times the current multiplier).
    fn score_tick(&mut self) {
        let metres = self.metres();
        if metres > self.metres_scored {
            let gained = (metres - self.metres_scored) as i64;
            self.score += gained * POINTS_PER_METRE * self.multiplier as i64;
            self.metres_scored = metres;
        }
        // A daric left behind without being taken breaks the chain.
        let x = self.x;
        let mut missed = false;
        for d in &self.darics {
            if !d.collected && d.x + DARIC_R < x - HIT_WIDTH / 2.0 {
                missed = true;
                break;
            }
        }
        if missed {
            self.chain = 0;
            self.multiplier = 1;
            self.darics.retain(|d| d.collected || d.x + DARIC_R >= x - HIT_WIDTH / 2.0);
        }
    }

    fn bump_multiplier(&mut self, add: u32) {
        self.multiplier = (self.multiplier + add).min(MULT_MAX).max(1);
    }

    fn collide(&mut self) {
        if self.is_over() {
            return;
        }
        let (left, right) = (self.x - HIT_WIDTH / 2.0, self.x + HIT_WIDTH / 2.0);
        let top = self.y + self.height();
        // The token is taken by reaching it: over its place, with the body in its height.
        let reach = (TOKEN_W + HIT_WIDTH) / 2.0;
        let mut took_token = false;
        for st in &mut self.stations {
            if !st.collected
                && (self.x - st.token_x()).abs() < reach
                && top > TOKEN_LOW
                && self.y < TOKEN_HIGH
            {
                st.collected = true;
                took_token = true;
            }
        }
        if took_token {
            // A fresh horse bursts the multiplier on top of the sprint.
            self.chain = self.chain.saturating_add(STATION_BURST);
            self.bump_multiplier(STATION_BURST);
        }
        // Darics: jump through them (several may land in one tick).
        let reach_d = HIT_WIDTH / 2.0 + DARIC_R;
        let mut took = 0u32;
        for d in &mut self.darics {
            if d.collected {
                continue;
            }
            if (self.x - d.x).abs() < reach_d && self.y < d.y + DARIC_R && top > d.y - DARIC_R {
                d.collected = true;
                took += 1;
            }
        }
        for _ in 0..took {
            self.darics_taken += 1;
            self.chain += 1;
            self.multiplier = self.chain.min(MULT_MAX).max(1);
            self.score += POINTS_PER_DARIC * self.multiplier as i64;
        }
        let mut crash: Option<Cause> = None;
        for o in &self.obstacles {
            if right <= o.x || left >= o.x + o.w {
                // Just cleared: if we were close, that is a graze.
                if left > o.x + o.w
                    && !self.grazed.contains(&o.id)
                    && matches!(o.kind, Kind::Block | Kind::Gate)
                {
                    if let Some((_, clearance)) = self.near.iter().find(|(id, _)| *id == o.id) {
                        if *clearance <= GRAZE_CLEAR {
                            self.grazed.push(o.id);
                            self.grazes += 1;
                            self.score += POINTS_PER_GRAZE * self.multiplier as i64;
                        }
                    }
                }
                continue;
            }
            let hit = match o.kind {
                Kind::Block => self.y < o.h,
                Kind::Gate => top > o.h,
                Kind::Pit => false,
            };
            if hit {
                crash = Some(match o.kind {
                    Kind::Block => Cause::Block,
                    _ => Cause::Gate,
                });
                break;
            }
            // Clearance while overlapping: block = how far above the top; gate = how far under.
            let clearance = match o.kind {
                Kind::Block => self.y - o.h,
                Kind::Gate => o.h - top,
                Kind::Pit => f32::MAX,
            };
            if clearance >= 0.0 {
                if let Some((_, c)) = self.near.iter_mut().find(|(id, _)| *id == o.id) {
                    *c = c.min(clearance);
                } else {
                    self.near.push((o.id, clearance));
                }
            }
        }
        if let Some(cause) = crash {
            self.phase = Phase::Over(cause);
        }
    }

    /// The speed the road is made for at `x`: a sprint is planned as taken (a rider who missed
    /// the token only meets it slower).
    fn planned_speed(&self, x: f32) -> f32 {
        let base = speed_at(x);
        if self.stations.iter().any(|s| x >= s.sprint_start() && x < s.sprint_end()) {
            base * SPRINT_FACTOR
        } else {
            base
        }
    }

    /// Makes obstacles and stations until the road is laid to the horizon.
    fn lay_road(&mut self) {
        while self.cursor < self.x + HORIZON {
            // A station, when the road has gone far enough since the last one.
            if self.cursor >= self.next_station_at {
                let x = self.cursor + speed_at(self.cursor) * 0.9;
                self.stations.push(Station {
                    id: self.next_station_id,
                    x,
                    collected: false,
                });
                self.next_station_id += 1;
                self.cursor = x + STATION_W;
                self.next_station_at =
                    x + STATION_W + SPRINT_LEN + STATION_GAP + STATION_GAP_VARY * self.rng.unit();
                continue;
            }
            let v = self.planned_speed(self.cursor);
            let base = speed_at(self.cursor);
            // Room to land and read the next one: more than 3/4 of a second, then some.
            let gap_start = self.cursor;
            let start = self.cursor + v * (0.75 + 0.9 * self.rng.unit());
            // A line of darics in a long enough gap (after the clear start), jump height.
            let gap = start - gap_start;
            if gap_start > START_CLEAR && gap > v * 0.9 && self.rng.unit() < 0.55 {
                let count = 3 + (self.rng.unit() * 3.0) as u32; // 3..5
                let line = self.next_line_id;
                self.next_line_id += 1;
                let first = gap_start + gap * 0.35;
                let step = (DARIC_R * 2.4).min(gap * 0.12);
                for i in 0..count {
                    self.darics.push(Daric {
                        id: self.next_daric_id,
                        line,
                        x: first + i as f32 * step,
                        y: DARIC_Y,
                        collected: false,
                    });
                    self.next_daric_id += 1;
                }
            }
            let pick = self.rng.unit();
            let hard = start > HARD_FROM;
            let (kind, w, h) = if !hard || pick < 0.45 {
                (
                    Kind::Block,
                    36.0 + 40.0 * self.rng.unit(),
                    36.0 + 30.0 * self.rng.unit(),
                )
            } else if pick < 0.72 {
                (Kind::Gate, 44.0, GATE_CLEAR)
            } else {
                // Never wider than a held jump clears at the road's own speed (a sprint is
                // optional, so a rider who missed the token must cross it too).
                let widest = (0.5 * base - 120.0).max(0.0);
                (Kind::Pit, 120.0 + widest * self.rng.unit(), 0.0)
            };
            self.obstacles.push(Obstacle {
                id: self.next_id,
                kind,
                x: start,
                w,
                h,
            });
            self.next_id += 1;
            self.cursor = start + w;
        }
    }

    fn forget_behind(&mut self) {
        let from = self.x - KEEP_BEHIND;
        self.obstacles.retain(|o| o.x + o.w > from);
        self.stations.retain(|s| s.sprint_end() > from);
        self.darics.retain(|d| d.x + DARIC_R > from);
        self.near.retain(|(id, _)| self.obstacles.iter().any(|o| o.id == *id));
        self.grazed.retain(|id| self.obstacles.iter().any(|o| o.id == *id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOLD: Input = Input {
        jump_pressed: false,
        jump_held: true,
        duck_held: false,
    };
    const TAP: Input = Input {
        jump_pressed: true,
        jump_held: true,
        duck_held: false,
    };
    const DUCK: Input = Input {
        jump_pressed: false,
        jump_held: false,
        duck_held: true,
    };

    /// Runs `ticks` ticks of one input.
    fn run_for(run: &mut Run, input: Input, ticks: u32) {
        for _ in 0..ticks {
            run.step(input);
        }
    }

    /// Plays the run with a simple bot: look at the next obstacle and meet it with the move it
    /// needs. Returns the run when it ends or `max_ticks` pass.
    fn bot(seed: u64, max_ticks: u32, collect: bool) -> (Run, u32) {
        let mut run = Run::new(seed);
        let mut sprint_ticks = 0;
        for _ in 0..max_ticks {
            if run.is_over() {
                break;
            }
            if run.sprinting() {
                sprint_ticks += 1;
            }
            let ahead = run
                .obstacles()
                .iter()
                .find(|o| o.x + o.w > run.x() - 30.0)
                .copied();
            let mut input = Input::default();
            // A station token: take off so the jump peaks over it, and hold.
            let token = run
                .stations()
                .iter()
                .find(|st| !st.collected && st.token_x() + 40.0 > run.x())
                .copied();
            let mut at_token = false;
            if let (true, Some(st)) = (collect, token) {
                let gap = st.token_x() - run.x();
                if gap < 0.38 * run.speed() && gap > -30.0 {
                    at_token = true;
                    input.jump_held = true;
                    input.jump_pressed = run.grounded() && gap > 0.0;
                }
            }
            if at_token {
                run.step(input);
                continue;
            }
            if let Some(o) = ahead {
                let gap = o.x - run.x();
                let v = run.speed();
                match o.kind {
                    // Take off early enough for the jump to clear the block's far end.
                    Kind::Block | Kind::Pit => {
                        let lead = 0.20 * v + if o.kind == Kind::Pit { 0.0 } else { 0.10 * v };
                        if gap < lead && gap > -o.w * 0.5 && run.grounded() {
                            input.jump_pressed = true;
                        }
                        input.jump_held = true;
                    }
                    Kind::Gate => {
                        input.duck_held = gap < 160.0 && gap > -o.w - 60.0;
                    }
                }
            }
            run.step(input);
        }
        (run, sprint_ticks)
    }

    #[test]
    fn the_same_seed_makes_the_same_road() {
        let a = Run::new(7);
        let b = Run::new(7);
        let c = Run::new(8);
        assert_eq!(a.obstacles(), b.obstacles());
        assert_ne!(a.obstacles(), c.obstacles());
    }

    #[test]
    fn the_same_inputs_make_the_same_run() {
        let play = || {
            let mut run = Run::new(3);
            for t in 0..600u32 {
                run.step(Input {
                    jump_pressed: t % 70 == 0,
                    jump_held: t % 70 < 30,
                    duck_held: t % 110 > 80,
                });
            }
            (run.x(), run.y(), run.phase(), run.ticks())
        };
        assert_eq!(play(), play());
    }

    #[test]
    fn the_road_is_clear_at_the_start_and_blocks_only_at_first() {
        let run = Run::new(1);
        let first = run.obstacles()[0];
        assert!(first.x >= START_CLEAR);
        assert!(
            run.obstacles()
                .iter()
                .filter(|o| o.x < HARD_FROM)
                .all(|o| o.kind == Kind::Block)
        );
    }

    #[test]
    fn nothing_overlaps_and_each_has_room_to_land() {
        let run = Run::new(11);
        for pair in run.obstacles().windows(2) {
            let v = speed_at(pair[0].x + pair[0].w);
            assert!(pair[1].x - (pair[0].x + pair[0].w) >= 0.75 * v - 0.5);
        }
    }

    #[test]
    fn the_rider_speeds_up_to_a_cap() {
        assert_eq!(speed_at(0.0), SPEED_START);
        assert!(speed_at(10_000.0) > speed_at(0.0));
        assert_eq!(speed_at(1_000_000.0), SPEED_MAX);
    }

    #[test]
    fn a_held_jump_goes_higher_than_a_hop() {
        let apex = |hold: bool| {
            let mut run = Run::new(1);
            let mut best = 0.0f32;
            run.step(TAP);
            for _ in 0..90 {
                run.step(Input {
                    jump_held: hold,
                    ..Input::default()
                });
                best = best.max(run.y());
            }
            best
        };
        let (high, hop) = (apex(true), apex(false));
        assert!(high > 150.0, "held jump {high}");
        assert!(hop < 70.0 && hop > 20.0, "hop {hop}");
    }

    #[test]
    fn a_jump_pressed_a_moment_before_landing_is_kept() {
        let mut run = Run::new(1);
        run.step(TAP);
        // Fall back down, pressing once just before the road is reached, then letting go.
        let mut pressed = false;
        let mut jumped_again = false;
        for _ in 0..120 {
            let press = !pressed && run.y() < 12.0 && run.vy() < 0.0;
            pressed |= press;
            run.step(Input {
                jump_pressed: press,
                jump_held: false,
                duck_held: false,
            });
            if pressed && run.vy() > 0.0 {
                jumped_again = true;
                break;
            }
        }
        assert!(jumped_again);
    }

    #[test]
    fn a_block_ends_the_run_unless_jumped() {
        let mut run = Run::new(1);
        run_for(&mut run, Input::default(), 600);
        assert_eq!(run.phase(), Phase::Over(Cause::Block));
        // The run stays over.
        let x = run.x();
        run.step(TAP);
        assert_eq!(run.x(), x);
    }

    #[test]
    fn a_gate_is_passed_by_ducking_and_not_by_standing() {
        let mut run = Run {
            obstacles: vec![Obstacle {
                id: 0,
                kind: Kind::Gate,
                x: 200.0,
                w: 44.0,
                h: GATE_CLEAR,
            }],
            cursor: 1.0e9,
            ..Run::new(1)
        };
        run_for(&mut run, DUCK, 120);
        assert!(!run.is_over(), "ducked under the gate");
        assert!(run.x() > 244.0 + HIT_WIDTH);

        let mut standing = Run {
            obstacles: run.obstacles.clone(),
            cursor: 1.0e9,
            ..Run::new(1)
        };
        run_for(&mut standing, Input::default(), 120);
        assert_eq!(standing.phase(), Phase::Over(Cause::Gate));
    }

    #[test]
    fn a_pit_is_crossed_by_jumping_and_fallen_into_otherwise() {
        let pit = Obstacle {
            id: 0,
            kind: Kind::Pit,
            x: 300.0,
            w: 160.0,
            h: 0.0,
        };
        let fresh = || Run {
            obstacles: vec![pit],
            cursor: 1.0e9,
            ..Run::new(1)
        };
        let mut walked = fresh();
        run_for(&mut walked, Input::default(), 240);
        assert_eq!(walked.phase(), Phase::Over(Cause::Pit));

        let mut jumped = fresh();
        // Jump from the lip: the pit starts at 300, the rider covers 6.3 px a tick.
        while jumped.x() < 270.0 {
            jumped.step(Input::default());
        }
        jumped.step(TAP);
        run_for(&mut jumped, HOLD, 80);
        run_for(&mut jumped, Input::default(), 60);
        assert!(!jumped.is_over());
        assert!(jumped.grounded());
    }

    #[test]
    fn a_bot_that_plays_every_move_runs_a_long_way() {
        // The road is fair: every obstacle can be passed with the move it names. Three seeds,
        // 90 s each, and the bot is never stopped by the road.
        for seed in [1u64, 2, 3] {
            let (run, _) = bot(seed, 90 * TICKS_PER_SECOND, false);
            assert!(
                !run.is_over(),
                "seed {seed} ended at {} m: {:?}",
                run.metres(),
                run.phase()
            );
            assert!(run.metres() > 1000, "seed {seed}: {} m", run.metres());
        }
    }

    #[test]
    fn old_obstacles_are_forgotten() {
        let (run, _) = bot(5, 40 * TICKS_PER_SECOND, false);
        assert!(run.obstacles().len() < 40);
    }

    #[test]
    fn stations_come_every_few_thousand_pixels_after_the_first() {
        let (run, _) = bot(4, 120 * TICKS_PER_SECOND, true);
        assert!(run.metres() > 1500, "{} m", run.metres());
        let mut run = Run::new(4);
        // Look far ahead by moving the rider (no input needed to read the road).
        run.x = 12_000.0;
        run.lay_road();
        let xs: Vec<f32> = run.stations().iter().map(|s| s.x).collect();
        assert!(xs.len() >= 2, "{xs:?}");
        for pair in xs.windows(2) {
            let gap = pair[1] - pair[0];
            assert!(gap > STATION_W + SPRINT_LEN + STATION_GAP - 1.0, "{gap}");
        }
        // Nothing stands inside a station.
        for st in run.stations() {
            assert!(
                !run.obstacles()
                    .iter()
                    .any(|o| o.x < st.x + STATION_W && o.x + o.w > st.x)
            );
        }
    }

    #[test]
    fn a_held_jump_takes_the_token_and_a_hop_does_not() {
        let try_jump = |hold: bool| {
            let mut run = Run::new(2);
            run.obstacles.clear();
            run.cursor = 1.0e9;
            run.stations = vec![Station {
                id: 0,
                x: 1000.0,
                collected: false,
            }];
            let mut jumped = false;
            for _ in 0..400 {
                let gap = run.stations[0].token_x() - run.x();
                let go = !jumped && gap < 0.38 * run.speed();
                jumped |= go;
                run.step(Input {
                    jump_pressed: go,
                    jump_held: hold && jumped,
                    duck_held: false,
                });
            }
            run
        };
        assert!(try_jump(true).stations()[0].collected);
        assert!(!try_jump(false).stations()[0].collected);
    }

    #[test]
    fn a_taken_token_makes_the_sprint_faster_and_it_ends() {
        let mut run = Run::new(2);
        run.obstacles.clear();
        run.cursor = 1.0e9;
        run.stations = vec![Station {
            id: 0,
            x: 1000.0,
            collected: true,
        }];
        let base_before = run.speed();
        let mut sprint_speed = 0.0;
        let mut after = 0.0;
        for _ in 0..1200 {
            run.step(Input::default());
            if run.sprinting() {
                sprint_speed = run.speed();
            } else if run.x() > 1000.0 + STATION_W + SPRINT_LEN {
                after = run.speed();
                break;
            }
        }
        assert!(sprint_speed > base_before * 1.4, "{sprint_speed} vs {base_before}");
        assert!(after < sprint_speed / 1.3, "{after}");
    }

    #[test]
    fn a_bot_that_takes_every_token_survives_the_sprints_too() {
        for seed in [1u64, 2, 3, 6] {
            let (run, sprint_ticks) = bot(seed, 120 * TICKS_PER_SECOND, true);
            assert!(sprint_ticks > 60, "seed {seed} never sprinted");
            assert!(
                !run.is_over(),
                "seed {seed} ended at {} m: {:?}",
                run.metres(),
                run.phase()
            );
            assert!(
                run.metres() > 1500,
                "seed {seed}: {} m",
                run.metres()
            );
        }
    }

    #[test]
    fn metres_raise_the_score_and_a_missed_daric_resets_the_multiplier() {
        let mut run = Run::new(1);
        run.obstacles.clear();
        run.darics.clear();
        run.cursor = 1.0e9;
        run_for(&mut run, Input::default(), 120);
        assert!(run.score() >= run.metres() as i64);
        assert_eq!(run.multiplier(), 1);

        // A daric already behind the rider (high enough that a standing body never took it).
        run.multiplier = 4;
        run.chain = 4;
        run.darics = vec![Daric {
            id: 0,
            line: 0,
            x: run.x() - 80.0,
            y: STAND_HEIGHT + 40.0,
            collected: false,
        }];
        run.step(Input::default());
        assert_eq!(run.multiplier(), 1);
        assert_eq!(run.chain, 0);
        assert_eq!(run.darics_taken(), 0);
    }

    #[test]
    fn collecting_darics_grows_the_multiplier_and_the_score() {
        let mut run = Run::new(1);
        run.obstacles.clear();
        run.darics.clear();
        run.cursor = 1.0e9;
        let base_x = 400.0;
        run.darics = (0..3)
            .map(|i| Daric {
                id: i,
                line: 0,
                x: base_x + i as f32 * DARIC_R * 2.4,
                y: 20.0, // on the road so a run-through takes them
                collected: false,
            })
            .collect();
        while run.x() < base_x + 200.0 && !run.is_over() {
            run.step(Input::default());
        }
        assert_eq!(run.darics_taken(), 3);
        assert!(run.multiplier() >= 3);
        assert!(run.score() > run.metres() as i64);
    }
}
