//! Game modes, typed on the Sides × Win Condition taxonomy (__docs__/game-modes.md; the illustrated guide the editor shows when a level's mode is set is `engine/src/modules/flow/game-mode-taxonomy.html`). Every level
//! declares one; the match runtime ([`crate::matches`]) plays them. Lived in the engine until stage 3.2.
//!
//! ```ignore
//! GameMode::grid(Sides::Solo, WinCondition::Objective).with(Modifier::TimeLimit { seconds: 120.0 })
//! GameMode::sandbox()
//! ```

/// How many independent parties compete.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sides {
    /// One player, one run.
    Solo,
    /// Two or more players on one side, shared fate.
    Coop,
    /// Two sides (one or many a side).
    Duel,
    /// Three or more independent sides.
    NGon(u8),
}

/// What decides the winner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WinCondition {
    /// Highest number wins, ranked on a board.
    Score,
    /// Pass or fail a defined goal.
    Objective,
    /// Distinct jobs with distinct win terms.
    Role,
    /// Endure longest, or be the last one left.
    Survival,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModeKind {
    /// A cell of the sides × win-condition grid.
    Grid { sides: Sides, win: WinCondition },
    /// No win condition: practice range, creative/build mode.
    Sandbox,
    /// Progressed, not won: loot, levels, gear, economy.
    Progression,
    /// Authored forward progress, no failure state.
    Narrative,
    /// Several kinds stacked (open-world games).
    Composite(Vec<ModeKind>),
    /// A meta wrapper repeating a Duel or N-gon cell across elimination rounds.
    Tournament(Box<ModeKind>),
}

/// A gameplay modifier: stacks onto any mode, never defines a new one.
#[derive(Debug, Clone, PartialEq)]
pub enum Modifier {
    Permadeath,
    Respawn { seconds: f32 },
    TimeLimit { seconds: f32 },
    FogOfWar,
    LowGravity,
    ShrinkingZone,
    Escort,
    MovingObjective,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameMode {
    pub kind: ModeKind,
    pub modifiers: Vec<Modifier>,
    /// Uneven side sizes (1-vs-many) with the same win condition.
    pub asymmetric: bool,
}

impl GameMode {
    pub fn new(kind: ModeKind) -> Self {
        Self {
            kind,
            modifiers: Vec::new(),
            asymmetric: false,
        }
    }

    pub fn grid(sides: Sides, win: WinCondition) -> Self {
        Self::new(ModeKind::Grid { sides, win })
    }

    pub fn sandbox() -> Self {
        Self::new(ModeKind::Sandbox)
    }

    pub fn with(mut self, modifier: Modifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    pub fn asymmetric(mut self) -> Self {
        self.asymmetric = true;
        self
    }

    /// Rejects modes that cannot exist: Solo × Role (a role needs other players), N-gon
    /// with fewer than 3 sides, a tournament of anything but Duel/N-gon cells, an empty
    /// composite, and non-positive times.
    pub fn validate(&self) -> Result<(), String> {
        validate_kind(&self.kind)?;
        for modifier in &self.modifiers {
            if let Modifier::Respawn { seconds } | Modifier::TimeLimit { seconds } = modifier
                && *seconds <= 0.0
            {
                return Err(format!("{modifier:?}: seconds must be positive"));
            }
        }
        if self.asymmetric
            && !matches!(
                self.kind,
                ModeKind::Grid {
                    sides: Sides::Duel | Sides::NGon(_),
                    ..
                }
            )
        {
            return Err("asymmetric needs a Duel or N-gon mode (uneven sides)".into());
        }
        Ok(())
    }

    /// Whether this mode needs more than one player (everything but Solo and the
    /// single-player kinds).
    pub fn needs_multiplayer(&self) -> bool {
        kind_needs_multiplayer(&self.kind)
    }

    /// The taxonomy's name for this mode ("Endurance run", "Round elimination", ...).
    pub fn cell_label(&self) -> String {
        kind_label(&self.kind)
    }
}

fn validate_kind(kind: &ModeKind) -> Result<(), String> {
    match kind {
        ModeKind::Grid { sides: Sides::Solo, win: WinCondition::Role } => {
            Err("Solo × Role is structurally empty: a role only means something relative to other players".into())
        }
        ModeKind::Grid { sides: Sides::NGon(n), .. } if *n < 3 => Err(format!("N-gon needs 3 or more sides, not {n}")),
        ModeKind::Composite(kinds) if kinds.is_empty() => Err("a composite mode needs at least one kind".into()),
        ModeKind::Composite(kinds) => kinds.iter().try_for_each(validate_kind),
        ModeKind::Tournament(inner) => match **inner {
            ModeKind::Grid { sides: Sides::Duel | Sides::NGon(_), .. } => validate_kind(inner),
            _ => Err("a tournament wraps a Duel or N-gon cell".into()),
        },
        _ => Ok(()),
    }
}

fn kind_needs_multiplayer(kind: &ModeKind) -> bool {
    match kind {
        ModeKind::Grid { sides, .. } => *sides != Sides::Solo,
        ModeKind::Composite(kinds) => kinds.iter().any(kind_needs_multiplayer),
        ModeKind::Tournament(_) => true,
        ModeKind::Sandbox | ModeKind::Progression | ModeKind::Narrative => false,
    }
}

/// The taxonomy's cell names.
pub fn grid_label(sides: Sides, win: WinCondition) -> &'static str {
    use {Sides::*, WinCondition::*};
    match (sides, win) {
        (Solo, Score) => "Time trial / high score",
        (Solo, Objective) => "Challenge clear",
        (Solo, Role) => "Structurally empty",
        (Solo, Survival) => "Endurance run",
        (Coop, Score) => "Shared high score",
        (Coop, Objective) => "Shared clear",
        (Coop, Role) => "Functional split",
        (Coop, Survival) => "Team endurance",
        (Duel, Score) => "Head-to-head score",
        (Duel, Objective) => "Base / goal destruction",
        (Duel, Role) => "Attacker vs. defender",
        (Duel, Survival) => "Round elimination",
        (NGon(_), Score) => "Multi-way race",
        (NGon(_), Objective) => "Multi-side control",
        (NGon(_), Role) => "Hidden role / social deduction",
        (NGon(_), Survival) => "Last one standing",
    }
}

fn kind_label(kind: &ModeKind) -> String {
    match kind {
        ModeKind::Grid { sides, win } => grid_label(*sides, *win).into(),
        ModeKind::Sandbox => "Sandbox".into(),
        ModeKind::Progression => "Progression".into(),
        ModeKind::Narrative => "Narrative".into(),
        ModeKind::Composite(kinds) => kinds.iter().map(kind_label).collect::<Vec<_>>().join(" + "),
        ModeKind::Tournament(inner) => format!("Tournament of {}", kind_label(inner)),
    }
}
