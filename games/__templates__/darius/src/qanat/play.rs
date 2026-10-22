//! Qanat in Bevy. The scene holds the room as cells ([`QanatCell`]); when they appear this builds
//! the rules from them ([`Board`]), turns pointer, touch, keys and HUD buttons into
//! [`Input`]s, runs the water on the fixed clock, and draws each cell's tile as child sprites.

use bevy::input::ButtonState;
use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use bevy::window::WindowEvent;
use xerxes_engine::modules::flow::{CurrentLevel, Flow, LoadLevel, Play};
use xerxes_engine::modules::input::Actions;
use xerxes_engine::modules::matches::{ActiveMatch, MatchReport, Report};

use xerxes_sim::qanat::{E, Ground, Input, N, Phase, Qanat, Rejected, Room, S, Tier, W};

/// Tiles at or under this count earn gold in the first room.
const PAR: u32 = 12;
/// Distance between cell centres, and half of it for hit tests (world pixels).
const CELL: f32 = 56.0;
/// Fixed ticks between two steps of the water.
const WATER_TICKS: u32 = 6;

const CLAY: Color = Color::srgb(0.80, 0.55, 0.33);
const WATER: Color = Color::srgb(0.30, 0.66, 0.92);

/// A cell of the room, written in the scene: where it is and what it is made of.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct QanatCell {
    pub col: u8,
    pub row: u8,
    pub ground: Ground,
}

/// The frame behind the selected cell (moved by the arrows and the pointer).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct CellCursor;

/// The scene's camera, to turn the pointer into a world position.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct BoardCamera;

/// What a HUD button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QanatAction {
    Open,
    Undo,
    Clear,
    /// Start the room again: a fresh room and a fresh match (also after it is cleared).
    Restart,
}

/// A HUD button.
#[derive(Component, Debug, Clone, Copy)]
pub struct QanatButton(pub QanatAction);

/// The line under the title: tiles, par, and what just happened.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct StatusText;

/// The full-screen banner shown when the room is cleared.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ResultBanner;

/// The banner's text.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ResultText;

/// The room being played, while a level with Qanat's cells is on.
#[derive(Resource)]
struct Board {
    game: Qanat,
    cursor: (usize, usize),
    /// A short message after a refused input, and how long it stays.
    notice: Option<(&'static str, f32)>,
    reported: bool,
    /// Built this frame: input from before it (the menu click that opened the level) is not
    /// the board's.
    fresh: bool,
}

impl Board {
    fn apply(&mut self, input: Input) {
        if let Err(why) = self.game.apply(input) {
            let text = match why {
                Rejected::NotSoil => "Tiles go in the soil only",
                Rejected::OutOfRoom => "Outside the hill",
                Rejected::Busy if matches!(self.game.phase(), Phase::Won(_)) => {
                    "The room is cleared: Restart to play it again"
                }
                Rejected::Busy => "The water is running",
                Rejected::NothingToUndo => "Nothing to undo",
            };
            self.notice = Some((text, 1.6));
        }
    }
}

/// What a cell's children show, so they are rebuilt only when it changes.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct Drawn {
    sides: u8,
    wet: bool,
}

pub struct QanatPlugin;

impl Plugin for QanatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(Flow::Level), |mut commands: Commands| {
            commands.remove_resource::<Board>();
        })
        .add_systems(
            Update,
            (
                build_board.run_if(not(resource_exists::<Board>)),
                (
                    pointer_input,
                    key_input,
                    press_buttons,
                    draw_cells,
                    refresh_hud,
                )
                    .chain()
                    .run_if(resource_exists::<Board>),
            )
                .chain()
                .run_if(in_state(Flow::Level)),
        )
        .add_systems(
            FixedUpdate,
            run_water.in_set(Play).run_if(resource_exists::<Board>),
        );
    }
}

/// The room is the scene's cells: once they are spawned, build the rules from them.
fn build_board(
    mut commands: Commands,
    added: Query<(), Added<QanatCell>>,
    cells: Query<&QanatCell>,
) {
    if added.is_empty() {
        return;
    }
    let list = cells
        .iter()
        .map(|c| (c.col as usize, c.row as usize, c.ground));
    match Room::from_cells(list, PAR) {
        Ok(room) => {
            let cursor = (0..room.width())
                .flat_map(|c| (0..room.height()).map(move |r| (c, r)))
                .find(|&(c, r)| room.ground(c, r) == Ground::Soil)
                .unwrap_or((0, 0));
            commands.insert_resource(Board {
                game: Qanat::new(room),
                cursor,
                notice: None,
                reported: false,
                fresh: true,
            });
        }
        Err(err) => error!("qanat scene: {err}"),
    }
}

/// The cell under a world position.
fn cell_at(cells: &Query<(&QanatCell, &GlobalTransform)>, at: Vec2) -> Option<(usize, usize)> {
    cells.iter().find_map(|(cell, transform)| {
        let d = (at - transform.translation().truncate()).abs();
        (d.x <= CELL / 2.0 && d.y <= CELL / 2.0).then_some((cell.col as usize, cell.row as usize))
    })
}

/// A click or a tap turns the tile under it; the pointer moves the cursor. A press on a HUD
/// button is the button's, not the board's.
///
/// The window's events are read in order, so every press is paired with where the pointer was
/// at that moment: several clicks within one frame (a slow frame, a fast player) each turn the
/// cell they were made on, not the cell the pointer ends the frame over.
fn pointer_input(
    mut board: ResMut<Board>,
    mut events: MessageReader<WindowEvent>,
    camera: Single<(&Camera, &GlobalTransform), With<BoardCamera>>,
    cells: Query<(&QanatCell, &GlobalTransform)>,
    ui: Query<&Interaction, With<Button>>,
    mut pointer: Local<Option<Vec2>>,
) {
    if board.fresh {
        board.fresh = false;
        events.clear();
        return;
    }
    let over_ui = ui.iter().any(|i| *i != Interaction::None);
    let (camera, camera_at) = *camera;
    let cell = |screen: Vec2| {
        camera
            .viewport_to_world_2d(camera_at, screen)
            .ok()
            .and_then(|at| cell_at(&cells, at))
    };
    for event in events.read() {
        let press = match event {
            WindowEvent::CursorMoved(moved) => {
                *pointer = Some(moved.position);
                if let Some(at) = cell(moved.position) {
                    board.cursor = at;
                }
                None
            }
            WindowEvent::CursorLeft(_) => {
                *pointer = None;
                None
            }
            WindowEvent::MouseButtonInput(click)
                if click.button == MouseButton::Left && click.state == ButtonState::Pressed =>
            {
                *pointer
            }
            WindowEvent::TouchInput(touch) if touch.phase == TouchPhase::Started => {
                Some(touch.position)
            }
            _ => None,
        };
        if over_ui {
            continue;
        }
        if let Some((col, row)) = press.and_then(cell) {
            board.cursor = (col, row);
            board.apply(Input::Turn { col, row });
        }
    }
}

/// Reloads the level: the flow rebuilds the scene (so the board), the engine starts a fresh
/// match.
fn restart(commands: &mut Commands, level: &CurrentLevel, load: &mut MessageWriter<LoadLevel>) {
    load.write(LoadLevel(level.0));
    commands.remove_resource::<ActiveMatch>();
}

/// The cursor steps on `cell_*` (arrows or WASD, the gamepad D-pad), `turn` turns, `undo`,
/// `clear`, `open` (the sluice) and `respawn` restarts. The keys and buttons are the game's
/// `input` map in `darius.project.rs`.
fn key_input(
    mut commands: Commands,
    mut board: ResMut<Board>,
    actions: Res<Actions>,
    level: Res<CurrentLevel>,
    mut load: MessageWriter<LoadLevel>,
) {
    if actions.just_pressed("respawn") {
        restart(&mut commands, &level, &mut load);
        return;
    }
    let (w, h) = (board.game.room().width(), board.game.room().height());
    let (col, row) = board.cursor;
    let moved = if actions.just_pressed("cell_left") {
        (col.saturating_sub(1), row)
    } else if actions.just_pressed("cell_right") {
        ((col + 1).min(w - 1), row)
    } else if actions.just_pressed("cell_up") {
        (col, row.saturating_sub(1))
    } else if actions.just_pressed("cell_down") {
        (col, (row + 1).min(h - 1))
    } else {
        (col, row)
    };
    board.cursor = moved;
    if actions.just_pressed("turn") {
        let (col, row) = moved;
        board.apply(Input::Turn { col, row });
    }
    if actions.just_pressed("undo") {
        board.apply(Input::Undo);
    }
    if actions.just_pressed("clear") {
        board.apply(Input::Clear);
    }
    if actions.just_pressed("open") {
        board.apply(Input::Open);
    }
}

fn press_buttons(
    mut commands: Commands,
    mut board: ResMut<Board>,
    buttons: Query<(&Interaction, &QanatButton), Changed<Interaction>>,
    level: Res<CurrentLevel>,
    mut load: MessageWriter<LoadLevel>,
) {
    for (interaction, QanatButton(action)) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            QanatAction::Open => board.apply(Input::Open),
            QanatAction::Undo => board.apply(Input::Undo),
            QanatAction::Clear => board.apply(Input::Clear),
            QanatAction::Restart => restart(&mut commands, &level, &mut load),
        }
    }
}

/// The water advances one cell every few fixed ticks; reaching the village wins the match.
fn run_water(
    mut board: ResMut<Board>,
    mut ticks: Local<u32>,
    mut reports: MessageWriter<MatchReport>,
) {
    if board.game.phase() == Phase::Flowing {
        *ticks += 1;
        if *ticks >= WATER_TICKS {
            *ticks = 0;
            board.game.step();
        }
    }
    if matches!(board.game.phase(), Phase::Won(_)) && !board.reported {
        board.reported = true;
        reports.write(MatchReport(Report::Progress { side: 0, amount: 1 }));
    }
}

/// Each cell shows its tile (or the spring's outlet) as a centre and an arm per open side,
/// blue once the water is in it; the cursor sits behind the selected cell.
fn draw_cells(
    mut commands: Commands,
    board: Res<Board>,
    cells: Query<(Entity, &QanatCell, &Transform, Option<&Drawn>), Without<CellCursor>>,
    mut cursor: Single<&mut Transform, With<CellCursor>>,
) {
    for (entity, cell, transform, drawn) in &cells {
        let (col, row) = (cell.col as usize, cell.row as usize);
        if (col, row) == board.cursor {
            cursor.translation.x = transform.translation.x;
            cursor.translation.y = transform.translation.y;
        }
        let want = Drawn {
            sides: board.game.sides(col, row),
            wet: board.game.is_wet(col, row),
        };
        if drawn == Some(&want) {
            continue;
        }
        let mut cell_commands = commands.entity(entity);
        cell_commands.despawn_related::<Children>().insert(want);
        if want.sides == 0 {
            continue;
        }
        let color = if want.wet { WATER } else { CLAY };
        cell_commands.with_children(|parent| {
            parent.spawn((
                Sprite::from_color(color, Vec2::splat(18.0)),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
            for (side, offset, size) in [
                (N, Vec2::new(0.0, 18.5), Vec2::new(18.0, 19.0)),
                (S, Vec2::new(0.0, -18.5), Vec2::new(18.0, 19.0)),
                (E, Vec2::new(18.5, 0.0), Vec2::new(19.0, 18.0)),
                (W, Vec2::new(-18.5, 0.0), Vec2::new(19.0, 18.0)),
            ] {
                if want.sides & side != 0 {
                    parent.spawn((
                        Sprite::from_color(color, size),
                        Transform::from_translation(offset.extend(0.1)),
                    ));
                }
            }
        });
    }
}

fn refresh_hud(
    mut board: ResMut<Board>,
    time: Res<Time>,
    mut status: Single<&mut Text, (With<StatusText>, Without<ResultText>)>,
    mut result: Single<&mut Text, (With<ResultText>, Without<StatusText>)>,
    mut banner: Single<&mut Visibility, With<ResultBanner>>,
) {
    if let Some((_, left)) = &mut board.notice {
        *left -= time.delta_secs();
    }
    if board.notice.is_some_and(|(_, left)| left <= 0.0) {
        board.notice = None;
    }
    let game = &board.game;
    let tiles = format!("Tiles {}  |  par {}", game.tiles_used(), game.room().par());
    status.0 = match (board.notice, game.phase()) {
        (Some((text, _)), _) => text.to_string(),
        (_, Phase::Flowing) => "The water is running...".into(),
        (_, Phase::Dry) => "The water stopped. Change the channel and try again".into(),
        _ => tiles,
    };
    if let Phase::Won(tier) = game.phase() {
        let tier = match tier {
            Tier::Gold => "GOLD",
            Tier::Silver => "SILVER",
            Tier::Bronze => "BRONZE",
        };
        result.0 = format!(
            "THE VILLAGE HAS WATER  |  {tier}  |  {} tiles (par {})",
            game.tiles_used(),
            game.room().par()
        );
        **banner = Visibility::Inherited;
    } else {
        **banner = Visibility::Hidden;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xerxes_engine::modules::assets::spawn_scene;
    use xerxes_sim::qanat::fixtures::{SOLUTION, lay};

    /// The scene is the room: its cells build a room the first room's solution clears with
    /// gold, and it has the cursor, the camera marker and the four HUD buttons.
    #[test]
    fn the_scene_is_a_room_the_solution_clears() {
        let mut world = World::new();
        spawn_scene(
            &mut world,
            &crate::assets::scenes::games::solo::qanat_scene::scene(),
        );
        let cells: Vec<QanatCell> = world.query::<&QanatCell>().iter(&world).copied().collect();
        assert_eq!(cells.len(), 63, "9 x 7 cells");
        let room = Room::from_cells(
            cells
                .iter()
                .map(|c| (c.col as usize, c.row as usize, c.ground)),
            PAR,
        )
        .unwrap();
        let mut game = Qanat::new(room);
        lay(&mut game, &SOLUTION);
        game.apply(Input::Open).unwrap();
        while game.step() {}
        assert_eq!(game.phase(), Phase::Won(Tier::Gold));

        // Every cell sits on its spot of a regular grid, CELL apart (row 0 at the top), so
        // what is drawn is what the pointer hits.
        let spots: Vec<(QanatCell, Vec3)> = world
            .query::<(&QanatCell, &Transform)>()
            .iter(&world)
            .map(|(c, t)| (*c, t.translation))
            .collect();
        let origin = spots
            .iter()
            .find(|(c, _)| c.col == 0 && c.row == 0)
            .unwrap()
            .1;
        for (cell, at) in &spots {
            let want = origin.truncate() + Vec2::new(cell.col as f32, -(cell.row as f32)) * CELL;
            assert!(
                at.truncate().distance(want) < 0.01,
                "cell {},{} is at {at}",
                cell.col,
                cell.row
            );
        }

        assert_eq!(world.query::<&CellCursor>().iter(&world).count(), 1);
        assert_eq!(world.query::<&BoardCamera>().iter(&world).count(), 1);
        assert_eq!(world.query::<&QanatButton>().iter(&world).count(), 4);
    }
}
