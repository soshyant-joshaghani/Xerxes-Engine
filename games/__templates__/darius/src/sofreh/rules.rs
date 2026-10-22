//! Sofreh's rules, with no Bevy: one shared kitchen, two cooks (one per team), one dish (a kebab:
//! skewer, grill, plate), delivered at your own hatch, and the clock. Slice 1 of the GDD.
//!
//! The kitchen is a grid. A cook moves one cell a tick; a move into a station or a wall only turns
//! the cook to face it. Interact acts on the cell the cook faces.

/// Grid size (columns, rows).
pub const W: usize = 9;
pub const H: usize = 5;
/// Ticks the grill takes to cook a kebab (3 s).
pub const COOK_TICKS: u32 = 3 * 60;

/// The kitchen map, top row first. `.` floor, `#` wall, `P` pantry (skewers), `G` grill (shared),
/// `C` plates (shared), `A` and `B` the hatches of team 0 and team 1.
const MAP: [&str; H] = [
    "##P.G.C##",
    "A.......B",
    "A.......B",
    "A.......B",
    "##.....##",
];

/// What a cell is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Floor,
    Wall,
    Pantry,
    Grill,
    Plates,
    /// The serving hatch of a team.
    Hatch(u8),
}

impl Cell {
    fn walkable(self) -> bool {
        self == Cell::Floor
    }
}

/// What a cook holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Held {
    Nothing,
    Skewer,
    /// A cooked kebab, not yet plated.
    Cooked,
    /// A cooked kebab on a plate: the dish to deliver.
    Plated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    fn delta(self) -> (isize, isize) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }
}

/// One tick of one cook's input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Move(Dir),
    Interact,
}

/// One cook: a cell, the item held, the way it faces, and its team.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cook {
    pub x: usize,
    pub y: usize,
    pub team: u8,
    pub held: Held,
    pub facing: Dir,
}

/// What happened on a tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A team's dish reached its own hatch.
    Delivered { team: u8 },
}

/// The kitchen: the cooks, the shared grill, the dishes each team delivered and the clock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kitchen {
    pub cooks: Vec<Cook>,
    /// The grill's kebab, if one is on it: `Some(ticks cooked)`.
    pub grill: Option<u32>,
    /// Dishes delivered, per team.
    pub delivered: [u32; 2],
    pub ticks: u64,
    /// The time limit in ticks.
    pub limit: u64,
}

/// The cell at (x, y) of the map.
pub fn cell(x: usize, y: usize) -> Cell {
    match MAP[y].as_bytes()[x] {
        b'.' => Cell::Floor,
        b'P' => Cell::Pantry,
        b'G' => Cell::Grill,
        b'C' => Cell::Plates,
        b'A' => Cell::Hatch(0),
        b'B' => Cell::Hatch(1),
        _ => Cell::Wall,
    }
}

impl Kitchen {
    /// A kitchen with team 0's cook at (1, 1) and team 1's at (7, 1), and `limit` ticks on the clock.
    pub fn new(limit: u64) -> Self {
        Self {
            cooks: vec![
                Cook {
                    x: 1,
                    y: 1,
                    team: 0,
                    held: Held::Nothing,
                    facing: Dir::Right,
                },
                Cook {
                    x: 7,
                    y: 1,
                    team: 1,
                    held: Held::Nothing,
                    facing: Dir::Left,
                },
            ],
            grill: None,
            delivered: [0, 0],
            ticks: 0,
            limit,
        }
    }

    /// Whether the clock has run out and one team is ahead. A tie plays on to the next delivery.
    pub fn over(&self) -> bool {
        self.ticks >= self.limit && self.delivered[0] != self.delivered[1]
    }

    /// The winning team, once the match is over.
    pub fn winner(&self) -> Option<u8> {
        if !self.over() {
            return None;
        }
        Some(if self.delivered[0] > self.delivered[1] {
            0
        } else {
            1
        })
    }

    /// Whether a cell can be stepped on: a floor cell with no cook on it.
    fn free(&self, x: usize, y: usize) -> bool {
        cell(x, y).walkable() && !self.cooks.iter().any(|c| c.x == x && c.y == y)
    }

    /// One tick: each cook with an input acts (in cook order), the grill cooks, and the clock runs.
    /// `inputs` has one entry per cook; `None` is no input. Nothing moves once the match is over.
    pub fn step(&mut self, inputs: &[Option<Input>]) -> Vec<Event> {
        let mut events = Vec::new();
        if self.over() {
            return events;
        }
        for i in 0..self.cooks.len() {
            let Some(input) = inputs.get(i).copied().flatten() else {
                continue;
            };
            match input {
                Input::Move(dir) => {
                    let c = self.cooks[i];
                    let (dx, dy) = dir.delta();
                    let (nx, ny) = (c.x as isize + dx, c.y as isize + dy);
                    self.cooks[i].facing = dir;
                    if nx >= 0 && ny >= 0 && (nx as usize) < W && (ny as usize) < H {
                        let (nx, ny) = (nx as usize, ny as usize);
                        if self.free(nx, ny) {
                            self.cooks[i].x = nx;
                            self.cooks[i].y = ny;
                        }
                    }
                }
                Input::Interact => {
                    if let Some(event) = self.interact(i) {
                        events.push(event);
                    }
                }
            }
        }
        if let Some(t) = self.grill.as_mut() {
            *t = (*t + 1).min(COOK_TICKS);
        }
        self.ticks += 1;
        events
    }

    /// Interact with the cell cook `i` faces.
    fn interact(&mut self, i: usize) -> Option<Event> {
        let c = self.cooks[i];
        let (dx, dy) = c.facing.delta();
        let (x, y) = (c.x as isize + dx, c.y as isize + dy);
        if x < 0 || y < 0 || x as usize >= W || y as usize >= H {
            return None;
        }
        let (x, y) = (x as usize, y as usize);
        match cell(x, y) {
            Cell::Pantry if c.held == Held::Nothing => {
                self.cooks[i].held = Held::Skewer;
            }
            Cell::Grill => {
                if c.held == Held::Skewer && self.grill.is_none() {
                    self.grill = Some(0);
                    self.cooks[i].held = Held::Nothing;
                } else if c.held == Held::Nothing && self.grill.is_some_and(|t| t >= COOK_TICKS) {
                    self.grill = None;
                    self.cooks[i].held = Held::Cooked;
                }
            }
            Cell::Plates if c.held == Held::Cooked => {
                self.cooks[i].held = Held::Plated;
            }
            Cell::Hatch(team) if c.held == Held::Plated && team == c.team => {
                self.cooks[i].held = Held::Nothing;
                self.delivered[team as usize] += 1;
                return Some(Event::Delivered { team });
            }
            _ => {}
        }
        None
    }

    /// The next step of the shortest walk from cook `i` to `(tx, ty)` (through free cells), or
    /// `None` when there is none. Used by bots and tests to walk a cook to a station.
    pub fn next_step(&self, i: usize, tx: usize, ty: usize) -> Option<Dir> {
        let c = self.cooks[i];
        // Breadth-first search over free cells from the cook's cell.
        let mut prev = [[None::<Dir>; W]; H];
        let mut seen = [[false; W]; H];
        let mut queue = std::collections::VecDeque::new();
        seen[c.y][c.x] = true;
        queue.push_back((c.x, c.y));
        while let Some((x, y)) = queue.pop_front() {
            if (x, y) == (tx, ty) {
                break;
            }
            for dir in [Dir::Up, Dir::Down, Dir::Left, Dir::Right] {
                let (dx, dy) = dir.delta();
                let (nx, ny) = (x as isize + dx, y as isize + dy);
                if nx < 0 || ny < 0 || nx as usize >= W || ny as usize >= H {
                    continue;
                }
                let (nx, ny) = (nx as usize, ny as usize);
                if seen[ny][nx] || !(self.free(nx, ny) || (nx, ny) == (tx, ty)) {
                    continue;
                }
                seen[ny][nx] = true;
                prev[ny][nx] = Some(dir);
                queue.push_back((nx, ny));
            }
        }
        if !seen[ty][tx] {
            return None;
        }
        // Walk back from the target to the first step out of the cook's cell.
        let (mut x, mut y) = (tx, ty);
        let mut first = None;
        while (x, y) != (c.x, c.y) {
            let dir = prev[y][x]?;
            first = Some(dir);
            let (dx, dy) = dir.delta();
            x = (x as isize - dx) as usize;
            y = (y as isize - dy) as usize;
        }
        first
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Walks cook `i` to `(tx, ty)` by the shortest route; the other cooks stand still.
    fn walk(k: &mut Kitchen, i: usize, tx: usize, ty: usize) {
        for _ in 0..100 {
            let c = k.cooks[i];
            if (c.x, c.y) == (tx, ty) {
                return;
            }
            let dir = k.next_step(i, tx, ty).expect("a route");
            let mut inputs = vec![None; k.cooks.len()];
            inputs[i] = Some(Input::Move(dir));
            k.step(&inputs);
        }
        panic!("cook {i} did not reach ({tx}, {ty})");
    }

    /// Faces cook `i` toward the cell at (x, y), by moving into it (a station or wall turns the cook).
    fn face(k: &mut Kitchen, i: usize, dir: Dir) {
        let mut inputs = vec![None; k.cooks.len()];
        inputs[i] = Some(Input::Move(dir));
        k.step(&inputs);
    }

    fn act(k: &mut Kitchen, i: usize) -> Vec<Event> {
        let mut inputs = vec![None; k.cooks.len()];
        inputs[i] = Some(Input::Interact);
        k.step(&inputs)
    }

    fn idle(k: &mut Kitchen, ticks: u32) {
        for _ in 0..ticks {
            k.step(&[None, None]);
        }
    }

    #[test]
    fn the_map_has_one_pantry_grill_counter_and_a_hatch_per_team() {
        let mut found = std::collections::HashMap::new();
        for y in 0..H {
            for x in 0..W {
                *found.entry(format!("{:?}", cell(x, y))).or_insert(0) += 1;
            }
        }
        assert_eq!(found["Pantry"], 1);
        assert_eq!(found["Grill"], 1);
        assert_eq!(found["Plates"], 1);
        assert_eq!(found["Hatch(0)"], 3);
        assert_eq!(found["Hatch(1)"], 3);
    }

    #[test]
    fn a_move_into_a_station_turns_the_cook_and_does_not_walk_onto_it() {
        let mut k = Kitchen::new(100);
        walk(&mut k, 0, 2, 1); // under the pantry
        face(&mut k, 0, Dir::Up);
        let c = k.cooks[0];
        assert_eq!((c.x, c.y), (2, 1), "the pantry is not walkable");
        assert_eq!(c.facing, Dir::Up);
    }

    #[test]
    fn a_dish_is_made_and_delivered_at_your_own_hatch() {
        let mut k = Kitchen::new(10_000);
        // Skewer from the pantry.
        walk(&mut k, 0, 2, 1);
        face(&mut k, 0, Dir::Up);
        act(&mut k, 0);
        assert_eq!(k.cooks[0].held, Held::Skewer);
        // On the grill, then cooking.
        walk(&mut k, 0, 4, 1);
        face(&mut k, 0, Dir::Up);
        act(&mut k, 0);
        assert_eq!(k.cooks[0].held, Held::Nothing);
        assert!(k.grill.is_some(), "the kebab is on the grill");
        idle(&mut k, COOK_TICKS);
        act(&mut k, 0);
        assert_eq!(k.cooks[0].held, Held::Cooked);
        assert_eq!(k.grill, None);
        // On a plate.
        walk(&mut k, 0, 6, 1);
        face(&mut k, 0, Dir::Up);
        act(&mut k, 0);
        assert_eq!(k.cooks[0].held, Held::Plated);
        // Delivered at team 0's hatch (the left wall).
        walk(&mut k, 0, 1, 1);
        face(&mut k, 0, Dir::Left);
        let events = act(&mut k, 0);
        assert_eq!(events, vec![Event::Delivered { team: 0 }]);
        assert_eq!(k.delivered, [1, 0]);
    }

    #[test]
    fn a_plate_is_delivered_only_at_your_own_hatch() {
        let mut k = Kitchen::new(10_000);
        k.cooks[0].held = Held::Plated;
        walk(&mut k, 0, 7, 2); // beside team 1's hatch (x = 8), below team 1's cook
        face(&mut k, 0, Dir::Right);
        assert!(
            act(&mut k, 0).is_empty(),
            "another team's hatch takes nothing"
        );
        assert_eq!(k.cooks[0].held, Held::Plated);
        assert_eq!(k.delivered, [0, 0]);
    }

    #[test]
    fn the_grill_is_shared_one_kebab_at_a_time() {
        let mut k = Kitchen::new(10_000);
        k.grill = Some(0);
        k.cooks[1].held = Held::Skewer;
        walk(&mut k, 1, 4, 1);
        face(&mut k, 1, Dir::Up);
        act(&mut k, 1);
        assert_eq!(k.cooks[1].held, Held::Skewer, "the grill is busy");
    }

    #[test]
    fn the_clock_decides_the_winner_and_a_tie_plays_on() {
        let mut k = Kitchen::new(5);
        idle(&mut k, 5);
        assert!(!k.over(), "a tie at the bell plays on");
        assert_eq!(k.winner(), None);
        k.delivered[1] += 1;
        assert!(k.over());
        assert_eq!(k.winner(), Some(1));
    }

    #[test]
    fn nothing_moves_once_the_match_is_over() {
        let mut k = Kitchen::new(2);
        k.delivered[0] = 1;
        idle(&mut k, 2);
        assert!(k.over());
        let before = k.clone();
        let mut inputs = vec![None; 2];
        inputs[0] = Some(Input::Move(Dir::Down));
        k.step(&inputs);
        assert_eq!(k, before);
    }
}
