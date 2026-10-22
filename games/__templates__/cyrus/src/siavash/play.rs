//! Siavash in Bevy. The scene holds the knight, the camera, the road and the header; this steps the
//! rules' [`Run`] on a fixed clock, places the knight, the enemies and the road's chunks from it,
//! reads the player (the move axis: keys, stick and touch stick; and the swipes: mouse drag or
//! touch), and shows the distance, the kills and the crash.
//!
//! **The road runs toward +Z** (a glTF model's front axis, with +Y up), so the camera stands
//! behind the knight and the model needs no turning. The player's right — the screen's right —
//! is therefore -X, which is what [`lane_x`] gives.
//!
//! The screen is the engine's bands: the header at the top (the HUD, the controls' hint and Back)
//! and the content below it, which is what the camera shows. There is no footer, and the bands
//! keep the camera's view inside the content. The camera does not move: it looks straight down the
//! road, so a lane switch only moves the knight across the screen and never changes the angle.
//!
//! The run steps on its own fixed clock through `cyrus_rules::siavash::Pace`, which never takes
//! more than a capped slice of time from one frame: a slow frame (the first ones of a level, where
//! the knight's model is still being built into the world) cannot fast-forward him into something
//! he never saw.
//!
//! The road is laid in chunks of [`CHUNK`] metres. Only the chunk the knight is on and the
//! [`ACTIVE_AHEAD`] chunks after it exist; a chunk is made when it becomes active and removed once
//! the knight has left it. A distance fog fades the far end, so nothing pops in.

use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use cyrus_rules::siavash::{
    FLYER_BOTTOM, FLYER_HEIGHT, FoeKind, GOON_HEIGHT, Input, KNIGHT_STAND, Pace, Phase, PickupKind,
    Run, pack_input, unpack_input,
};
use xerxes_sim::ghost::{Ghost, Recording};
use xerxes_engine::modules::assets::{MaterialDef, MeshDef, Object, spawn};
use xerxes_engine::modules::camera::GameCamera3D;
use xerxes_engine::modules::flow::{Flow, LevelReady, Play};
use xerxes_engine::modules::input::Actions;
use xerxes_engine::modules::storage;

/// The seed of this run: one fixed road for now (the daily road comes with the board).
const SEED: u64 = 1;
/// The length of a road chunk, in metres, and how many chunks ahead of the knight are active.
pub const CHUNK: f32 = 20.0;
pub const ACTIVE_AHEAD: i64 = 3;
/// The fog: the road fades from `FOG_NEAR` to `FOG_FAR` metres from the camera.
const FOG_NEAR: f32 = 25.0;
const FOG_FAR: f32 = 80.0;
const FOG_COLOR: Color = Color::srgb(0.12, 0.13, 0.15);

/// The knight. The scene gives him a model (`prefabs/games/solo/siavash/knight`), standing on
/// the road with his feet at his object's origin, facing +Z — the way he runs.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Knight;

/// The ring at the knight's feet, shown while his shield is on. A model cannot be tinted the way
/// a single-colour cube could, so the shield says so with a light of its own.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ShieldRing;

/// The road: the parent of the enemies and the chunks, at the origin (the knight is at z = 0, the
/// road runs toward +z).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Road;

/// The distance and the kills, in the header.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DistanceText;

/// The crash banner, hidden while the run goes on.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Banner;

/// The crash banner's text.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct BannerText;

/// The header's Restart button: a press starts the run again.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct RestartButton;

/// An enemy's entity, found again by its distance and lane (the rules keep both fixed).
#[derive(Component, Debug, Clone, Copy)]
struct FoeView {
    at: f32,
    lane: u8,
}

/// A pickup's entity, found again by its distance and lane, like an enemy's.
#[derive(Component, Debug, Clone, Copy)]
struct PickupView {
    at: f32,
    lane: u8,
    kind: PickupKind,
}

/// A foe whose strike flash has been started (so it starts one only once).
#[derive(Component, Debug, Clone, Copy)]
struct Struck;

/// The gun's flash for a strike: a short bar ahead of the knight in the strike's lane, which
/// goes after `left` seconds.
#[derive(Component, Debug, Clone, Copy)]
struct Flash {
    left: f32,
}

/// A chunk of road: the stretch of `CHUNK` metres from `index * CHUNK`.
#[derive(Component, Debug, Clone, Copy)]
struct RoadChunk {
    index: i64,
}

/// The road's darker chunk colour, every other one (the prefab holds the lighter one).
const TILE_B: Color = Color::srgb(0.56, 0.47, 0.33);

/// The run being played, while a level with a [`Knight`] is on.
#[derive(Resource)]
struct Course {
    run: Run,
    /// Real time turned into fixed ticks, capped so a slow frame cannot fast-forward the run.
    pace: Pace,
    /// What was asked since the last fixed tick (a request is kept until a tick takes it).
    latch: Input,
    /// The move axis last frame, to find the frame a push goes past [`STEER`].
    steer: Vec2,
    /// The inputs of this attempt so far, one packed byte per tick (what the ghost is made from).
    record: Vec<u8>,
    /// The best attempt on this road, ridden again beside this one (none until one is kept).
    ghost: Option<Ghost<Run>>,
    /// How far the best attempt got, in metres (0 when there is none).
    best: u32,
    /// This attempt's end has been kept (once per attempt).
    kept: bool,
}

/// Where the best attempt is kept, and its format tag.
const GHOST_KEY: &str = "cyrus.siavash.ghost";
const GHOST_TAG: &str = "cyrus1";

/// The ghost knight on the road, drawn from the ghost's run.
#[derive(Component, Debug, Clone, Copy, Default)]
struct GhostKnight;

/// How far the move axis must go to count as a push (a key is 1; a stick has a dead zone below it).
const STEER: f32 = 0.5;

/// The world X of a lane. The road runs toward +Z, so the player's right (and the screen's right,
/// with the camera behind him) is -X: lane 0, the left one, sits at +X.
fn lane_x(lane: u8) -> f32 {
    -Run::lane_x(lane)
}

pub struct SiavashPlugin;

impl Plugin for SiavashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(Flow::Level), teardown).add_systems(
            Update,
            (
                open.run_if(not(resource_exists::<Course>)),
                (
                    restart_pressed,
                    // Only the step is play: the engine holds `Play` back until the level's
                    // ready beat is over (`LevelReady`), while the road, the knight and the HUD
                    // keep being drawn so there is something to be ready for.
                    step_run.in_set(Play),
                    show_shield,
                    spawn_and_despawn,
                    road_chunks,
                    place,
                    spawn_ghost,
                    place_ghost,
                    strike_flash,
                    fade_flash,
                    show_hud,
                )
                    .chain()
                    .run_if(resource_exists::<Course>),
            )
                .chain()
                .run_if(in_state(Flow::Level)),
        );
    }
}

fn teardown(mut commands: Commands) {
    commands.remove_resource::<Course>();
}

/// Starts a run on the first frame of a level that has a knight, and fogs the camera's view.
fn open(
    mut commands: Commands,
    knights: Query<(), With<Knight>>,
    cameras: Query<Entity, With<GameCamera3D>>,
) {
    if knights.is_empty() {
        return;
    }
    // The best attempt on this road, if one was kept (only a run on this seed counts).
    let prior = storage::load(GHOST_KEY)
        .and_then(|text| Recording::decode(GHOST_TAG, &text))
        .filter(|r| r.key == SEED);
    commands.insert_resource(Course {
        run: Run::new(SEED),
        pace: Pace::default(),
        latch: Input::default(),
        steer: Vec2::ZERO,
        record: Vec::new(),
        best: prior.as_ref().map_or(0, |r| r.score),
        ghost: prior.map(|r| Ghost::new(Run::new(SEED), &r, unpack_input)),
        kept: false,
    });
    for camera in &cameras {
        commands.entity(camera).insert(DistanceFog {
            color: FOG_COLOR,
            falloff: FogFalloff::Linear {
                start: FOG_NEAR,
                end: FOG_FAR,
            },
            ..default()
        });
    }
}

/// Reads the player and steps the run for the time that has passed. R starts a new run at any time.
fn step_run(
    time: Res<Time>,
    actions: Res<Actions>,
    mut course: ResMut<Course>,
    mut ready: ResMut<LevelReady>,
    road: Query<(), With<RoadChunk>>,
) {
    if actions.just_pressed("respawn") {
        restart(&mut course, &mut ready);
        return;
    }
    // The run waits for the road to be laid under him (the frame after the level starts): a step
    // taken before the world is there is a step the player never saw.
    if road.is_empty() {
        course.pace.reset();
        return;
    }

    steer(&mut course, actions.vector("move"));
    // The special ability is the global jump (Space, the pad, a double tap): one charge a run.
    course.latch.ability |= actions.just_pressed("jump");
    // A drag on the content (the mouse or a finger) makes the same step as its key.
    if actions.just_pressed("swipe_up") {
        course.latch.jump = true;
    }
    if actions.just_pressed("swipe_down") {
        course.latch.down = true;
    }
    if actions.just_pressed("swipe_left") {
        switch_lane(&mut course, -1);
    }
    if actions.just_pressed("swipe_right") {
        switch_lane(&mut course, 1);
    }

    if !matches!(course.run.phase, Phase::Running) {
        keep_attempt(&mut course);
        course.pace.reset();
        return;
    }
    for _ in 0..course.pace.ticks(time.delta_secs()) {
        if !matches!(course.run.phase, Phase::Running) {
            break;
        }
        let input = std::mem::take(&mut course.latch);
        course.record.push(pack_input(input));
        course.run.step(input);
        if let Some(ghost) = course.ghost.as_mut() {
            ghost.step();
        }
    }
}

/// Turns the move axis into the knight's steps: W jumps, S rolls (or dives in the air), A and D
/// switch lanes. A push past [`STEER`] is one step; holding it does not repeat it. The keys, the
/// left stick and the touch stick all come through the axis.
fn steer(course: &mut Course, now: Vec2) {
    let was = course.steer;
    course.steer = now;
    if now.y > STEER && was.y <= STEER {
        course.latch.jump = true;
    }
    if now.y < -STEER && was.y >= -STEER {
        course.latch.down = true;
    }
    if now.x > STEER && was.x <= STEER {
        switch_lane(course, 1);
    }
    if now.x < -STEER && was.x >= -STEER {
        switch_lane(course, -1);
    }
}

/// One lane to the left (-1) or right (+1), kept on the road's three lanes.
fn switch_lane(course: &mut Course, step: i8) {
    course.latch.lane = (course.latch.lane + step).clamp(-1, 1);
}

/// The ring at the knight's feet is lit while the shield is on.
fn show_shield(course: Res<Course>, mut rings: Query<&mut Visibility, With<ShieldRing>>) {
    let on = course.run.ability_active();
    for mut visibility in &mut rings {
        let want = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
    }
}

/// Keeps one entity for each enemy on the road, and removes those the knight has passed. Every
/// enemy is an instance of its prefab, so what one looks like is said in one place
/// (`prefabs/games/solo/siavash/goon`, `flyer`) and the editor can show and change it.
fn spawn_and_despawn(
    mut commands: Commands,
    course: Res<Course>,
    road: Query<Entity, With<Road>>,
    views: Query<(Entity, &FoeView)>,
    pickup_views: Query<(Entity, &PickupView)>,
) {
    let Ok(road) = road.single() else {
        return;
    };
    for (entity, view) in &pickup_views {
        if !course
            .run
            .pickups
            .iter()
            .any(|p| p.at == view.at && p.lane == view.lane && p.kind == view.kind)
        {
            commands.entity(entity).despawn();
        }
    }
    for pickup in &course.run.pickups {
        if pickup_views
            .iter()
            .any(|(_, v)| v.at == pickup.at && v.lane == pickup.lane && v.kind == pickup.kind)
        {
            continue;
        }
        let prefab = match pickup.kind {
            PickupKind::Coin => crate::assets::prefabs::games::solo::siavash::coin_prefab::prefab,
            PickupKind::Ammo => crate::assets::prefabs::games::solo::siavash::ammo_prefab::prefab,
            PickupKind::PowerUp => {
                crate::assets::prefabs::games::solo::siavash::power_up_prefab::prefab
            }
        };
        let object = Object::prefab(prefab).with(Transform::from_xyz(
            lane_x(pickup.lane),
            PICKUP_Y,
            0.0,
        ));
        let entity = spawn(&mut commands, object, Some(road));
        commands.entity(entity).insert(PickupView {
            at: pickup.at,
            lane: pickup.lane,
            kind: pickup.kind,
        });
    }
    for (entity, view) in &views {
        if !course
            .run
            .foes
            .iter()
            .any(|f| f.at == view.at && f.lane == view.lane)
        {
            commands.entity(entity).despawn();
        }
    }
    for foe in &course.run.foes {
        if views
            .iter()
            .any(|(_, view)| view.at == foe.at && view.lane == foe.lane)
        {
            continue;
        }
        let prefab = match foe.kind {
            FoeKind::Goon if foe.shielded => {
                crate::assets::prefabs::games::solo::siavash::shielded_goon_prefab::prefab
            }
            FoeKind::Goon => crate::assets::prefabs::games::solo::siavash::goon_prefab::prefab,
            FoeKind::Flyer => crate::assets::prefabs::games::solo::siavash::flyer_prefab::prefab,
        };
        // Its lane; `place` puts it at its distance along the road every frame.
        let object = Object::prefab(prefab).with(Transform::from_xyz(
            lane_x(foe.lane),
            foe_y(foe.kind),
            0.0,
        ));
        let entity = spawn(&mut commands, object, Some(road));
        commands.entity(entity).insert(FoeView {
            at: foe.at,
            lane: foe.lane,
        });
    }
}

/// The gun's flash colour and how long it shows (seconds).
const FLASH: Color = Color::srgb(1.0, 0.92, 0.5);
const FLASH_TIME: f32 = 0.15;

/// How high a pickup floats (its middle, above the road).
const PICKUP_Y: f32 = 0.8;

/// How high an enemy's middle sits: a goon stands on the road, a flyer hangs over it. The prefabs
/// are built at these heights; the rules decide what the knight actually hits.
fn foe_y(kind: FoeKind) -> f32 {
    match kind {
        FoeKind::Goon => GOON_HEIGHT / 2.0,
        FoeKind::Flyer => FLYER_BOTTOM + FLYER_HEIGHT / 2.0,
    }
}

/// The chunk the knight is on, and the chunks ahead of it: these exist; the others do not. Each one
/// is an instance of the road-chunk prefab, every other one in the darker colour.
fn road_chunks(
    mut commands: Commands,
    course: Res<Course>,
    road: Query<Entity, With<Road>>,
    chunks: Query<(Entity, &RoadChunk)>,
) {
    let Ok(road) = road.single() else {
        return;
    };
    let current = (course.run.dist / CHUNK).floor() as i64;
    let last = current + ACTIVE_AHEAD;
    for (entity, chunk) in &chunks {
        if chunk.index < current || chunk.index > last {
            commands.entity(entity).despawn();
        }
    }
    for index in current..=last {
        if chunks.iter().any(|(_, chunk)| chunk.index == index) {
            continue;
        }
        let mut object =
            Object::prefab(crate::assets::prefabs::games::solo::siavash::road_tile_prefab::prefab);
        if index.rem_euclid(2) != 0 {
            object = object.with(MaterialDef {
                color: TILE_B,
                texture: None,
            });
        }
        let entity = spawn(&mut commands, object, Some(road));
        commands.entity(entity).insert(RoadChunk { index });
    }
}

/// Places the knight (in his lane, feet on the road, lower while rolling), the enemies (at their
/// distance ahead of him; a killed one is hidden) and the road's chunks (each at its place ahead).
fn place(
    course: Res<Course>,
    mut knight: Query<
        &mut Transform,
        (With<Knight>, Without<FoeView>, Without<RoadChunk>, Without<PickupView>),
    >,
    mut foes: Query<
        (&FoeView, &mut Transform, &mut Visibility),
        (Without<Knight>, Without<RoadChunk>, Without<PickupView>),
    >,
    mut pickups: Query<
        (&PickupView, &mut Transform, &mut Visibility),
        (Without<Knight>, Without<RoadChunk>, Without<FoeView>),
    >,
    mut chunks: Query<
        (&RoadChunk, &mut Transform),
        (Without<Knight>, Without<FoeView>, Without<PickupView>),
    >,
) {
    let run = &course.run;
    for mut transform in &mut knight {
        let height = run.height();
        transform.translation = Vec3::new(lane_x(run.lane), run.y, 0.0);
        // The model stands as tall as the rules say he is: a roll presses him down. An animation
        // takes this over once the knight has one.
        transform.scale = Vec3::new(1.0, height / KNIGHT_STAND, 1.0);
    }
    for (view, mut transform, mut visibility) in &mut foes {
        transform.translation.z = view.at - run.dist;
        let dead = run
            .foes
            .iter()
            .find(|f| f.at == view.at && f.lane == view.lane)
            .is_some_and(|f| f.dead);
        *visibility = if dead {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    for (view, mut transform, mut visibility) in &mut pickups {
        transform.translation.z = view.at - run.dist;
        let taken = run
            .pickups
            .iter()
            .find(|p| p.at == view.at && p.lane == view.lane && p.kind == view.kind)
            .is_some_and(|p| p.taken);
        *visibility = if taken {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    for (chunk, mut transform) in &mut chunks {
        let middle = chunk.index as f32 * CHUNK + CHUNK / 2.0;
        transform.translation.z = middle - run.dist;
    }
}

/// A strike shows the gun's flash: a bar ahead of the knight in the lane the foe was in, once.
fn strike_flash(
    mut commands: Commands,
    course: Res<Course>,
    knight: Query<Entity, With<Knight>>,
    foes: Query<(Entity, &FoeView), Without<Struck>>,
) {
    let Ok(knight) = knight.single() else {
        return;
    };
    for (entity, view) in &foes {
        let Some(foe) = course
            .run
            .foes
            .iter()
            .find(|f| f.at == view.at && f.lane == view.lane)
        else {
            continue;
        };
        if !foe.dead {
            continue;
        }
        commands.entity(entity).insert(Struck);
        // Relative to the knight, who stands at the origin of his lane.
        let offset = lane_x(foe.lane) - lane_x(course.run.lane);
        let object = Object::new("Strike Flash")
            .with(MeshDef::Cuboid {
                size: Vec3::new(0.12, 0.12, 6.0),
            })
            .with(MaterialDef {
                color: FLASH,
                texture: None,
            })
            .with(Transform::from_xyz(offset, 1.0, 3.0))
            .with(Flash { left: FLASH_TIME });
        spawn(&mut commands, object, Some(knight));
    }
}

/// Shortens each flash by the frame's time, and removes it when it is gone.
fn fade_flash(mut commands: Commands, time: Res<Time>, mut flashes: Query<(Entity, &mut Flash)>) {
    for (entity, mut flash) in &mut flashes {
        flash.left -= time.delta_secs();
        if flash.left <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// The distance and the kills in the header, and the crash banner with what the knight ran into.
fn show_hud(
    course: Res<Course>,
    ready: Res<LevelReady>,
    mut distance: Query<&mut Text, (With<DistanceText>, Without<BannerText>)>,
    mut banner: Query<&mut Visibility, With<Banner>>,
    mut banner_text: Query<&mut Text, (With<BannerText>, Without<DistanceText>)>,
) {
    let run = &course.run;
    for mut text in &mut distance {
        text.0 = format!(
            "{} m   {} down   {} coins x{}   {} ammo   chain {}",
            run.metres(),
            run.kills,
            run.coins,
            run.multiplier(),
            run.ammo,
            run.combo,
        );
    }
    let crashed = match run.phase {
        Phase::Running => None,
        Phase::Crashed(kind) => Some(kind),
    };
    let waiting = crashed.is_none() && !ready.is_live();
    for mut visibility in &mut banner {
        *visibility = if crashed.is_some() || waiting {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut text in &mut banner_text {
        text.0 = match crashed {
            Some(kind) => format!(
                "{} at {} m. Press R to run again.",
                cause(kind),
                run.metres()
            ),
            None if waiting => format!("Get ready - {}", ready.left().ceil() as u32),
            None => String::new(),
        };
    }
}

fn cause(kind: FoeKind) -> &'static str {
    match kind {
        FoeKind::Goon => "Ran into a goon",
        FoeKind::Flyer => "Hit by a flyer",
    }
}

/// Starts the run again from the beginning of the road, beat and all: a retry is a fresh run,
/// and the player has just been looking at a crash banner rather than at the lane.
fn restart(course: &mut Course, ready: &mut LevelReady) {
    course.run = Run::new(SEED);
    course.latch = Input::default();
    course.record.clear();
    course.kept = false;
    course.pace.reset();
    ready.restart();
}

/// Keeps a finished attempt as the best on this road when it went further than the best so far, in
/// storage (as Chapar keeps its ghost), and makes it the ghost. Once per attempt.
fn keep_attempt(course: &mut Course) {
    if course.kept {
        return;
    }
    course.kept = true;
    let metres = course.run.metres();
    if metres > course.best {
        course.best = metres;
        let recording = Recording {
            key: SEED,
            score: metres,
            inputs: course.record.clone(),
        };
        storage::save(GHOST_KEY, &recording.encode(GHOST_TAG));
        course.ghost = Some(Ghost::new(Run::new(SEED), &recording, unpack_input));
    }
}

/// Lays the ghost knight on the road once there is a ghost to show.
fn spawn_ghost(
    mut commands: Commands,
    course: Res<Course>,
    road: Query<Entity, With<Road>>,
    ghosts: Query<(), With<GhostKnight>>,
) {
    let Ok(road) = road.single() else {
        return;
    };
    if course.ghost.is_none() || !ghosts.is_empty() {
        return;
    }
    let object = Object::prefab(crate::assets::prefabs::games::solo::siavash::ghost_knight_prefab::prefab);
    let entity = spawn(&mut commands, object, Some(road));
    commands.entity(entity).insert(GhostKnight);
}

/// Places the ghost knight: its lane and height, and its distance from the knight along the road
/// (ahead of him when it got further, behind when it did not).
fn place_ghost(course: Res<Course>, mut ghosts: Query<&mut Transform, With<GhostKnight>>) {
    let Some(ghost) = course.ghost.as_ref() else {
        return;
    };
    let run = ghost.sim();
    for mut transform in &mut ghosts {
        transform.translation = Vec3::new(lane_x(run.lane), run.y, run.dist - course.run.dist);
        transform.scale = Vec3::new(1.0, run.height() / KNIGHT_STAND, 1.0);
    }
}

/// The header's Restart button: a press starts the run again.
fn restart_pressed(
    buttons: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
    mut course: ResMut<Course>,
    mut ready: ResMut<LevelReady>,
) {
    if buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        restart(&mut course, &mut ready);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;
    use xerxes_engine::modules::input::{ActionsPlugin, InputMap};

    /// The two ways a run can be lost before it is seen, both reported from a phone: the rules
    /// stepping while the world is still being built, and one slow frame (the first frames of a
    /// level are the slowest there are) spending its whole delta on road the player never saw.
    #[test]
    fn the_run_waits_for_the_road_and_no_frame_fast_forwards_it() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::input::InputPlugin))
            .add_plugins(ActionsPlugin(InputMap::new()))
            // Every frame of this test took two seconds, as a frame building a heavy model does.
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(2)))
            // The level's ready beat is the engine's and has its own test (tests/engine/flow.rs);
            // this one is about the road and the clock, so the run is already live.
            .insert_resource(LevelReady::of(0.0))
            .insert_resource(Course {
                run: Run::new(SEED),
                pace: Pace::default(),
                latch: Input::default(),
                steer: Vec2::ZERO,
                record: Vec::new(),
                ghost: None,
                best: 0,
                kept: false,
            })
            .add_systems(Update, step_run);

        app.update();
        assert_eq!(
            app.world().resource::<Course>().run.dist,
            0.0,
            "the run waits for the road to be laid under him"
        );

        app.world_mut().spawn(RoadChunk { index: 0 });
        app.update();
        let dist = app.world().resource::<Course>().run.dist;
        assert!(
            dist > 0.0 && dist < 2.0,
            "a two-second frame moves him a step, not {dist} m of road"
        );
    }
}
