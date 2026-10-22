//! Arash's Arrow's diorama in Bevy: Qanat rules on a 3D diorama. Drag orbits (editor viewport touch
//! path); tap / click turns a cell. Open / Undo / Clear / Restart are HUD buttons and keys.

use bevy::input::ButtonState;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::WindowEvent;
use xerxes_engine::modules::flow::{CurrentLevel, Flow, LoadLevel, Play};
use xerxes_engine::modules::input::Actions;
use xerxes_engine::modules::matches::{ActiveMatch, MatchReport, Report};
use xerxes_sim::qanat::fixtures::ROOM;
use xerxes_sim::qanat::{E, Ground, Input, N, Phase, Qanat, Rejected, Room, S, Tier, W};

const PAR: u32 = 12;
const WATER_TICKS: u32 = 6;
const CELL: f32 = 1.0;
const CLICK_PX: f32 = 10.0;
const LOOK: f32 = 0.004;
const LOOK_TOUCH: f32 = 0.006;

const CLAY: Color = Color::srgb(0.80, 0.55, 0.33);
const WATER: Color = Color::srgb(0.30, 0.66, 0.92);
const ROCK: Color = Color::srgb(0.38, 0.32, 0.26);
const SOIL: Color = Color::srgb(0.62, 0.48, 0.34);
const SPRING: Color = Color::srgb(0.22, 0.50, 0.78);
const VILLAGE: Color = Color::srgb(0.87, 0.76, 0.52);

/// Root of the diorama; cells are spawned under it when the level opens.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GateBoard;

/// A cell of the diorama (col / row / ground).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GateCell {
    pub col: u8,
    pub row: u8,
    pub ground: Ground,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GateCursor;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GateCamera;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateAction {
    Open,
    Undo,
    Clear,
    Restart,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct GateButton(pub GateAction);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct StatusText;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ResultBanner;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ResultText;

/// Orbit rig (same math as the editor viewport / Pairidaeza).
#[derive(Debug, Clone, Copy)]
struct Orbit {
    pivot: Vec3,
    distance: f32,
    yaw: f32,
    pitch: f32,
}

impl Orbit {
    fn scene_default() -> Self {
        let eye = Vec3::new(0.0, 11.0, 14.0);
        let pivot = Vec3::new(0.0, 0.4, 0.0);
        let offset = eye - pivot;
        let distance = offset.length().max(0.5);
        Self {
            pivot,
            distance,
            yaw: offset.x.atan2(offset.z),
            pitch: (-offset.y / distance).asin().clamp(-1.54, 1.54),
        }
    }

    fn rotation(self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }

    fn position(self) -> Vec3 {
        self.pivot - self.rotation() * Vec3::NEG_Z * self.distance
    }

    fn transform(self) -> Transform {
        Transform::from_translation(self.position()).with_rotation(self.rotation())
    }
}

#[derive(Resource)]
struct Board {
    game: Qanat,
    cursor: (usize, usize),
    orbit: Orbit,
    mouse_drag: Option<MouseButton>,
    touch_on_scene: bool,
    /// Screen position when a press started; if the pointer moves past [`CLICK_PX`], it orbits.
    press_at: Option<Vec2>,
    orbiting: bool,
    notice: Option<(&'static str, f32)>,
    reported: bool,
    fresh: bool,
}

impl Board {
    fn apply(&mut self, input: Input) {
        if let Err(why) = self.game.apply(input) {
            let text = match why {
                Rejected::NotSoil => "Tiles go in the soil only",
                Rejected::OutOfRoom => "Outside the diorama",
                Rejected::Busy if matches!(self.game.phase(), Phase::Won(_)) => {
                    "Cleared — Restart to play again"
                }
                Rejected::Busy => "The water is running",
                Rejected::NothingToUndo => "Nothing to undo",
            };
            self.notice = Some((text, 1.6));
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct Drawn {
    sides: u8,
    wet: bool,
}

pub struct ArashsArrowPlugin;

impl Plugin for ArashsArrowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(Flow::Level), |mut commands: Commands| {
            commands.remove_resource::<Board>();
        })
        .add_systems(
            Update,
            (
                build_board.run_if(not(resource_exists::<Board>)),
                (
                    pointer_and_camera,
                    touch_camera,
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

fn build_board(
    mut commands: Commands,
    added: Query<Entity, Added<GateBoard>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(parent) = added.single() else {
        return;
    };
    let room = match Room::from_rows(&ROOM, PAR) {
        Ok(room) => room,
        Err(err) => {
            error!("arash's arrow room: {err}");
            return;
        }
    };
    let ox = (room.width() as f32 - 1.0) * 0.5;
    let oz = (room.height() as f32 - 1.0) * 0.5;
    commands.entity(parent).with_children(|board| {
        for row in 0..room.height() {
            for col in 0..room.width() {
                let ground = room.ground(col, row);
                let (color, h) = match ground {
                    Ground::Rock => (ROCK, 0.55),
                    Ground::Soil => (SOIL, 0.22),
                    Ground::Spring => (SPRING, 0.40),
                    Ground::Village => (VILLAGE, 0.35),
                };
                board.spawn((
                    GateCell {
                        col: col as u8,
                        row: row as u8,
                        ground,
                    },
                    Mesh3d(meshes.add(Cuboid::new(0.92, h, 0.92))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: color,
                        ..default()
                    })),
                    Transform::from_xyz(col as f32 - ox, h * 0.5, row as f32 - oz),
                ));
            }
        }
    });
    let cursor = (0..room.width())
        .flat_map(|c| (0..room.height()).map(move |r| (c, r)))
        .find(|&(c, r)| room.ground(c, r) == Ground::Soil)
        .unwrap_or((0, 0));
    commands.insert_resource(Board {
        game: Qanat::new(room),
        cursor,
        orbit: Orbit::scene_default(),
        mouse_drag: None,
        touch_on_scene: false,
        press_at: None,
        orbiting: false,
        notice: None,
        reported: false,
        fresh: true,
    });
}

fn over_ui(ui: &Query<&Interaction, With<Button>>) -> bool {
    ui.iter().any(|i| *i != Interaction::None)
}

fn pick_cell(ray: Ray3d, cells: &Query<(&GateCell, &GlobalTransform)>) -> Option<(usize, usize)> {
    let origin = ray.origin;
    let dir = *ray.direction;
    let mut best: Option<(usize, usize, f32)> = None;
    for (cell, at) in cells {
        let center = at.translation();
        let to = center - origin;
        let t = to.dot(dir);
        if t < 0.0 {
            continue;
        }
        let closest = origin + dir * t;
        let dist = closest.distance(center);
        if dist <= CELL * 0.55 {
            if best.is_none_or(|(_, _, d)| dist < d) {
                best = Some((cell.col as usize, cell.row as usize, dist));
            }
        }
    }
    best.map(|(c, r, _)| (c, r))
}

/// Mouse: drag orbits; a short click turns the cell. Wheel dollies. Mirror of editor control +
/// click-to-turn for the puzzle.
fn pointer_and_camera(
    mut board: ResMut<Board>,
    mut events: MessageReader<WindowEvent>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    camera: Query<(&Camera, &GlobalTransform), With<GateCamera>>,
    mut cam_tf: Query<&mut Transform, With<GateCamera>>,
    cells: Query<(&GateCell, &GlobalTransform)>,
    ui: Query<&Interaction, With<Button>>,
    mut pointer: Local<Option<Vec2>>,
) {
    if board.fresh {
        board.fresh = false;
        events.clear();
        if let Ok(mut tf) = cam_tf.single_mut() {
            *tf = board.orbit.transform();
        }
        return;
    }
    let Ok((camera, camera_at)) = camera.single() else {
        return;
    };
    let blocked = over_ui(&ui);

    for event in events.read() {
        match event {
            WindowEvent::CursorMoved(moved) => {
                *pointer = Some(moved.position);
                if let Some(start) = board.press_at {
                    if !board.orbiting && start.distance(moved.position) > CLICK_PX {
                        board.orbiting = true;
                        board.mouse_drag = Some(MouseButton::Left);
                    }
                }
            }
            WindowEvent::MouseButtonInput(input) => {
                if input.button != MouseButton::Left
                    && input.button != MouseButton::Middle
                    && input.button != MouseButton::Right
                {
                    continue;
                }
                if input.state == ButtonState::Pressed && !blocked {
                    if input.button == MouseButton::Left {
                        board.press_at = *pointer;
                        board.orbiting = false;
                    } else {
                        board.mouse_drag = Some(input.button);
                        board.orbiting = true;
                    }
                } else if input.state == ButtonState::Released {
                    if input.button == MouseButton::Left
                        && !board.orbiting
                        && !blocked
                        && let Some(screen) = board.press_at.or(*pointer)
                        && let Ok(ray) = camera.viewport_to_world(camera_at, screen)
                        && let Some((col, row)) = pick_cell(ray, &cells)
                    {
                        board.cursor = (col, row);
                        board.apply(Input::Turn { col, row });
                    }
                    if input.button == MouseButton::Left {
                        board.press_at = None;
                        board.orbiting = false;
                    }
                    if board.mouse_drag == Some(input.button) {
                        board.mouse_drag = None;
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(button) = board.mouse_drag {
        if !buttons.pressed(button) {
            board.mouse_drag = None;
        }
    }

    let delta = motion.delta;
    let wheel = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / 40.0,
    };
    let pan = board.orbit.distance.max(1.0) * 0.0015;
    let drag = board.mouse_drag;
    let orbiting = board.orbiting;
    let cam = &mut board.orbit;
    match drag {
        Some(MouseButton::Left) if orbiting => {
            cam.yaw -= delta.x * LOOK;
            cam.pitch = (cam.pitch - delta.y * LOOK).clamp(-1.54, 1.54);
        }
        Some(MouseButton::Middle) | Some(MouseButton::Right) => {
            let right = cam.rotation() * Vec3::X;
            let up = cam.rotation() * Vec3::Y;
            cam.pivot += (-right * delta.x + up * delta.y) * pan;
        }
        _ => {
            if !blocked && wheel != 0.0 {
                let step = cam.distance.max(2.0) * 0.12 * wheel;
                if cam.distance - step > 0.5 {
                    cam.distance -= step;
                } else {
                    let forward = cam.rotation() * Vec3::NEG_Z;
                    cam.pivot += forward * step;
                }
            }
        }
    }

    if let Ok(mut tf) = cam_tf.single_mut() {
        *tf = board.orbit.transform();
    }
}

/// Touch: one-finger drag orbits; tap turns. Two fingers pan + pinch. Same as editor `touch_camera`.
fn touch_camera(
    mut board: ResMut<Board>,
    touches: Res<Touches>,
    camera: Query<(&Camera, &GlobalTransform), With<GateCamera>>,
    mut cam_tf: Query<&mut Transform, With<GateCamera>>,
    cells: Query<(&GateCell, &GlobalTransform)>,
    ui: Query<&Interaction, With<Button>>,
) {
    let blocked = over_ui(&ui);
    if let Some(t) = touches.iter_just_pressed().next() {
        if touches.iter().count() == 1 {
            board.touch_on_scene = !blocked;
            board.orbiting = false;
            board.press_at = Some(t.position());
        }
    }

    let lifted = touches
        .iter_just_released()
        .chain(touches.iter_just_canceled())
        .next()
        .is_some();
    let fingers: Vec<_> = touches.iter().collect();
    if fingers.is_empty() {
        if lifted
            && board.touch_on_scene
            && !board.orbiting
            && let Some(screen) = board.press_at
            && let Ok((camera, camera_at)) = camera.single()
            && let Ok(ray) = camera.viewport_to_world(camera_at, screen)
            && let Some((col, row)) = pick_cell(ray, &cells)
        {
            board.cursor = (col, row);
            board.apply(Input::Turn { col, row });
        }
        if lifted {
            board.press_at = None;
            board.orbiting = false;
        }
        return;
    }

    if !board.touch_on_scene {
        return;
    }

    match fingers.as_slice() {
        [one] => {
            let delta = one.delta();
            if delta.length() > 0.5 {
                board.orbiting = true;
            }
            if board.orbiting {
                let cam = &mut board.orbit;
                cam.yaw -= delta.x * LOOK_TOUCH;
                cam.pitch = (cam.pitch - delta.y * LOOK_TOUCH).clamp(-1.54, 1.54);
            }
        }
        [a, b, ..] => {
            board.orbiting = true;
            let delta = (a.delta() + b.delta()) * 0.5;
            let now = a.position().distance(b.position());
            let before = (a.position() - a.delta()).distance(b.position() - b.delta());
            let pinch = if now > 1.0 && before > 1.0 {
                before / now
            } else {
                1.0
            };
            let cam = &mut board.orbit;
            let pan = cam.distance.max(1.0) * 0.0015;
            let right = cam.rotation() * Vec3::X;
            let up = cam.rotation() * Vec3::Y;
            cam.pivot += (-right * delta.x + up * delta.y) * pan;
            cam.distance = (cam.distance * pinch).clamp(0.5, 200.0);
        }
        _ => {}
    }

    if let Ok(mut tf) = cam_tf.single_mut() {
        *tf = board.orbit.transform();
    }
}

fn restart(commands: &mut Commands, level: &CurrentLevel, load: &mut MessageWriter<LoadLevel>) {
    load.write(LoadLevel(level.0));
    commands.remove_resource::<ActiveMatch>();
}

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
    buttons: Query<(&Interaction, &GateButton), Changed<Interaction>>,
    level: Res<CurrentLevel>,
    mut load: MessageWriter<LoadLevel>,
) {
    for (interaction, GateButton(action)) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            GateAction::Open => board.apply(Input::Open),
            GateAction::Undo => board.apply(Input::Undo),
            GateAction::Clear => board.apply(Input::Clear),
            GateAction::Restart => restart(&mut commands, &level, &mut load),
        }
    }
}

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

/// Cell top Y in world space (board parent at origin).
fn cell_top(ground: Ground) -> f32 {
    match ground {
        Ground::Rock => 0.55,
        Ground::Soil => 0.22,
        Ground::Spring => 0.40,
        Ground::Village => 0.35,
    }
}

/// Stack on pipe cells: soil → yellow pad (just above the top) → pipes above that.
fn draw_cells(
    mut commands: Commands,
    board: Res<Board>,
    cells: Query<(Entity, &GateCell, &Transform, Option<&Drawn>), Without<GateCursor>>,
    mut cursor: Query<(&mut Transform, &MeshMaterial3d<StandardMaterial>), With<GateCursor>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cursor_styled: Local<bool>,
) {
    if let Ok((mut cursor_tf, mat_h)) = cursor.single_mut() {
        if !*cursor_styled {
            if let Some(mat) = materials.get_mut(&mat_h.0) {
                mat.base_color = Color::srgba(0.98, 0.82, 0.30, 0.45);
                mat.alpha_mode = AlphaMode::Blend;
                mat.unlit = true;
            }
            cursor_tf.scale = Vec3::new(0.90, 0.03, 0.90);
            *cursor_styled = true;
        }
        if let Some((_, cell, transform, _)) = cells
            .iter()
            .find(|(_, c, _, _)| (c.col as usize, c.row as usize) == board.cursor)
        {
            // Sit on the cell top, under the pipe pieces (pipes use local y ≈ 0.38+).
            let top = cell_top(cell.ground);
            cursor_tf.translation =
                Vec3::new(transform.translation.x, top + 0.02, transform.translation.z);
        }
    }

    // Pipes clear the highlighter pad (pad top ≈ cell_top + 0.035).
    const PIPE_Y: f32 = 0.42;
    for (entity, cell, _, drawn) in &cells {
        let (col, row) = (cell.col as usize, cell.row as usize);
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
        let mesh_center = meshes.add(Cuboid::new(0.28, 0.20, 0.28));
        let mat = materials.add(StandardMaterial {
            base_color: color,
            ..default()
        });
        cell_commands.with_children(|parent| {
            parent.spawn((
                Mesh3d(mesh_center),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(0.0, PIPE_Y, 0.0),
            ));
            for (side, offset, scale) in [
                (
                    N,
                    Vec3::new(0.0, PIPE_Y, -0.28),
                    Vec3::new(0.22, 0.14, 0.32),
                ),
                (S, Vec3::new(0.0, PIPE_Y, 0.28), Vec3::new(0.22, 0.14, 0.32)),
                (E, Vec3::new(0.28, PIPE_Y, 0.0), Vec3::new(0.32, 0.14, 0.22)),
                (
                    W,
                    Vec3::new(-0.28, PIPE_Y, 0.0),
                    Vec3::new(0.32, 0.14, 0.22),
                ),
            ] {
                if want.sides & side != 0 {
                    let mesh = meshes.add(Cuboid::from_size(scale));
                    parent.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_translation(offset),
                    ));
                }
            }
        });
    }
}

fn refresh_hud(
    mut board: ResMut<Board>,
    time: Res<Time>,
    mut status: Query<&mut Text, (With<StatusText>, Without<ResultText>)>,
    mut result: Query<&mut Text, (With<ResultText>, Without<StatusText>)>,
    mut banner: Query<&mut Visibility, With<ResultBanner>>,
) {
    if let Some((_, left)) = &mut board.notice {
        *left -= time.delta_secs();
    }
    if board.notice.is_some_and(|(_, left)| left <= 0.0) {
        board.notice = None;
    }
    let game = &board.game;
    let tiles = format!("Tiles {}  |  par {}", game.tiles_used(), game.room().par());
    let line = match (board.notice, game.phase()) {
        (Some((text, _)), _) => text.to_string(),
        (_, Phase::Flowing) => "The river is falling...".into(),
        (_, Phase::Dry) => "Stopped short. Change the channel and open again".into(),
        _ => tiles,
    };
    for mut text in &mut status {
        text.0 = line.clone();
    }
    if let Phase::Won(tier) = game.phase() {
        let tier = match tier {
            Tier::Gold => "GOLD",
            Tier::Silver => "SILVER",
            Tier::Bronze => "BRONZE",
        };
        let msg = format!(
            "THE GATE IS OPEN  |  {tier}  |  {} tiles (par {})",
            game.tiles_used(),
            game.room().par()
        );
        for mut text in &mut result {
            text.0 = msg.clone();
        }
        for mut vis in &mut banner {
            *vis = Visibility::Inherited;
        }
    } else {
        for mut vis in &mut banner {
            *vis = Visibility::Hidden;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xerxes_engine::modules::assets::spawn_scene;
    use xerxes_sim::qanat::fixtures::{SOLUTION, lay};

    #[test]
    fn the_first_room_clears_with_gold_and_the_scene_has_a_board() {
        let room = Room::from_rows(&ROOM, PAR).unwrap();
        let mut game = Qanat::new(room);
        lay(&mut game, &SOLUTION);
        game.apply(Input::Open).unwrap();
        while game.step() {}
        assert_eq!(game.phase(), Phase::Won(Tier::Gold));

        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        spawn_scene(
            &mut world,
            &crate::assets::scenes::games::solo::arashs_arrow_scene::scene(),
        );
        assert_eq!(world.query::<&GateBoard>().iter(&world).count(), 1);
        assert_eq!(world.query::<&GateCamera>().iter(&world).count(), 1);
        assert_eq!(world.query::<&GateButton>().iter(&world).count(), 4);
    }
}
