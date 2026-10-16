//! Channel / sluice step rules (Bevy-free): a room of rock and soil, a spring, a village.
//! The player lays and turns channel tiles, then opens the sluice; water runs cell by cell along
//! matching openings, never uphill. Reaching the village clears the room, with a tier for how few
//! tiles were used.
//!
//! Used by Darius's **Qanat** (2D). Cyrus's Arash's Arrow diorama (slice 1) used it too. Inputs come one at a
//! time and may be rejected; every change can be undone; water advances one ring per
//! [`Qanat::step`].

/// The open sides of a cell, as bits.
pub const N: u8 = 1;
pub const E: u8 = 2;
pub const S: u8 = 4;
pub const W: u8 = 8;

/// Every shape a tile can take, in the order a turn cycles through them (`0` is no tile).
pub const SHAPES: [u8; 7] = [0, E | W, N | S, N | E, E | S, S | W, W | N];

/// The spring lets the water out at the bottom.
pub const SPRING_SIDES: u8 = S;

/// What a cell of the room is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ground {
    #[default]
    Rock,
    /// Where tiles can be laid.
    Soil,
    /// Where the water comes from (one per room).
    Spring,
    /// Where the water must reach (one per room); it takes water from any side.
    Village,
}

/// A room: its cells (row 0 at the top) and the tile count for gold.
#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    width: usize,
    height: usize,
    ground: Vec<Ground>,
    par: u32,
}

impl Room {
    /// A room from text rows: `#` rock, `.` soil, `S` the spring, `V` the village.
    pub fn from_rows(rows: &[&str], par: u32) -> Result<Self, String> {
        let mut cells = Vec::new();
        for (row, line) in rows.iter().enumerate() {
            for (col, c) in line.chars().enumerate() {
                let ground = match c {
                    '#' => Ground::Rock,
                    '.' => Ground::Soil,
                    'S' => Ground::Spring,
                    'V' => Ground::Village,
                    other => return Err(format!("unknown cell `{other}` at {col},{row}")),
                };
                cells.push((col, row, ground));
            }
        }
        Self::from_cells(cells, par)
    }

    /// A room from cells (`col`, `row`, ground), as a scene lists them. Missing cells are rock.
    pub fn from_cells(
        cells: impl IntoIterator<Item = (usize, usize, Ground)>,
        par: u32,
    ) -> Result<Self, String> {
        let cells: Vec<_> = cells.into_iter().collect();
        let width = cells.iter().map(|c| c.0 + 1).max().unwrap_or(0);
        let height = cells.iter().map(|c| c.1 + 1).max().unwrap_or(0);
        let mut ground = vec![Ground::Rock; width * height];
        for (col, row, g) in cells {
            ground[row * width + col] = g;
        }
        let count = |g| ground.iter().filter(|&&x| x == g).count();
        if count(Ground::Spring) != 1 || count(Ground::Village) != 1 {
            return Err("a room needs exactly one spring and one village".into());
        }
        Ok(Self {
            width,
            height,
            ground,
            par,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn par(&self) -> u32 {
        self.par
    }

    pub fn ground(&self, col: usize, row: usize) -> Ground {
        self.ground[row * self.width + col]
    }

    fn index(&self, col: usize, row: usize) -> Option<usize> {
        (col < self.width && row < self.height).then_some(row * self.width + col)
    }

    /// The cell next to `i` through `side`, if it is inside the room.
    fn neighbour(&self, i: usize, side: u8) -> Option<usize> {
        let (col, row) = (i % self.width, i / self.width);
        match side {
            N => row.checked_sub(1).map(|r| r * self.width + col),
            S => self.index(col, row + 1),
            E => self.index(col + 1, row),
            W => col.checked_sub(1).map(|c| row * self.width + c),
            _ => None,
        }
    }
}

fn opposite(side: u8) -> u8 {
    match side {
        N => S,
        S => N,
        E => W,
        _ => E,
    }
}

/// One thing the player does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    /// Lay a tile on empty soil, or turn the tile there to its next shape (the last shape
    /// lifts it).
    Turn {
        col: usize,
        row: usize,
    },
    Undo,
    /// Lift every tile (undoable).
    Clear,
    /// Open the sluice: the water starts to run.
    Open,
}

/// Why an input was refused. Nothing changes when one is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejected {
    OutOfRoom,
    /// Tiles go in soil only, not in rock, the spring or the village.
    NotSoil,
    /// The water is running, or the room is cleared.
    Busy,
    NothingToUndo,
}

/// How well the room was cleared: gold at par tiles or fewer, silver within two more.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Gold,
    Silver,
    Bronze,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Laying tiles.
    Building,
    /// The water is running.
    Flowing,
    /// The water stopped short of the village; the next change dries the channel.
    Dry,
    /// The water reached the village.
    Won(Tier),
}

/// A room being played.
#[derive(Debug, Clone)]
pub struct Qanat {
    room: Room,
    tiles: Vec<u8>,
    history: Vec<Vec<u8>>,
    wet: Vec<bool>,
    front: Vec<usize>,
    phase: Phase,
}

impl Qanat {
    pub fn new(room: Room) -> Self {
        let cells = room.ground.len();
        Self {
            room,
            tiles: vec![0; cells],
            history: Vec::new(),
            wet: vec![false; cells],
            front: Vec::new(),
            phase: Phase::Building,
        }
    }

    pub fn room(&self) -> &Room {
        &self.room
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// The open sides of a cell: its tile, the spring's outlet, or nothing.
    pub fn sides(&self, col: usize, row: usize) -> u8 {
        let i = row * self.room.width + col;
        match self.room.ground[i] {
            Ground::Soil => self.tiles[i],
            Ground::Spring => SPRING_SIDES,
            _ => 0,
        }
    }

    pub fn is_wet(&self, col: usize, row: usize) -> bool {
        self.wet[row * self.room.width + col]
    }

    pub fn tiles_used(&self) -> u32 {
        self.tiles.iter().filter(|&&t| t != 0).count() as u32
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn tier(&self) -> Tier {
        let used = self.tiles_used();
        if used <= self.room.par {
            Tier::Gold
        } else if used <= self.room.par + 2 {
            Tier::Silver
        } else {
            Tier::Bronze
        }
    }

    /// Applies one input, or says why not.
    pub fn apply(&mut self, input: Input) -> Result<(), Rejected> {
        if matches!(self.phase, Phase::Flowing | Phase::Won(_)) {
            return Err(Rejected::Busy);
        }
        match input {
            Input::Turn { col, row } => {
                let i = self.room.index(col, row).ok_or(Rejected::OutOfRoom)?;
                if self.room.ground[i] != Ground::Soil {
                    return Err(Rejected::NotSoil);
                }
                self.remember();
                let at = SHAPES.iter().position(|&s| s == self.tiles[i]).unwrap_or(0);
                self.tiles[i] = SHAPES[(at + 1) % SHAPES.len()];
            }
            Input::Undo => {
                self.tiles = self.history.pop().ok_or(Rejected::NothingToUndo)?;
            }
            Input::Clear => {
                if self.tiles_used() > 0 {
                    self.remember();
                    self.tiles.fill(0);
                }
            }
            Input::Open => {
                self.dry();
                let spring = self.room.ground.iter().position(|&g| g == Ground::Spring);
                let spring = spring.expect("a room has a spring");
                self.wet[spring] = true;
                self.front = vec![spring];
                self.phase = Phase::Flowing;
                return Ok(());
            }
        }
        self.dry();
        Ok(())
    }

    /// Advances the water by one cell along every open channel. Returns whether it moved or
    /// the phase changed (false when there is no water running).
    pub fn step(&mut self) -> bool {
        if self.phase != Phase::Flowing {
            return false;
        }
        let mut next = Vec::new();
        for &i in &self.front {
            let out = match self.room.ground[i] {
                Ground::Spring => SPRING_SIDES,
                _ => self.tiles[i],
            };
            // Water never flows uphill.
            for side in [E, S, W].into_iter().filter(|s| out & s != 0) {
                let Some(j) = self.room.neighbour(i, side) else {
                    continue;
                };
                let takes = match self.room.ground[j] {
                    Ground::Village => true,
                    Ground::Soil => self.tiles[j] & opposite(side) != 0,
                    _ => false,
                };
                if takes && !self.wet[j] {
                    self.wet[j] = true;
                    if self.room.ground[j] == Ground::Village {
                        self.phase = Phase::Won(self.tier());
                        return true;
                    }
                    next.push(j);
                }
            }
        }
        if next.is_empty() {
            self.phase = Phase::Dry;
        }
        self.front = next;
        true
    }

    fn remember(&mut self) {
        self.history.push(self.tiles.clone());
    }

    /// Empties the channel after a dry run, back to building.
    fn dry(&mut self) {
        self.wet.fill(false);
        self.front.clear();
        self.phase = Phase::Building;
    }
}

/// The first room and its gold solution (shared by Qanat and its tests).
pub mod fixtures {
    use super::*;

    /// The first room, the same as `assets/scenes/games/solo/qanat.scene.rs`.
    pub const ROOM: [&str; 7] = [
        "S..######",
        "..##...##",
        "#..#.#..#",
        "##...##.#",
        "####..#..",
        "#####...V",
        "#########",
    ];

    /// A gold solution of [`ROOM`]: (col, row, turns from empty).
    pub const SOLUTION: [(usize, usize, usize); 12] = [
        (0, 1, 3), // N|E
        (1, 1, 5), // S|W
        (1, 2, 3),
        (2, 2, 5),
        (2, 3, 3),
        (3, 3, 1), // E|W
        (4, 3, 5),
        (4, 4, 3),
        (5, 4, 5),
        (5, 5, 3),
        (6, 5, 1),
        (7, 5, 1),
    ];

    pub fn lay(game: &mut Qanat, solution: &[(usize, usize, usize)]) {
        for &(col, row, turns) in solution {
            for _ in 0..turns {
                game.apply(Input::Turn { col, row }).unwrap();
            }
        }
    }

    pub fn run(game: &mut Qanat) {
        game.apply(Input::Open).unwrap();
        for _ in 0..100 {
            if !game.step() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;

    fn room() -> Qanat {
        Qanat::new(Room::from_rows(&ROOM, 12).unwrap())
    }

    #[test]
    fn a_room_needs_one_spring_and_one_village() {
        assert!(Room::from_rows(&["S.V"], 1).is_ok());
        assert!(Room::from_rows(&["S.S", "..V"], 1).is_err());
        assert!(Room::from_rows(&["S.."], 1).is_err());
        assert!(Room::from_rows(&["S?V"], 1).is_err());
    }

    #[test]
    fn a_turn_cycles_every_shape_and_lifts_the_tile() {
        let mut game = room();
        for shape in &SHAPES[1..] {
            game.apply(Input::Turn { col: 1, row: 0 }).unwrap();
            assert_eq!(game.sides(1, 0), *shape);
        }
        game.apply(Input::Turn { col: 1, row: 0 }).unwrap();
        assert_eq!(game.sides(1, 0), 0, "the last turn lifts it");
    }

    #[test]
    fn rock_spring_village_and_outside_are_refused() {
        let mut game = room();
        let turn = |col, row| Input::Turn { col, row };
        assert_eq!(game.apply(turn(3, 0)), Err(Rejected::NotSoil));
        assert_eq!(game.apply(turn(0, 0)), Err(Rejected::NotSoil));
        assert_eq!(game.apply(turn(8, 5)), Err(Rejected::NotSoil));
        assert_eq!(game.apply(turn(9, 0)), Err(Rejected::OutOfRoom));
        assert!(!game.can_undo(), "a refused input changes nothing");
    }

    #[test]
    fn undo_and_clear() {
        let mut game = room();
        assert_eq!(game.apply(Input::Undo), Err(Rejected::NothingToUndo));
        lay(&mut game, &SOLUTION[..2]);
        game.apply(Input::Undo).unwrap();
        assert_eq!(game.sides(1, 1), SHAPES[4], "one turn back");
        game.apply(Input::Clear).unwrap();
        assert_eq!(game.tiles_used(), 0);
        game.apply(Input::Undo).unwrap();
        assert_eq!(game.tiles_used(), 2, "clearing can be undone");
    }

    #[test]
    fn the_solution_clears_the_room_with_gold() {
        let mut game = room();
        lay(&mut game, &SOLUTION);
        run(&mut game);
        assert_eq!(game.phase(), Phase::Won(Tier::Gold));
        assert!(game.is_wet(7, 5));
        assert_eq!(
            game.apply(Input::Turn { col: 1, row: 0 }),
            Err(Rejected::Busy)
        );
    }

    #[test]
    fn extra_tiles_lower_the_tier() {
        let mut game = room();
        lay(&mut game, &SOLUTION);
        lay(&mut game, &[(1, 0, 1), (2, 0, 1), (4, 1, 1)]);
        run(&mut game);
        assert_eq!(game.phase(), Phase::Won(Tier::Bronze));
    }

    #[test]
    fn a_gap_leaves_it_dry_and_the_next_change_dries_the_channel() {
        let mut game = room();
        lay(&mut game, &SOLUTION[..11]);
        run(&mut game);
        assert_eq!(game.phase(), Phase::Dry);
        assert!(game.is_wet(6, 5) && !game.is_wet(7, 5));
        game.apply(Input::Turn { col: 7, row: 5 }).unwrap();
        assert_eq!(game.phase(), Phase::Building);
        assert!(!game.is_wet(6, 5));
        run(&mut game);
        assert!(matches!(game.phase(), Phase::Won(_)));
    }

    #[test]
    fn water_never_flows_uphill() {
        // The village is above the end of the channel: a W|N tile would lead up into it.
        let mut game = Qanat::new(Room::from_rows(&["S#V", "..."], 1).unwrap());
        lay(&mut game, &[(0, 1, 3), (1, 1, 1), (2, 1, 6)]);
        assert_eq!(game.sides(2, 1), W | N);
        run(&mut game);
        assert_eq!(game.phase(), Phase::Dry);
    }

    #[test]
    fn the_water_runs_one_cell_a_step() {
        let mut game = room();
        lay(&mut game, &SOLUTION);
        game.apply(Input::Open).unwrap();
        assert!(game.is_wet(0, 0) && !game.is_wet(0, 1));
        game.step();
        assert!(game.is_wet(0, 1) && !game.is_wet(1, 1));
        assert_eq!(game.phase(), Phase::Flowing);
    }
}
