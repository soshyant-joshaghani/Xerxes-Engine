# Game modes

Every level declares a `GameMode` (`engine/sim/src/mode.rs`, re-exported by `engine/src/modules/flow/mode.rs`), typed on the
*Game Mode Taxonomy: Sides × Win Condition*, an illustrated guide kept with the engine code that uses it: [`engine/src/modules/flow/game-mode-taxonomy.html`](../engine/src/modules/flow/game-mode-taxonomy.html). The editor uses it as the guide for picking a scene's game mode.
Any mode in any game fits one cell, plus modifiers. The match runtime (`engine/sim/src/matches.rs`, in Bevy `modules/matches`) plays them: see "How a mode plays" below.

## The grid: sides × win condition

| Sides ↓ / Win → | Score | Objective | Role | Survival |
|---|---|---|---|---|
| **Solo** (one player) | Time trial / high score | Challenge clear | *structurally empty* | Endurance run |
| **Coop** (one side, shared fate) | Shared high score | Shared clear | Functional split | Team endurance |
| **Duel** (2 sides) | Head-to-head score | Base / goal destruction | Attacker vs. defender | Round elimination |
| **N-gon** (3+ sides) | Multi-way race | Multi-side control | Hidden role / social deduction | Last one standing |

- Solo × Role does not exist: a role only means something relative to other players. `validate()` rejects it.
- N-gon needs at least 3 sides.
- Everything except Solo needs multiplayer (`needs_multiplayer()`); the editor marks those cells.

## Outside the grid

| Kind | Meaning |
|---|---|
| `Sandbox` | No win condition: practice range, creative/build mode |
| `Progression` | Progressed, not won: loot, levels, gear, economy |
| `Narrative` | Authored forward progress, no failure state |
| `Composite([...])` | Several kinds stacked (open-world: missions + sandbox + progression + online modes) |
| `Tournament(kind)` | A meta wrapper repeating a Duel/N-gon cell across elimination rounds |

## Modifiers (a third axis, never a new cell)

`Permadeath`, `Respawn { seconds }`, `TimeLimit { seconds }`, `FogOfWar`, `LowGravity`,
`ShrinkingZone`, `Escort`, `MovingObjective`, plus the `asymmetric` flag (uneven side sizes,
1-vs-many). Example: "2 sides, Objective, escort payload, respawns, 15-minute limit" is
Duel × Objective + `Escort` + `Respawn` + `TimeLimit`.

```rust
// assets/<name>.project.rs, one level
LevelDef {
    name: "Level 1",
    scene: crate::assets::scenes::main_scene::scene,
    mode: GameMode::grid(Sides::Solo, WinCondition::Objective).with(Modifier::TimeLimit { seconds: 120.0 }),
}
```

## How a mode plays

The game reports what happened and the match decides the rest. Phases: Lobby, Countdown, Playing, RoundEnd, Results. A match is `Rules` (the mode, the sides and their players, rounds to win, countdown, score target, goal, timeout winner) plus reports: `Score`, `Progress`, `Win`, `Down`.

| Win condition | The round ends when | Winner |
|---|---|---|
| Score | the time limit passes, or a side reaches the score target | the highest score; a tie at the top is a draw |
| Objective, Role | a side's progress reaches the goal, or a side is reported `Win` | that side; at the time limit the `timeout_winner` (the defender), else nobody |
| Survival | one side or none is left standing (alone: when it is out); at the time limit | the side left, or every side still standing |

Solo and Coop have one side, so their results are `Finished` (Score; Survival ended by being out, ranked by time survived), `Won` or `Lost`. Duel and N-gon rank the sides (equal results share a rank). `rounds_to_win` above 1 plays best-of rounds. Sandbox, progression and narrative never end; composites and tournaments are not one match (their parts are).

Modifiers: `TimeLimit` is the round clock; `Respawn` brings a downed player back after the delay (a side is eliminated only when nobody is up or coming back) and `Permadeath` turns that off; `ShrinkingZone` gives `zone()` (1 down to 0.2 over the time limit, or 90 s); `LowGravity` gives `gravity_scale()`; `FogOfWar`, `Escort` and `MovingObjective` are flags the scene reads. `asymmetric` is the side sizes.
