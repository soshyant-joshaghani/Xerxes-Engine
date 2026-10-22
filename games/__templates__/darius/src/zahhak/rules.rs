//! Zahhak's rules, with no Bevy: serpents on one round field, eating pellets, dying on the wall
//! or on a body. The Bevy side (`play.rs`) steers, draws and reports the changes to the match.
//!
//! Units: world pixels, and a fixed step of one tick (1/60 s). A serpent's length is in units:
//! its path is `units * UNIT` long, and it grows by one unit per pellet.

use std::collections::VecDeque;

/// Radius of the round field; the wall is at this distance from the centre.
pub const FIELD_RADIUS: f32 = 215.0;
/// Length of a new serpent, in units.
pub const START_UNITS: u32 = 10;
/// Pellets kept on the field (a pellet eaten is replaced at once).
pub const PELLETS: usize = 90;
/// Most pellets on the field at once: a dead serpent's food is added to the count, so the oldest
/// food goes first when the field is over this.
pub const MAX_PELLETS: usize = 2 * PELLETS;

/// Path length per unit of length.
pub const UNIT: f32 = 6.0;
/// Distance between the path samples.
const SAMPLE: f32 = 4.0;
/// How far a serpent moves per tick (110 px a second).
const SPEED: f32 = 110.0 / 60.0;
/// The most a serpent turns per tick (about 170 degrees a second).
const TURN: f32 = 3.0 / 60.0;
const HEAD_R: f32 = 6.0;
const BODY_R: f32 = 4.5;
const PELLET_R: f32 = 6.0;
/// Path samples at the head that do not count as the serpent's own body.
const NECK: usize = 6;
/// Speed multiplier while boosting.
pub const BOOST_SPEED: f32 = 1.8;
/// Ticks of boosting per unit of length it costs (0.3 s).
pub const BOOST_COST_TICKS: u32 = 18;
/// Clear space a respawning serpent needs from every other serpent's body.
const CLEAR: f32 = 40.0;
/// Path samples between the pellets a dead serpent leaves behind.
const FOOD_SPACING: usize = 6;

/// A point of the field.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn dist(self, other: Point) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    fn len(self) -> f32 {
        self.dist(Point::default())
    }
}

/// One serpent. Its path runs from the tail (first) to the head (last).
#[derive(Debug, Clone, PartialEq)]
pub struct Serpent {
    pub heading: f32,
    pub path: VecDeque<Point>,
    pub units: u32,
    pub alive: bool,
    /// Boost is held (it works only above the start length).
    pub boosting: bool,
    /// Ticks boosted since the last unit it cost.
    pub boost_timer: u32,
}

impl Serpent {
    pub fn head(&self) -> Point {
        *self.path.back().expect("a serpent always has a head")
    }

    /// The length above the start: what the match scores for this serpent.
    pub fn score(&self) -> i64 {
        i64::from(self.units) - i64::from(START_UNITS)
    }
}

/// What happened on a tick, for the match and the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The serpent ate a pellet: one unit longer.
    Ate { seat: usize },
    /// Boosting cost one unit of length (its tail is left behind as food).
    Shed { seat: usize },
    /// The serpent died: its length above the start is lost (`score`, negative or zero), and
    /// its body is left as food.
    Died { seat: usize, score: i64 },
}

/// The whole field: every serpent (one per seat) and the pellets, with its own seeded generator.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub serpents: Vec<Serpent>,
    pub pellets: Vec<Point>,
    rng: u64,
}

impl Field {
    /// A field with `seats` serpents on a ring, each starting in its own lane toward the centre,
    /// and the pellets laid out from `seed`.
    pub fn new(seats: usize, seed: u64) -> Self {
        let mut field = Self {
            serpents: Vec::with_capacity(seats),
            pellets: Vec::with_capacity(PELLETS),
            rng: seed,
        };
        for seat in 0..seats {
            field.serpents.push(field.fresh(seat, seats));
        }
        while field.pellets.len() < PELLETS {
            let p = field.random_point(0.95);
            field.pellets.push(p);
        }
        field
    }

    /// Splitmix64, a small well-mixed generator with no platform dependence.
    fn next_u64(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn random_unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// A random point within `fraction` of the field's radius.
    fn random_point(&mut self, fraction: f32) -> Point {
        let angle = self.random_unit() * std::f32::consts::TAU;
        let r = self.random_unit().sqrt() * FIELD_RADIUS * fraction;
        Point::new(r * angle.cos(), r * angle.sin())
    }

    /// A serpent of the start length on its ring slot, aimed inward without converging on the
    /// other starting serpents.
    fn fresh(&self, seat: usize, seats: usize) -> Serpent {
        let angle = seat as f32 / seats.max(1) as f32 * std::f32::consts::TAU;
        let head = Point::new(
            0.6 * FIELD_RADIUS * angle.cos(),
            0.6 * FIELD_RADIUS * angle.sin(),
        );
        Self::serpent_at(head, Self::start_heading(angle))
    }

    fn start_heading(slot_angle: f32) -> f32 {
        slot_angle + std::f32::consts::PI - std::f32::consts::FRAC_PI_4
    }

    /// A serpent of the start length with its head at `head` and the given heading.
    fn serpent_at(head: Point, heading: f32) -> Serpent {
        let mut path = VecDeque::new();
        let length = START_UNITS as f32 * UNIT;
        let steps = (length / SAMPLE).ceil() as usize;
        for i in (0..=steps).rev() {
            let back = i as f32 * SAMPLE;
            path.push_back(Point::new(
                head.x - heading.cos() * back,
                head.y - heading.sin() * back,
            ));
        }
        Serpent {
            heading,
            path,
            units: START_UNITS,
            alive: true,
            boosting: false,
            boost_timer: 0,
        }
    }

    /// The serpent of `seat`.
    pub fn serpent(&self, seat: usize) -> &Serpent {
        &self.serpents[seat]
    }

    /// Brings a dead serpent back at its start: on its ring slot, or the nearest slot with clear
    /// space around it when the ring slot is taken.
    pub fn respawn(&mut self, seat: usize) {
        let seats = self.serpents.len();
        let base = seat as f32 / seats.max(1) as f32 * std::f32::consts::TAU;
        let ring = |angle: f32| {
            Point::new(
                0.6 * FIELD_RADIUS * angle.cos(),
                0.6 * FIELD_RADIUS * angle.sin(),
            )
        };
        let mut head = ring(base);
        let mut slot_angle = base;
        for k in 0..=12 {
            // 0, +a, -a, +2a, -2a ... around the ring slot.
            let step = (k as f32 / 2.0).ceil() * 0.3 * if k % 2 == 1 { 1.0 } else { -1.0 };
            let candidate = ring(base + step);
            if self.clear(candidate, seat) {
                head = candidate;
                slot_angle = base + step;
                break;
            }
        }
        self.serpents[seat] = Self::serpent_at(head, Self::start_heading(slot_angle));
    }

    /// Whether a serpent could stand at `head`: inside the wall and clear of every other body.
    fn clear(&self, head: Point, seat: usize) -> bool {
        if head.len() > FIELD_RADIUS - 40.0 {
            return false;
        }
        self.serpents.iter().enumerate().all(|(other, o)| {
            other == seat || !o.alive || o.path.iter().all(|p| head.dist(*p) >= CLEAR)
        })
    }

    /// Turns boost on or off for a seat (it has no effect on a serpent at the start length).
    pub fn set_boost(&mut self, seat: usize, on: bool) {
        if let Some(s) = self.serpents.get_mut(seat) {
            s.boosting = on;
            if !on {
                s.boost_timer = 0;
            }
        }
    }

    /// The heading a bot steers for: toward the nearest pellet, or back to the centre near the wall.
    pub fn bot_heading(&self, seat: usize) -> f32 {
        let s = &self.serpents[seat];
        let head = s.head();
        if head.len() > FIELD_RADIUS - 45.0 {
            return (-head.y).atan2(-head.x);
        }
        let nearest = self
            .pellets
            .iter()
            .min_by(|a, b| head.dist(**a).total_cmp(&head.dist(**b)));
        match nearest {
            Some(p) => (p.y - head.y).atan2(p.x - head.x),
            None => s.heading,
        }
    }

    /// One fixed tick: each live serpent turns toward its heading (at most `TURN`), moves, eats
    /// the pellets it touches, and dies on the wall or on a body. `headings` is one per seat.
    pub fn step(&mut self, headings: &[f32]) -> Vec<Event> {
        let mut events = Vec::new();
        for (seat, s) in self.serpents.iter_mut().enumerate() {
            if !s.alive {
                continue;
            }
            let wanted = headings.get(seat).copied().unwrap_or(s.heading);
            s.heading += turn_diff(s.heading, wanted).clamp(-TURN, TURN);
            let boosting = s.boosting && s.units > START_UNITS;
            let speed = if boosting { SPEED * BOOST_SPEED } else { SPEED };
            let head = s.head();
            s.path.push_back(Point::new(
                head.x + s.heading.cos() * speed,
                head.y + s.heading.sin() * speed,
            ));
            // Boosting costs a unit of length every so often; its tail is left behind as food.
            if boosting {
                s.boost_timer += 1;
                if s.boost_timer >= BOOST_COST_TICKS {
                    s.boost_timer = 0;
                    s.units -= 1;
                    if let Some(tail) = s.path.front().copied() {
                        self.pellets.push(tail);
                    }
                    events.push(Event::Shed { seat });
                }
            } else {
                s.boost_timer = 0;
            }
            // Keep the path at the length of the serpent: the tail falls off as the head moves.
            let max = s.units as f32 * UNIT;
            let mut total = path_len(&s.path);
            while s.path.len() > 2 && total > max {
                total -= s.path[0].dist(s.path[1]);
                s.path.pop_front();
            }
        }

        // Pellets: a serpent that reaches one eats it and a new one is laid somewhere on the field.
        for seat in 0..self.serpents.len() {
            if !self.serpents[seat].alive {
                continue;
            }
            let head = self.serpents[seat].head();
            if let Some(i) = self
                .pellets
                .iter()
                .position(|p| head.dist(*p) < HEAD_R + PELLET_R)
            {
                self.pellets.swap_remove(i);
                self.serpents[seat].units += 1;
                let p = self.random_point(0.95);
                self.pellets.push(p);
                events.push(Event::Ate { seat });
            }
        }

        // Deaths: the wall, another serpent's body, and a head-on meeting (the shorter one dies,
        // both if they are the same length).
        let mut dying = vec![false; self.serpents.len()];
        for seat in 0..self.serpents.len() {
            let s = &self.serpents[seat];
            if !s.alive {
                continue;
            }
            let head = s.head();
            if head.len() > FIELD_RADIUS {
                dying[seat] = true;
                continue;
            }
            for (other, o) in self.serpents.iter().enumerate() {
                // Your own body is safe to touch. Only another serpent's body kills a head, whatever
                // the sizes: the bigger one or the smaller one alike.
                if other == seat || !o.alive {
                    continue;
                }
                // The neck (the last samples of a path, the head's own) is not body: a head meeting
                // another's head is decided below.
                let body = o.path.len().saturating_sub(NECK);
                let hit = o
                    .path
                    .iter()
                    .take(body)
                    .any(|p| head.dist(*p) < HEAD_R + BODY_R);
                if hit {
                    dying[seat] = true;
                    break;
                }
                if other != seat && o.head().dist(head) < HEAD_R + HEAD_R {
                    let (a, b) = (self.serpents[seat].units, o.units);
                    if a <= b {
                        dying[seat] = true;
                    }
                }
            }
        }
        for (seat, dies) in dying.into_iter().enumerate() {
            if dies {
                events.push(self.die(seat));
            }
        }
        events
    }

    /// Ends a serpent: its body is left as food, and its length above the start is lost.
    fn die(&mut self, seat: usize) -> Event {
        let s = &mut self.serpents[seat];
        s.alive = false;
        let score = -s.score().max(0);
        for (i, p) in s.path.iter().enumerate() {
            if i % FOOD_SPACING == 0 {
                self.pellets.push(*p);
            }
        }
        let excess = self.pellets.len().saturating_sub(MAX_PELLETS);
        self.pellets.drain(..excess);
        Event::Died { seat, score }
    }
}

/// The length of a path (its polyline).
fn path_len(path: &VecDeque<Point>) -> f32 {
    path.iter()
        .zip(path.iter().skip(1))
        .map(|(a, b)| a.dist(*b))
        .sum()
}

/// The signed turn from `from` to `to`, in (-pi, pi].
fn turn_diff(from: f32, to: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut d = (to - from).rem_euclid(tau);
    if d > std::f32::consts::PI {
        d -= tau;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(seats: usize) -> Field {
        Field::new(seats, 7)
    }

    /// Puts a serpent's head at `at`, heading `heading`, with a straight body behind it.
    fn place(f: &mut Field, seat: usize, at: Point, heading: f32, units: u32) {
        let s = &mut f.serpents[seat];
        s.heading = heading;
        s.units = units;
        s.alive = true;
        s.path.clear();
        let length = units as f32 * UNIT;
        let steps = (length / SAMPLE).ceil() as usize;
        for i in (0..=steps).rev() {
            let back = i as f32 * SAMPLE;
            s.path.push_back(Point::new(
                at.x - heading.cos() * back,
                at.y - heading.sin() * back,
            ));
        }
    }

    #[test]
    fn serpents_start_at_length_ten_in_separate_inward_lanes() {
        let f = field(4);
        for seat in 0..4 {
            let s = f.serpent(seat);
            assert_eq!(s.units, START_UNITS);
            assert!(s.alive);
            assert!(s.head().len() < FIELD_RADIUS * 0.7);
            let slot_angle = seat as f32 / 4.0 * std::f32::consts::TAU;
            assert!(turn_diff(s.heading, Field::start_heading(slot_angle)).abs() < 1e-4);
        }
    }

    #[test]
    fn a_four_serpent_opening_does_not_collapse_into_a_center_collision() {
        let mut f = field(4);
        let headings: Vec<f32> = f.serpents.iter().map(|s| s.heading).collect();

        for _ in 0..60 {
            f.step(&headings);
        }

        assert!(
            f.serpents.iter().all(|s| s.alive),
            "the opening lanes should give the player time to recognize and steer"
        );
    }

    #[test]
    fn a_serpent_turns_at_a_limited_rate() {
        let mut f = field(1);
        place(&mut f, 0, Point::default(), 0.0, START_UNITS);
        f.step(&[std::f32::consts::FRAC_PI_2]);
        let turned = f.serpent(0).heading;
        assert!(turned > 0.0, "it turns toward the wanted heading");
        assert!(
            turned <= TURN + 1e-6,
            "but no faster than TURN a tick: {turned}"
        );
    }

    #[test]
    fn a_serpent_moves_along_its_heading() {
        let mut f = field(1);
        place(&mut f, 0, Point::default(), 0.0, START_UNITS);
        let before = f.serpent(0).head();
        f.step(&[0.0]);
        let after = f.serpent(0).head();
        assert!((after.x - before.x - SPEED).abs() < 1e-4);
        assert!((after.y - before.y).abs() < 1e-4);
    }

    #[test]
    fn eating_a_pellet_grows_the_serpent_and_the_field_keeps_its_pellets() {
        let mut f = field(1);
        place(&mut f, 0, Point::new(0.0, 0.0), 0.0, START_UNITS);
        f.pellets.clear();
        f.pellets.push(Point::new(3.0, 0.0));
        f.pellets
            .extend((0..PELLETS - 1).map(|i| Point::new(150.0, i as f32 * 0.1 - 40.0)));
        let events = f.step(&[0.0]);
        assert_eq!(events, vec![Event::Ate { seat: 0 }]);
        assert_eq!(f.serpent(0).units, START_UNITS + 1);
        assert_eq!(f.pellets.len(), PELLETS, "the eaten pellet is replaced");
    }

    #[test]
    fn the_wall_kills_and_the_length_above_the_start_is_lost() {
        let mut f = field(1);
        place(
            &mut f,
            0,
            Point::new(FIELD_RADIUS - 1.0, 0.0),
            0.0,
            START_UNITS + 14,
        );
        let events = f.step(&[0.0]);
        assert_eq!(
            events,
            vec![Event::Died {
                seat: 0,
                score: -14
            }]
        );
        assert!(!f.serpent(0).alive);
    }

    #[test]
    fn dying_at_the_start_length_costs_nothing() {
        let mut f = field(1);
        place(
            &mut f,
            0,
            Point::new(FIELD_RADIUS - 1.0, 0.0),
            0.0,
            START_UNITS,
        );
        assert_eq!(f.step(&[0.0]), vec![Event::Died { seat: 0, score: 0 }]);
    }

    #[test]
    fn a_body_kills_the_serpent_that_runs_into_it() {
        let mut f = field(2);
        // Seat 1 lies across the field (its body runs along y = 0); seat 0 drives up into it.
        place(&mut f, 1, Point::new(0.0, 0.0), 0.0, START_UNITS);
        place(
            &mut f,
            0,
            Point::new(-30.0, -20.0),
            std::f32::consts::FRAC_PI_2,
            START_UNITS,
        );
        let mut died = Vec::new();
        for _ in 0..30 {
            for event in f.step(&[std::f32::consts::FRAC_PI_2, 0.0]) {
                died.push(event);
            }
        }
        assert!(died.contains(&Event::Died { seat: 0, score: 0 }));
        assert!(f.serpent(1).alive, "the body does not die");
    }

    #[test]
    fn a_head_touching_its_own_body_is_safe() {
        let mut f = field(1);
        // A body that runs back past the head: the head is 5 px from its own tail (index 0).
        let s = &mut f.serpents[0];
        s.heading = std::f32::consts::FRAC_PI_2;
        s.units = 30;
        s.path = VecDeque::from(vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 10.0),
            Point::new(0.0, 20.0),
            Point::new(0.0, 30.0),
            Point::new(0.0, 40.0),
            Point::new(0.0, 50.0),
            Point::new(0.0, 60.0),
            Point::new(0.0, 70.0),
            Point::new(3.0, 5.0),
        ]);
        f.pellets.clear();
        assert!(f.step(&[std::f32::consts::FRAC_PI_2]).is_empty());
        assert!(f.serpent(0).alive, "its own body does not kill it");
    }

    #[test]
    fn a_head_into_a_body_dies_whatever_the_sizes() {
        // Seat 1 lies across the field (its body runs along y = 0); seat 0 drives up into it. The
        // rule is the same whether the body belongs to a bigger serpent or a smaller one.
        for (small, big) in [
            (START_UNITS, START_UNITS + 10),
            (START_UNITS + 10, START_UNITS),
        ] {
            let mut f = field(2);
            place(&mut f, 1, Point::new(0.0, 0.0), 0.0, big);
            place(
                &mut f,
                0,
                Point::new(-30.0, -20.0),
                std::f32::consts::FRAC_PI_2,
                small,
            );
            let mut died = false;
            for _ in 0..30 {
                for event in f.step(&[std::f32::consts::FRAC_PI_2, 0.0]) {
                    died |= matches!(event, Event::Died { seat: 0, .. });
                }
            }
            assert!(
                died,
                "a head into a body of {big} units kills, from {small}"
            );
            assert!(f.serpent(1).alive, "the body does not die");
        }
    }

    #[test]
    fn a_head_on_meeting_kills_the_shorter_serpent() {
        let mut f = field(2);
        place(&mut f, 0, Point::new(-5.0, 0.0), 0.0, START_UNITS + 5);
        place(
            &mut f,
            1,
            Point::new(5.0, 0.0),
            std::f32::consts::PI,
            START_UNITS,
        );
        let events = f.step(&[0.0, std::f32::consts::PI]);
        assert!(f.serpent(0).alive, "the longer one lives");
        assert!(!f.serpent(1).alive);
        assert!(events.contains(&Event::Died { seat: 1, score: 0 }));
    }

    #[test]
    fn a_death_leaves_food_and_a_respawn_brings_the_serpent_back() {
        let mut f = field(1);
        place(
            &mut f,
            0,
            Point::new(FIELD_RADIUS - 1.0, 0.0),
            0.0,
            START_UNITS + 4,
        );
        let before = f.pellets.len();
        f.step(&[0.0]);
        assert!(f.pellets.len() > before, "its body is food");
        f.respawn(0);
        let s = f.serpent(0);
        assert!(s.alive);
        assert_eq!(s.units, START_UNITS);
    }

    #[test]
    fn sixteen_serpents_start_apart_and_play_on_with_bots() {
        let mut f = Field::new(16, 5);
        // No serpent starts inside another's body.
        for seat in 0..16 {
            for other in 0..16 {
                if other == seat {
                    continue;
                }
                let s = f.serpent(seat).head();
                for p in f.serpent(other).path.iter() {
                    assert!(
                        s.dist(*p) >= HEAD_R + BODY_R,
                        "seat {seat} starts on seat {other}"
                    );
                }
            }
        }
        let mut eaten = 0;
        let mut died = 0;
        for _ in 0..(60 * 60) {
            let headings: Vec<f32> = (0..16).map(|seat| f.bot_heading(seat)).collect();
            for event in f.step(&headings) {
                match event {
                    Event::Ate { .. } => eaten += 1,
                    Event::Shed { .. } => {}
                    Event::Died { seat, .. } => {
                        died += 1;
                        f.respawn(seat);
                    }
                }
            }
        }
        assert!(eaten > 100, "sixteen bots eat: {eaten}");
        assert!(
            died < eaten,
            "most of the time they live: {died} deaths, {eaten} meals"
        );
        assert!(f.serpents.iter().filter(|s| s.alive).count() >= 8);
    }

    #[test]
    fn the_food_on_the_field_stays_bounded() {
        // Every death leaves food; over a long match the count must not keep climbing.
        let mut f = field(4);
        for round in 0..200 {
            let seat = round % 4;
            place(
                &mut f,
                seat,
                Point::new(FIELD_RADIUS - 1.0, 0.0),
                0.0,
                START_UNITS + 30,
            );
            f.step(&[0.0, 0.0, 0.0, 0.0]);
            f.respawn(seat);
            assert!(
                f.pellets.len() <= MAX_PELLETS,
                "{} pellets",
                f.pellets.len()
            );
        }
    }

    #[test]
    fn boosting_costs_a_unit_every_so_often_and_leaves_food() {
        let mut f = field(1);
        place(&mut f, 0, Point::new(0.0, 0.0), 0.0, START_UNITS + 5);
        f.pellets.clear();
        f.set_boost(0, true);
        let mut shed = 0;
        for _ in 0..BOOST_COST_TICKS * 2 {
            shed += f
                .step(&[0.0])
                .iter()
                .filter(|e| matches!(e, Event::Shed { .. }))
                .count();
        }
        assert_eq!(shed, 2);
        assert_eq!(f.serpent(0).units, START_UNITS + 3);
        assert_eq!(f.pellets.len(), 2, "each unit lost is food");
    }

    #[test]
    fn boost_is_faster_and_stops_at_the_start_length() {
        let mut f = field(1);
        place(&mut f, 0, Point::new(0.0, 0.0), 0.0, START_UNITS);
        f.pellets.clear();
        f.set_boost(0, true);
        let before = f.serpent(0).head().x;
        let mut shed = 0;
        for _ in 0..BOOST_COST_TICKS * 3 {
            shed += f
                .step(&[0.0])
                .iter()
                .filter(|e| matches!(e, Event::Shed { .. }))
                .count();
        }
        assert_eq!(shed, 0, "no cost at the start length");
        assert_eq!(f.serpent(0).units, START_UNITS);
        let moved = f.serpent(0).head().x - before;
        let expected = SPEED * (BOOST_COST_TICKS * 3) as f32;
        assert!(
            (moved - expected).abs() < 0.5,
            "at the start length boost does nothing: moved {moved}"
        );
    }

    #[test]
    fn respawn_finds_a_clear_slot_when_the_ring_slot_is_taken() {
        let mut f = field(2);
        // Seat 1 lies across seat 0's ring slot (angle 0, head at (129, 0)).
        place(&mut f, 1, Point::new(129.0, 0.0), 0.0, START_UNITS);
        f.serpents[0].alive = false;
        f.respawn(0);
        let s = f.serpent(0);
        assert!(s.alive);
        assert_eq!(s.units, START_UNITS);
        for p in f.serpent(1).path.iter() {
            assert!(s.head().dist(*p) >= CLEAR, "clear of the other body");
        }
    }

    #[test]
    fn bots_eat_and_the_same_seed_plays_the_same_field() {
        let run = || {
            let mut f = Field::new(4, 11);
            let mut eaten = 0;
            for _ in 0..(60 * 45) {
                let headings: Vec<f32> = (0..4).map(|seat| f.bot_heading(seat)).collect();
                for event in f.step(&headings) {
                    match event {
                        Event::Ate { .. } => eaten += 1,
                        Event::Shed { .. } => {}
                        Event::Died { seat, .. } => f.respawn(seat),
                    }
                }
            }
            (eaten, f)
        };
        let (eaten, first) = run();
        assert!(eaten > 20, "bots found food: {eaten}");
        let (_, second) = run();
        assert_eq!(first, second);
    }

    #[test]
    fn the_bot_turns_back_from_the_wall() {
        let mut f = field(1);
        place(
            &mut f,
            0,
            Point::new(FIELD_RADIUS - 5.0, 0.0),
            0.0,
            START_UNITS,
        );
        let heading = f.bot_heading(0);
        // Back toward the centre: the heading points at -x.
        assert!(heading.cos() < 0.0);
    }
}
