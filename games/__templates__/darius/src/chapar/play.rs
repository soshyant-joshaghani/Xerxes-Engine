//! Chapar in Bevy. The scene holds the rider, the road, the scenery and the HUD; this builds the
//! rules' [`Run`] when a level with a [`Rider`] starts, samples jump and duck once per fixed tick,
//! scrolls the world past the (fixed) rider, draws each obstacle as sprites while it is near, and
//! tells the engine's match when the run is over so it ranks the result.
//!
//! The rider never moves sideways on screen: the road, the dashes, the far mesas and the
//! obstacles move left. (The shared layout module owns the game camera's position, so the world
//! scrolls instead of the camera following.)

use std::collections::HashSet;

use bevy::prelude::*;
use xerxes_engine::modules::flow::Flow;
use xerxes_engine::modules::input::Actions;
use xerxes_engine::modules::matches::{ActiveMatch, MatchPause, MatchReport, Phase as MatchPhase, Report};
use xerxes_engine::modules::storage;
use xerxes_engine::modules::ui::TouchSeen;

use super::daily;
use super::ghost::{self, Ghost, Recording};
use super::rules::{
    Cause, DARIC_R, Daric, Input, Kind, Obstacle, Phase, Run, STATION_W, Station, TICKS_PER_SECOND,
    TOKEN_HIGH, TOKEN_LOW, TOKEN_W,
};

/// The rider's place on screen, and the road's surface (the game canvas's own pixels).
const RIDER_X: f32 = -300.0;
const ROAD_Y: f32 = -110.0;
/// Where the best run of the day is kept (one slot: the latest day played).
const GHOST_KEY: &str = "darius.chapar.ghost";
/// After a crash, how long a jump press is ignored (so the button mash that failed the jump does
/// not restart the run at once).
const RETRY_AFTER: u32 = TICKS_PER_SECOND * 3 / 4;

const STONE: Color = Color::srgb(0.55, 0.42, 0.30);
const STONE_TOP: Color = Color::srgb(0.69, 0.55, 0.40);
const BEAM: Color = Color::srgb(0.70, 0.18, 0.16);
const CHAIN: Color = Color::srgb(0.35, 0.37, 0.42);
const PIT: Color = Color::srgb(0.04, 0.05, 0.08);
const POST: Color = Color::srgb(0.36, 0.25, 0.17);
const PENNANT: Color = Color::srgb(0.20, 0.55, 0.40);
const TOKEN: Color = Color::srgb(0.98, 0.82, 0.30);
const DARIC: Color = Color::srgb(0.95, 0.78, 0.22);

/// The rider: horse and courier as one body. Its transform is set from the run (feet on the
/// road), and it is squashed while ducking.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Rider;

/// The ghost of the day's best run, drawn like the rider but faint. Hidden when there is none.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GhostRider;

/// The object obstacles are spawned under (the road's origin in the game canvas).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Road;

/// Scenery that scrolls with the run: it sits `offset` along a loop `span` wide and moves at
/// `factor` times the road's speed (1 for the road's dashes, less for the far mesas).
#[derive(Component, Debug, Clone, Copy)]
pub struct Parallax {
    pub factor: f32,
    pub offset: f32,
    pub span: f32,
}

/// An obstacle's sprites, by the obstacle's id.
#[derive(Component, Debug, Clone, Copy)]
struct ObstacleSprite(u32);

/// A relay station's sprites, by the station's id.
#[derive(Component, Debug, Clone, Copy)]
struct StationSprite(u32);

/// A station's token (the fresh horse): removed when the rider takes it.
#[derive(Component, Debug, Clone, Copy)]
struct StationToken(u32);

/// A daric sprite, by the daric's id.
#[derive(Component, Debug, Clone, Copy)]
struct DaricSprite(u32);

/// The day's road in the header ("Daily road 2026-10-13").
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DayText;

/// The metres and best text.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DistanceText;

/// The Pause / Resume label.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PauseLabel;

/// The full-screen banner (paused, countdown, result).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Banner;

/// The banner's text.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct BannerText;

/// The controls hint in the header.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HintText;

/// What a HUD button does.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaparAction {
    TogglePause,
    Restart,
}

/// A HUD button.
#[derive(Component, Debug, Clone, Copy)]
pub struct HudButton(pub ChaparAction);

/// The run being played, while a level with a [`Rider`] is on.
#[derive(Resource)]
struct Course {
    run: Run,
    /// What was held since the last fixed tick (a press is kept until a tick takes it).
    latch: Input,
    /// Whether the match was told the run is over.
    reported: bool,
    /// Fixed ticks since the run ended.
    over_for: u32,
    /// The day this road is made for (days since 1970-01-01 UTC).
    day: u64,
    /// The inputs of this run so far, one packed byte per tick (what a ghost is made from).
    record: Vec<u8>,
    /// The day's best run so far, ridden again beside this one.
    ghost: Option<Ghost>,
    /// How far that best run got (0 when there is none): this run beats it or not.
    prior_metres: u32,
}

impl Course {
    /// A fresh run on today's road. A retry asks the clock again, so a run begun after midnight
    /// is on the new day's road.
    fn new(store: &mut GhostStore) -> Self {
        let day = daily::today();
        let seed = daily::seed_of_day(day);
        let prior = store.for_day(day);
        Self {
            day,
            run: Run::new(seed),
            latch: Input::default(),
            reported: false,
            over_for: 0,
            record: Vec::new(),
            prior_metres: prior.as_ref().map_or(0, |r| r.metres),
            ghost: prior.map(|r| Ghost::new(seed, &r)),
        }
    }
}

/// The saved best run, read from storage once and kept up to date. Only the latest day played
/// is kept: a new day's first run starts without a ghost.
#[derive(Resource, Default)]
struct GhostStore {
    saved: Option<Recording>,
    loaded: bool,
}

impl GhostStore {
    /// The best run on `day`'s road, if one was ridden.
    fn for_day(&mut self, day: u64) -> Option<Recording> {
        if !self.loaded {
            self.loaded = true;
            self.saved = storage::load(GHOST_KEY).and_then(|text| Recording::decode(&text));
        }
        self.saved.clone().filter(|r| r.day == day)
    }

    /// Keeps `recording` as the best run, here and in storage.
    fn keep(&mut self, recording: Recording) {
        storage::save(GHOST_KEY, &recording.encode());
        self.saved = Some(recording);
    }
}

/// The furthest run this session, in metres.
#[derive(Resource, Debug, Clone, Copy, Default)]
struct Best {
    metres: u32,
    /// The run that just ended set it.
    new: bool,
}

pub struct ChaparPlugin;

impl Plugin for ChaparPlugin {
    fn build(&self, app: &mut App) {
        // The run is stepped on the same 60 Hz clock the physics uses.
        app.insert_resource(Time::<Fixed>::from_hz(TICKS_PER_SECOND as f64))
            .init_resource::<Best>()
            .init_resource::<GhostStore>()
            .add_message::<ChaparAction>()
            .add_systems(OnExit(Flow::Level), |mut commands: Commands| {
                commands.remove_resource::<Course>();
            })
            .add_systems(
                Update,
                (
                    build_course.run_if(not(resource_exists::<Course>)),
                    (
                        sample_input,
                        press_buttons,
                        apply_actions,
                        sync_obstacles,
                        sync_stations,
                        sync_darics,
                        scroll_view,
                        place_ghost,
                        refresh_hud,
                        touch_hint,
                    )
                        .chain()
                        .run_if(resource_exists::<Course>),
                )
                    .run_if(in_state(Flow::Level)),
            )
            .add_systems(
                FixedUpdate,
                step_run.run_if(resource_exists::<Course>),
            );
    }
}

/// A level with a rider starts a fresh run.
fn build_course(
    mut commands: Commands,
    mut store: ResMut<GhostStore>,
    mut best: ResMut<Best>,
    _rider: Single<Entity, With<Rider>>,
) {
    let course = Course::new(&mut store);
    best.metres = best.metres.max(course.prior_metres);
    commands.insert_resource(course);
}

/// Jump and duck, from whichever device is in use (keys, pad, the footer's touch buttons).
fn sample_input(actions: Res<Actions>, mut course: ResMut<Course>) {
    course.latch.jump_pressed |= actions.just_pressed("jump");
    course.latch.jump_held = actions.pressed("jump");
    course.latch.duck_held = actions.pressed("duck");
}

fn press_buttons(
    buttons: Query<(&Interaction, &HudButton), Changed<Interaction>>,
    mut actions: MessageWriter<ChaparAction>,
) {
    for (interaction, HudButton(action)) in &buttons {
        if *interaction == Interaction::Pressed {
            actions.write(*action);
        }
    }
}

/// The Pause button, Restart (button, `R`, or a jump a moment after the crash).
///
/// The Esc / P pause key is not handled here: Ahriman's Horde's logic module toggles
/// [`MatchPause`] for every level, so a second toggle would cancel it.
fn apply_actions(
    mut commands: Commands,
    mut messages: MessageReader<ChaparAction>,
    mut pause: ResMut<MatchPause>,
    actions: Res<Actions>,
    mut course: ResMut<Course>,
    mut best: ResMut<Best>,
    mut store: ResMut<GhostStore>,
    obstacles: Query<Entity, Or<(With<ObstacleSprite>, With<StationSprite>)>>,
) {
    let mut restart = actions.just_pressed("respawn")
        || (course.run.is_over()
            && course.over_for >= RETRY_AFTER
            && actions.just_pressed("jump"));
    for action in messages.read() {
        match action {
            ChaparAction::TogglePause => pause.0 = !pause.0,
            ChaparAction::Restart => restart = true,
        }
    }
    if restart {
        *course = Course::new(&mut store);
        best.metres = best.metres.max(course.prior_metres);
        best.new = false;
        pause.0 = false;
        for entity in &obstacles {
            commands.entity(entity).despawn();
        }
        // A fresh match (and its countdown) begins by itself.
        commands.remove_resource::<ActiveMatch>();
    }
}

/// One fixed tick of the run, while the match is playing and not paused. When it ends, the
/// match is told the rider is down so it ranks the result.
fn step_run(
    mut course: ResMut<Course>,
    active: Option<Res<ActiveMatch>>,
    pause: Res<MatchPause>,
    mut best: ResMut<Best>,
    mut store: ResMut<GhostStore>,
    mut reports: MessageWriter<MatchReport>,
) {
    let playing = active.is_some_and(|m| m.0.phase() == MatchPhase::Playing);
    if course.run.is_over() {
        course.over_for = course.over_for.saturating_add(1);
    } else if playing && !pause.0 {
        let input = course.latch;
        course.run.step(input);
        // What was ridden is what a ghost is made from; the best of the day's ghost is ridden in
        // step with it.
        course.record.push(ghost::pack(input));
        if let Some(ghost) = course.ghost.as_mut() {
            ghost.step();
        }
        // A press is taken by one tick; holds stay.
        course.latch.jump_pressed = false;
    }
        if course.run.is_over() && !course.reported {
        course.reported = true;
        let metres = course.run.metres();
        let score = course.run.score();
        best.metres = best.metres.max(metres);
        // Beating the day's best (not only this session's) makes this run the new ghost.
        if metres > course.prior_metres {
            best.new = true;
            store.keep(Recording {
                day: course.day,
                metres,
                inputs: course.record.clone(),
            });
        }
        // Solo x Score: the points are the result; then the rider going down ends the round.
        if score > 0 {
            reports.write(MatchReport(Report::Score {
                side: 0,
                amount: score,
            }));
        }
        reports.write(MatchReport(Report::Down { side: 0, player: 0 }));
    }
}

/// Draws obstacles as they come near and drops them when they are behind.
fn sync_obstacles(
    mut commands: Commands,
    course: Res<Course>,
    road: Single<Entity, With<Road>>,
    shown: Query<(Entity, &ObstacleSprite)>,
) {
    let live: HashSet<u32> = course.run.obstacles().iter().map(|o| o.id).collect();
    let mut have = HashSet::new();
    for (entity, ObstacleSprite(id)) in &shown {
        if live.contains(id) {
            have.insert(*id);
        } else {
            commands.entity(entity).despawn();
        }
    }
    for o in course.run.obstacles() {
        if !have.contains(&o.id) {
            spawn_obstacle(&mut commands, *road, o);
        }
    }
}

/// Draws relay stations as they come near, drops them when behind, and takes a token off once
/// the rider has it.
fn sync_stations(
    mut commands: Commands,
    course: Res<Course>,
    road: Single<Entity, With<Road>>,
    shown: Query<(Entity, &StationSprite)>,
    tokens: Query<(Entity, &StationToken)>,
) {
    let stations = course.run.stations();
    let mut have = HashSet::new();
    for (entity, StationSprite(id)) in &shown {
        if stations.iter().any(|s| s.id == *id) {
            have.insert(*id);
        } else {
            commands.entity(entity).despawn();
        }
    }
    for st in stations {
        if !have.contains(&st.id) {
            spawn_station(&mut commands, *road, st);
        }
    }
    for (entity, StationToken(id)) in &tokens {
        if stations.iter().any(|s| s.id == *id && s.collected) {
            commands.entity(entity).despawn();
        }
    }
}

/// Draws darics as they come near; drops them when collected or left behind.
fn sync_darics(
    mut commands: Commands,
    course: Res<Course>,
    road: Single<Entity, With<Road>>,
    shown: Query<(Entity, &DaricSprite)>,
) {
    let live: HashSet<u32> = course
        .run
        .darics()
        .iter()
        .filter(|d| !d.collected)
        .map(|d| d.id)
        .collect();
    let mut have = HashSet::new();
    for (entity, DaricSprite(id)) in &shown {
        if live.contains(id) {
            have.insert(*id);
        } else {
            commands.entity(entity).despawn();
        }
    }
    for d in course.run.darics().iter().filter(|d| !d.collected) {
        if !have.contains(&d.id) {
            spawn_daric(&mut commands, *road, d);
        }
    }
}

fn spawn_daric(commands: &mut Commands, road: Entity, d: &Daric) {
    let size = DARIC_R * 2.0;
    commands.entity(road).with_children(|parent| {
        parent.spawn((
            DaricSprite(d.id),
            Sprite {
                color: DARIC,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_xyz(RIDER_X + 4000.0, ROAD_Y + d.y, 1.5),
            Visibility::default(),
        ));
    });
}

/// A station: two posts with a pennant between them, and the token held high over its middle.
fn spawn_station(commands: &mut Commands, road: Entity, st: &Station) {
    let w = STATION_W;
    let token_y = (TOKEN_LOW + TOKEN_HIGH) / 2.0;
    commands.entity(road).with_children(|parent| {
        parent
            .spawn((
                StationSprite(st.id),
                // Off screen until the next scroll puts it at its place.
                Transform::from_xyz(RIDER_X + 4000.0, ROAD_Y, 0.5),
                Visibility::default(),
            ))
            .with_children(|p| {
                for x in [0.0, w] {
                    p.spawn((
                        Sprite::from_color(POST, Vec2::new(8.0, 120.0)),
                        Transform::from_xyz(x, 60.0, 0.0),
                    ));
                }
                p.spawn((
                    Sprite::from_color(PENNANT, Vec2::new(w, 22.0)),
                    Transform::from_xyz(w / 2.0, 110.0, 0.1),
                ));
                if !st.collected {
                    p.spawn((
                        StationToken(st.id),
                        Sprite::from_color(
                            TOKEN,
                            Vec2::new(TOKEN_W * 0.7, TOKEN_HIGH - TOKEN_LOW),
                        ),
                        Transform::from_xyz(w / 2.0, token_y, 0.2),
                    ));
                }
            });
    });
}

/// An obstacle's sprites, as children of one entity at its left edge on the road.
fn spawn_obstacle(commands: &mut Commands, road: Entity, o: &Obstacle) {
    let (w, h) = (o.w, o.h);
    commands.entity(road).with_children(|parent| {
        parent
            .spawn((
                ObstacleSprite(o.id),
                // Off screen until the next scroll puts it at its place.
                Transform::from_xyz(RIDER_X + 4000.0, ROAD_Y, 1.0),
                Visibility::default(),
            ))
            .with_children(|p| match o.kind {
                Kind::Block => {
                    p.spawn((
                        Sprite::from_color(STONE, Vec2::new(w, h)),
                        Transform::from_xyz(w / 2.0, h / 2.0, 0.0),
                    ));
                    p.spawn((
                        Sprite::from_color(STONE_TOP, Vec2::new(w, 6.0)),
                        Transform::from_xyz(w / 2.0, h - 3.0, 0.1),
                    ));
                }
                Kind::Gate => {
                    // A banner hung on a chain from above: duck under it.
                    p.spawn((
                        Sprite::from_color(BEAM, Vec2::new(w, 40.0)),
                        Transform::from_xyz(w / 2.0, h + 20.0, 0.0),
                    ));
                    p.spawn((
                        Sprite::from_color(CHAIN, Vec2::new(4.0, 600.0)),
                        Transform::from_xyz(w / 2.0, h + 40.0 + 300.0, -0.1),
                    ));
                }
                Kind::Pit => {
                    p.spawn((
                        Sprite::from_color(PIT, Vec2::new(w, 260.0)),
                        Transform::from_xyz(w / 2.0, -130.0, 0.0),
                    ));
                }
            });
    });
}

/// How far to draw the run scrolled: its distance, carried on by the part of a fixed tick that
/// has passed (so the motion is smooth at any frame rate).
fn drawn_scroll(run: &Run, ahead: f32) -> f32 {
    run.x() + run.speed() * ahead / TICKS_PER_SECOND as f32
}

/// How high to draw the rider above the road. Above the road it is carried on by its speed;
/// below it (falling into a pit) too.
fn drawn_height(run: &Run, ahead: f32) -> f32 {
    let carried = run.y() + run.vy() * ahead / TICKS_PER_SECOND as f32;
    if run.grounded() {
        0.0
    } else if run.y() >= 0.0 {
        carried.max(0.0)
    } else {
        carried
    }
}

/// Puts the rider, the obstacles and the scenery where the run says. The run advances in fixed
/// ticks; the drawn position is carried on by the fraction of a tick that has passed, so the
/// motion is smooth at any frame rate.
fn scroll_view(
    course: Res<Course>,
    fixed: Res<Time<Fixed>>,
    pause: Res<MatchPause>,
    mut rider: Single<
        &mut Transform,
        (
            With<Rider>,
            Without<ObstacleSprite>,
            Without<Parallax>,
            Without<StationSprite>,
            Without<DaricSprite>,
        ),
    >,
    mut obstacles: Query<
        (&ObstacleSprite, &mut Transform),
        (
            Without<Rider>,
            Without<Parallax>,
            Without<StationSprite>,
            Without<DaricSprite>,
        ),
    >,
    mut stations: Query<
        (&StationSprite, &mut Transform),
        (
            Without<Rider>,
            Without<ObstacleSprite>,
            Without<Parallax>,
            Without<DaricSprite>,
        ),
    >,
    mut darics: Query<
        (&DaricSprite, &mut Transform),
        (
            Without<Rider>,
            Without<ObstacleSprite>,
            Without<Parallax>,
            Without<StationSprite>,
        ),
    >,
    mut scenery: Query<
        (&Parallax, &mut Transform),
        (
            Without<Rider>,
            Without<ObstacleSprite>,
            Without<StationSprite>,
            Without<DaricSprite>,
        ),
    >,
) {
    let run = &course.run;
    let moving = !run.is_over() && !pause.0;
    let ahead = if moving { fixed.overstep_fraction() } else { 0.0 };
    let scroll = drawn_scroll(run, ahead);
    let height = drawn_height(run, ahead);

    rider.translation = Vec3::new(RIDER_X, ROAD_Y + height, 2.0);
    rider.scale.y = if run.ducking() {
        super::rules::DUCK_HEIGHT / super::rules::STAND_HEIGHT
    } else {
        1.0
    };

    for (ObstacleSprite(id), mut transform) in &mut obstacles {
        if let Some(o) = run.obstacles().iter().find(|o| o.id == *id) {
            transform.translation.x = RIDER_X + (o.x - scroll);
        }
    }
    for (StationSprite(id), mut transform) in &mut stations {
        if let Some(st) = run.stations().iter().find(|s| s.id == *id) {
            transform.translation.x = RIDER_X + (st.x - scroll);
        }
    }
    for (DaricSprite(id), mut transform) in &mut darics {
        if let Some(d) = run.darics().iter().find(|d| d.id == *id) {
            transform.translation.x = RIDER_X + (d.x - scroll);
            transform.translation.y = ROAD_Y + d.y;
        }
    }
    for (parallax, mut transform) in &mut scenery {
        let travelled = scroll * parallax.factor;
        let wrapped = (parallax.offset - travelled).rem_euclid(parallax.span);
        transform.translation.x = wrapped - parallax.span / 2.0;
    }
}

/// Puts the ghost where its run is, relative to the player: ahead of the rider on the screen
/// when it is further along the road, behind when it is not. Hidden when there is none.
fn place_ghost(
    course: Res<Course>,
    fixed: Res<Time<Fixed>>,
    pause: Res<MatchPause>,
    mut ghost: Single<(&mut Transform, &mut Visibility), With<GhostRider>>,
) {
    let (transform, visibility) = &mut *ghost;
    let Some(g) = course.ghost.as_ref() else {
        **visibility = Visibility::Hidden;
        return;
    };
    **visibility = Visibility::Inherited;
    let moving = !course.run.is_over() && !pause.0;
    let ahead = if moving { fixed.overstep_fraction() } else { 0.0 };
    let mine = drawn_scroll(&course.run, ahead);
    // A ghost whose recording ended (it crashed there) stands where it fell, and the player's
    // run goes on past it. When the player's run ends the ghost is stepped no more.
    let gone = g.run().is_over();
    let theirs = if gone {
        g.run().x()
    } else {
        drawn_scroll(g.run(), ahead)
    };
    let height = if gone { g.run().y().max(0.0) } else { drawn_height(g.run(), ahead) };
    transform.translation = Vec3::new(RIDER_X + (theirs - mine), ROAD_Y + height, 1.9);
    transform.scale.y = if g.run().ducking() {
        super::rules::DUCK_HEIGHT / super::rules::STAND_HEIGHT
    } else {
        1.0
    };
}

/// On a touch device the hint names the touch buttons instead of the keys.
fn touch_hint(seen: Res<TouchSeen>, mut hint: Single<&mut Text, With<HintText>>) {
    if seen.is_changed() && seen.0 {
        hint.0 = "Tap JUMP (hold for height) or DUCK".into();
    }
}

#[allow(clippy::type_complexity)]
fn refresh_hud(
    course: Res<Course>,
    best: Res<Best>,
    pause: Res<MatchPause>,
    active: Option<Res<ActiveMatch>>,
    mut distance: Single<
        &mut Text,
        (With<DistanceText>, Without<PauseLabel>, Without<BannerText>, Without<DayText>),
    >,
    mut pause_label: Single<
        &mut Text,
        (With<PauseLabel>, Without<DistanceText>, Without<BannerText>, Without<DayText>),
    >,
    mut banner_text: Single<
        &mut Text,
        (With<BannerText>, Without<DistanceText>, Without<PauseLabel>, Without<DayText>),
    >,
    mut day_text: Single<
        &mut Text,
        (With<DayText>, Without<DistanceText>, Without<PauseLabel>, Without<BannerText>),
    >,
    mut banner: Single<&mut Visibility, With<Banner>>,
) {
    let metres = course.run.metres();
    let score = course.run.score();
    let mult = course.run.multiplier();
    let sprint = if course.run.sprinting() { "   FRESH HORSE" } else { "" };
    let ghost = course
        .ghost
        .as_ref()
        .map_or(String::new(), |g| format!("   ghost {} m", g.run().metres()));
    distance.0 = format!(
        "{score}   {metres} m  x{mult}   best {}{ghost}{sprint}",
        best.metres.max(metres)
    );
    day_text.0 = format!("Daily road {}", daily::date_label(course.day));
    pause_label.0 = if pause.0 { "Resume" } else { "Pause" }.into();

    let message = match course.run.phase() {
        Phase::Over(cause) => {
            let what = match cause {
                Cause::Block => "CRASHED",
                Cause::Gate => "STRUCK THE BANNER",
                Cause::Pit => "FELL",
            };
            let record = if best.new { "   NEW BEST" } else { "" };
            format!("{what}  {score} pts  {metres} m{record}\nJump or R to ride again")
        }
        Phase::Running => match active.as_ref().map(|m| m.0.phase()) {
            Some(MatchPhase::Countdown(left)) => {
                format!("GET READY  {}", left.div_ceil(TICKS_PER_SECOND))
            }
            _ if pause.0 => "PAUSED".into(),
            _ => String::new(),
        },
    };
    **banner = if message.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    banner_text.0 = message;
}

#[cfg(test)]
mod tests {
    use super::*;
    use xerxes_engine::modules::assets::spawn_scene;
    use xerxes_engine::modules::matches::{Match, Outcome, Rules};
    use xerxes_engine::prelude::{GameMode, Sides, WinCondition};

    /// The scene is the one source of the level: one rider, one road, scenery that scrolls, and
    /// the HUD with its two buttons.
    #[test]
    fn the_scene_has_the_rider_the_road_and_the_hud() {
        let mut world = World::new();
        spawn_scene(
            &mut world,
            &crate::assets::scenes::games::solo::chapar_scene::scene(),
        );
        assert_eq!(world.query::<&Rider>().iter(&world).count(), 1);
        assert_eq!(world.query::<&Road>().iter(&world).count(), 1);
        assert!(world.query::<&Parallax>().iter(&world).count() >= 10);
        assert_eq!(world.query::<&HudButton>().iter(&world).count(), 2);
        assert_eq!(world.query::<&Banner>().iter(&world).count(), 1);
        assert_eq!(world.query::<&DayText>().iter(&world).count(), 1);
        assert_eq!(world.query::<&GhostRider>().iter(&world).count(), 1);
    }

    /// The run's end reaches the engine's Solo x Score match as the rider going down: the round
    /// ends and the one side is ranked first and Finished, with the distance it ran.
    #[test]
    fn a_crash_ends_the_score_match_with_a_ranked_result() {
        let rules = Rules::for_mode(GameMode::grid(Sides::Solo, WinCondition::Score)).unwrap();
        let mut m = Match::new(rules);
        m.start();
        for _ in 0..600 {
            if m.phase() == MatchPhase::Playing {
                break;
            }
            m.tick();
        }
        assert_eq!(m.phase(), MatchPhase::Playing);
        for _ in 0..90 {
            m.tick();
        }
        m.report(Report::Down { side: 0, player: 0 });
        m.tick();
        assert_eq!(m.phase(), MatchPhase::Results);
        let result = m.result().expect("a result");
        assert_eq!(result.standings[0].rank, 1);
        assert_eq!(result.standings[0].outcome, Outcome::Finished);
        assert!(result.standings[0].survived >= 90);
    }
}
