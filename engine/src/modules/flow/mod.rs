//! The game flow every game runs: `Preload → Splash → MainMenu → Loading → Level`.
//!
//! - **Preload** downloads and extracts the preload bundles behind a progress bar. Keep this to
//!   what the menu itself needs: everything else arrives per level, which is what makes a web
//!   build progressive (see [`web-progressive.md`](../../../../__docs__/web-progressive.md)).
//! - **Splash** (optional) shows a title and/or image for a few seconds; a click or key skips it.
//! - **MainMenu** is an engine bevy_ui menu: Play, an optional level list, Quit (not on the web).
//! - **Loading** has the level's files in memory before it starts: it reads what the scene's
//!   objects load ([`scene_needs`]), downloads the bundles they ship in and waits for the asset
//!   server to finish with them, behind a progress bar. A level whose files are already there
//!   (a second visit, a prefetch, a scene with no art) passes straight through, so nothing
//!   flashes. This is why a heavy model no longer pops into a level that has already begun.
//! - **Level** spawns the current level's scene and sets its [`ActiveGameMode`].
//!
//! Templates and games move the flow with messages; the engine owns the transitions:
//! [`LoadLevel`] (any level), [`LevelComplete`] (next level, or the menu after the last one),
//! [`ExitToMenu`], and [`PrefetchLevel`] to fetch a level's files **before** it is asked for (the
//! engine's own menu prefetches a level while the pointer rests on its button, and Warp will
//! prefetch where a link leads; the first level is fetched as soon as the menu is up, which is the
//! only head start a touch screen can have, since no pointer rests on a button there). In a scene,
//! [`LevelButton`] and [`ExitToMenuButton`] send the first and the last.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::modules::assets::defs::{AssetNeed, SceneAsset};
use crate::modules::assets::spawn::{SceneRootObject, scene_needs, spawn_scene};
use crate::modules::bundles::{Bundles, LoadBundle, load_image, load_need};

pub mod mode;

pub use self::mode::{GameMode, ModeKind, Modifier, Sides, WinCondition};

/// The splash screen.
#[derive(Debug, Clone, PartialEq)]
pub struct SplashDef {
    pub title: &'static str,
    /// An image from a bundle, shown above the title.
    pub image: Option<&'static str>,
    pub seconds: f32,
}

/// The main menu.
#[derive(Debug, Clone)]
pub struct MainMenuDef {
    pub title: &'static str,
    /// An image from a bundle, behind the menu.
    pub background: Option<&'static str>,
    /// Show a button per level under Play.
    pub level_select: bool,
    /// A scene that is the menu: the engine spawns it instead of drawing its own list and
    /// despawns it when the menu ends. Its buttons ([`LevelButton`]) start levels, so the menu
    /// is made in the editor like any other scene.
    pub scene: Option<fn() -> SceneAsset>,
}

/// One level: a scene and the game mode it runs.
#[derive(Debug, Clone)]
pub struct LevelDef {
    pub name: &'static str,
    pub scene: fn() -> SceneAsset,
    pub mode: GameMode,
}

/// The flow part of the project settings.
#[derive(Debug, Clone)]
pub struct FlowSettings {
    pub splash: Option<SplashDef>,
    pub main_menu: MainMenuDef,
    pub levels: Vec<LevelDef>,
}

impl FlowSettings {
    /// Every level needs a valid game mode, and there must be at least one level.
    pub fn validate(&self) -> Result<(), String> {
        if self.levels.is_empty() {
            return Err("project settings: the flow needs at least one level".into());
        }
        for level in &self.levels {
            level
                .mode
                .validate()
                .map_err(|err| format!("level `{}`: {err}", level.name))?;
        }
        Ok(())
    }
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Flow {
    #[default]
    Preload,
    Splash,
    MainMenu,
    /// Getting the next level's files into memory. Skipped when they are already there.
    Loading,
    Level,
}

/// The level being played (index into the settings' levels).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct CurrentLevel(pub usize);

/// The game mode of the level being played.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct ActiveGameMode(pub GameMode);

/// Start (or restart) level `n`.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct LoadLevel(pub usize);

/// The current level is done: go to the next one, or back to the menu after the last.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct LevelComplete;

/// Leave the level (or splash) for the main menu.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct ExitToMenu;

/// Fetch a level's files now, without going to it: the download happens while the player is
/// still deciding, so [`LoadLevel`] then starts at once. Sending it for a level that is already
/// in memory does nothing. The engine's menu sends it for the button under the pointer.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct PrefetchLevel(pub usize);

/// A button (give it `Button` and a `Node`) that starts level `n` when pressed. Menu scenes
/// use it; it works in the main menu and in levels (a "next level" button, say).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct LevelButton(pub usize);

/// A button (give it `Button` and a `Node`) that leaves the level for the main menu when
/// pressed: a level's Back button, written in the scene like [`LevelButton`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct ExitToMenuButton;

/// Something a logic module spawned for the level outside the scene (UI roots, effects):
/// despawned with the level, like the scene's own objects.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct LevelEntity;

/// The flow settings and the bundles to preload, as a resource.
#[derive(Resource, Debug, Clone)]
pub struct FlowConfig {
    pub settings: FlowSettings,
    pub preload_bundles: Vec<&'static str>,
}

/// The files the flow has asked the asset server for, by the need that asked. Holding the
/// handles keeps the assets alive, so a level that has been loaded once (or prefetched) starts
/// at once the next time instead of downloading again.
#[derive(Resource, Default)]
pub struct Loaded(HashMap<AssetNeed, UntypedHandle>);

impl Loaded {
    /// How many of these needs are in memory, of how many there are.
    pub fn progress(&self, needs: &[AssetNeed], server: &AssetServer) -> (usize, usize) {
        let ready = needs
            .iter()
            .filter(|need| self.is_loaded(need, server))
            .count();
        (ready, needs.len())
    }

    /// Whether this need is in memory.
    pub fn is_loaded(&self, need: &AssetNeed, server: &AssetServer) -> bool {
        self.0
            .get(need)
            .is_some_and(|handle| server.load_state(handle.id()).is_loaded())
    }

    /// Whether every one of these needs is in memory.
    pub fn has_all(&self, needs: &[AssetNeed], server: &AssetServer) -> bool {
        let (ready, total) = self.progress(needs, server);
        ready == total
    }

    /// The needs whose file could not be loaded at all. A level is never held back by one: it
    /// starts without it, as it did before there was a loading step, and says so.
    pub fn failed(&self, needs: &[AssetNeed], server: &AssetServer) -> Vec<AssetNeed> {
        needs
            .iter()
            .filter(|need| {
                self.0
                    .get(need)
                    .is_some_and(|handle| server.load_state(handle.id()).is_failed())
            })
            .copied()
            .collect()
    }
}

/// What a level is waiting for, so the screen can say it: a download is the network, the
/// preparing that follows is the asset server turning what arrived into meshes and textures.
/// A big model is mostly the second, which is the part a progress bar cannot hurry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadStep {
    /// Bundles are still coming, with this many bytes left in the ones that have not arrived.
    Downloading { bytes: u64 },
    /// Everything is here; the asset server is working through it.
    Preparing,
}

/// The level being loaded: what it needs and when it started. Keeping this means a long wait
/// does not read the scene again on every frame of it.
#[derive(Resource, Debug, Clone)]
struct LoadingLevel {
    level: usize,
    needs: Vec<AssetNeed>,
    since: f32,
}

/// The files a level's scene loads. The scene is plain data, so reading it to ask what it needs
/// costs nothing and never depends on the world.
pub fn level_needs(config: &FlowConfig, level: usize) -> Vec<AssetNeed> {
    config
        .settings
        .levels
        .get(level)
        .map(|level| scene_needs(&(level.scene)()))
        .unwrap_or_default()
}

/// Asks for every file these needs name: the bundle it ships in, and then the file itself once
/// the bundle is extracted. Returns how many are in memory, of how many, and what the wait is on.
fn fetch(world: &mut World, needs: &[AssetNeed]) -> (usize, usize, LoadStep) {
    let mut wanted = Vec::new();
    let mut missing: Vec<&'static str> = Vec::new();
    let mut missing_bytes = 0;
    for need in needs {
        let bundles = world.resource::<Bundles>();
        let bundle = bundles.bundle_of(need.path);
        if bundles.is_ready(bundle) {
            wanted.push(*need);
        } else {
            if !missing.contains(&bundle) {
                missing.push(bundle);
                missing_bytes += bundles.info(bundle).map_or(0, |info| info.size);
            }
            world.write_message(LoadBundle(bundle));
        }
    }
    for need in wanted {
        if world.resource::<Loaded>().0.contains_key(&need) {
            continue;
        }
        let handle = load_need(world, &need);
        world.resource_mut::<Loaded>().0.insert(need, handle);
    }
    let server = world.resource::<AssetServer>();
    let (ready, total) = world.resource::<Loaded>().progress(needs, server);
    let step = if missing.is_empty() {
        LoadStep::Preparing
    } else {
        LoadStep::Downloading {
            bytes: missing_bytes,
        }
    };
    (ready, total, step)
}

/// What the loading screen says: a download names the megabytes still to come, the preparing
/// that follows counts the files the asset server has finished with.
fn loading_label(name: &str, ready: usize, total: usize, step: LoadStep) -> String {
    match step {
        LoadStep::Downloading { bytes } if bytes > 0 => {
            format!("Downloading {name}...  {:.1} MB", bytes as f64 / 1e6)
        }
        LoadStep::Downloading { .. } => format!("Downloading {name}..."),
        LoadStep::Preparing => format!("Preparing {name}...  {ready} / {total}"),
    }
}

/// The level's name, for the screen and the log.
fn level_name(world: &World, index: usize) -> String {
    world
        .resource::<FlowConfig>()
        .settings
        .levels
        .get(index)
        .map(|level| level.name.to_string())
        .unwrap_or_else(|| "level".into())
}

/// Installs the flow. The app must have `Bundles` and the bundle messages (`launch_project`
/// sets them up).
pub struct FlowPlugin(pub FlowConfig);

impl Plugin for FlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Flow>()
            .insert_resource(self.0.clone())
            .init_resource::<CurrentLevel>()
            .init_resource::<Loaded>()
            .init_resource::<LevelReady>()
            .add_message::<LoadLevel>()
            .add_message::<LevelComplete>()
            .add_message::<ExitToMenu>()
            .add_message::<PrefetchLevel>()
            .add_systems(OnEnter(Flow::Preload), enter_preload)
            .add_systems(OnEnter(Flow::Splash), enter_splash)
            .add_systems(OnEnter(Flow::MainMenu), (enter_menu, prefetch_first_level))
            .add_systems(OnEnter(Flow::Loading), enter_loading)
            // `Update` and `FixedUpdate` both: a game steps its rules in whichever of the two
            // suits it (Siavash takes its own ticks in `Update`, Chapar and Qanat run on the
            // fixed clock), and the beat means the same thing in both.
            .configure_sets(
                Update,
                Play.run_if(in_state(Flow::Level).and(level_is_live)),
            )
            .configure_sets(
                FixedUpdate,
                Play.run_if(in_state(Flow::Level).and(level_is_live)),
            )
            .add_systems(OnEnter(Flow::Level), (enter_level, start_ready))
            .add_systems(OnExit(Flow::Preload), despawn_screen)
            .add_systems(OnExit(Flow::Splash), despawn_screen)
            .add_systems(OnExit(Flow::MainMenu), (despawn_screen, despawn_menu_scene))
            .add_systems(OnExit(Flow::Loading), (despawn_screen, forget_loading))
            .add_systems(OnExit(Flow::Level), exit_level)
            .add_systems(
                Update,
                (
                    preload.run_if(in_state(Flow::Preload)),
                    splash.run_if(in_state(Flow::Splash)),
                    menu_buttons.run_if(in_state(Flow::MainMenu)),
                    loading.run_if(in_state(Flow::Loading)),
                    spend_ready.run_if(in_state(Flow::Level)),
                    // Scene buttons write their message before `navigate` reads it, so a press
                    // moves the flow in the same frame.
                    (level_buttons, exit_buttons).before(navigate),
                    navigate,
                    // A prefetch is asked for after the buttons, and never changes the flow.
                    prefetch.after(navigate),
                ),
            );
    }
}

/// The beat between a level appearing on screen and its world starting to move.
///
/// A level opens with the player looking at it for the first time: where they stand, what is
/// coming, which way is forward. A game that starts the instant the scene spawns takes that away,
/// and in a game with no second chance — a runner meeting its first obstacle — it takes the run
/// with it. So every level is given a beat, and the game asks for it with [`level_is_live`] or
/// [`LevelReady::is_live`] before it steps anything.
///
/// The beat is counted in **frames the player could watch**, not in wall-clock seconds: a level's
/// first frames are the slowest it will ever have (models going into the world, pipelines being
/// built), and one two-second frame must not swallow the whole beat. At most
/// [`LevelReady::MOST_PER_FRAME`] of it is spent per frame, so the beat is always both a stretch
/// of time *and* a number of frames.
///
/// `beat` is the game's to set (`app.insert_resource(LevelReady::of(1.0))`, or 0 for a level that
/// should simply start); [`restart`](Self::restart) gives the beat again when a game restarts a
/// run inside the level the player has just been reading a banner on.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct LevelReady {
    left: f32,
    /// How long the beat is, in seconds of frames the player saw.
    pub beat: f32,
}

impl Default for LevelReady {
    fn default() -> Self {
        // Three seconds: played on a phone, two was over before the lane had been read.
        Self::of(3.0)
    }
}

impl LevelReady {
    /// The most of the beat one frame may spend, however long that frame took.
    pub const MOST_PER_FRAME: f32 = 0.1;

    pub const fn of(beat: f32) -> Self {
        Self { left: beat, beat }
    }

    /// Whether the level is the player's to play: false while the beat is still owed.
    pub fn is_live(&self) -> bool {
        self.left <= 0.0
    }

    /// What is left of the beat, for a game that counts it down on screen.
    pub fn left(&self) -> f32 {
        self.left.max(0.0)
    }

    /// Give the beat again (a retry inside the same level).
    pub fn restart(&mut self) {
        self.left = self.beat;
    }

    fn spend(&mut self, delta: f32) {
        if self.left > 0.0 {
            self.left -= delta.min(Self::MOST_PER_FRAME);
        }
    }
}

/// Run condition: the level is up, the beat is over, the game is the player's.
pub fn level_is_live(ready: Res<LevelReady>) -> bool {
    ready.is_live()
}

/// **Where a game's play systems go.** The engine runs this set while a level is on and the
/// [`LevelReady`] beat is over, so every game gets the beat by writing
///
/// ```ignore
/// app.add_systems(Update, (steer, step, collide).in_set(Play));
/// ```
///
/// instead of its own `run_if(in_state(Flow::Level))`. What must keep running *during* the beat —
/// building the level's world, the HUD, the countdown, a restart button — stays out of the set
/// and gates itself; that is the whole distinction the set draws: advancing the game, or showing
/// it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Play;

/// A level starts owing the player the beat.
fn start_ready(mut ready: ResMut<LevelReady>) {
    ready.restart();
}

/// One frame of the beat, never more than [`LevelReady::MOST_PER_FRAME`] of it.
fn spend_ready(time: Res<Time>, mut ready: ResMut<LevelReady>) {
    ready.spend(time.delta_secs());
}

/// Everything a flow screen (preload, splash, menu) spawns; despawned when it ends.
#[derive(Component)]
struct FlowScreen;

#[derive(Component)]
struct LoadingBar;

#[derive(Component)]
struct LoadingText;

#[derive(Resource)]
struct SplashTimer(Timer);

#[derive(Component, Clone, Copy)]
enum MenuButton {
    Play,
    Level(usize),
    Quit,
}

const BACKDROP: Color = Color::srgb(0.043, 0.051, 0.071);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const DIM: Color = Color::srgb(0.6, 0.63, 0.68);
const ACCENT: Color = Color::srgb(0.95, 0.45, 0.12);

fn screen() -> (FlowScreen, Node, BackgroundColor) {
    (
        FlowScreen,
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(14),
            ..default()
        },
        BackgroundColor(BACKDROP),
    )
}

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

fn despawn_screen(
    mut commands: Commands,
    screens: Query<Entity, (With<FlowScreen>, Without<ChildOf>)>,
) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}

/// The loading screen: a line of text over a bar. Preload and Loading show the same one, so a
/// game that has been waiting since the first frame never changes the look of waiting.
fn spawn_loading_screen(commands: &mut Commands, label: &str) {
    commands.spawn((FlowScreen, Camera2d));
    let label = label.to_string();
    commands.spawn(screen()).with_children(|screen| {
        screen.spawn((LoadingText, text(label, 15.0, DIM)));
        screen
            .spawn((
                Node {
                    width: px(160),
                    height: px(3),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.14, 0.19)),
            ))
            .with_child((
                LoadingBar,
                Node {
                    width: percent(0),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(ACCENT),
            ));
    });
}

/// Fills the bar and writes the count. `(ready, total)`; a step with nothing to do is full.
fn show_progress(
    ready: usize,
    total: usize,
    label: &str,
    bar: &mut Query<&mut Node, With<LoadingBar>>,
    text: &mut Query<&mut Text, With<LoadingText>>,
) {
    for mut node in bar.iter_mut() {
        node.width = percent(if total == 0 {
            100.0
        } else {
            100.0 * ready as f32 / total as f32
        });
    }
    for mut line in text.iter_mut() {
        line.0 = format!("{label} {ready} / {total}");
    }
}

fn enter_preload(
    mut commands: Commands,
    config: Res<FlowConfig>,
    mut requests: MessageWriter<LoadBundle>,
) {
    for id in &config.preload_bundles {
        requests.write(LoadBundle(id));
    }
    spawn_loading_screen(&mut commands, "Loading...");
}

fn preload(
    config: Res<FlowConfig>,
    bundles: Res<Bundles>,
    mut bar: Query<&mut Node, With<LoadingBar>>,
    mut label: Query<&mut Text, With<LoadingText>>,
    mut next: ResMut<NextState<Flow>>,
) {
    let total = config.preload_bundles.len();
    let ready = config
        .preload_bundles
        .iter()
        .filter(|id| bundles.is_ready(id))
        .count();
    show_progress(ready, total, "Loading...", &mut bar, &mut label);
    if ready == total {
        next.set(if config.settings.splash.is_some() {
            Flow::Splash
        } else {
            Flow::MainMenu
        });
    }
}

/// Asks for the level's files. When they are all in memory already the level starts at once and
/// no screen is shown, so the common case (a second visit, a prefetch, a scene with no art) does
/// not flash.
fn enter_loading(world: &mut World) {
    let index = world.resource::<CurrentLevel>().0;
    let config = world.resource::<FlowConfig>().clone();
    let needs = level_needs(&config, index);
    let (ready, total, step) = fetch(world, &needs);
    if ready == total {
        world.resource_mut::<NextState<Flow>>().set(Flow::Level);
        return;
    }
    let name = level_name(world, index);
    info!("{name}: waiting for {} of {total} file(s)", total - ready);
    let label = loading_label(&name, ready, total, step);
    let since = world.resource::<Time>().elapsed_secs();
    // The needs are kept: the frames of a long wait do not read the scene again.
    world.insert_resource(LoadingLevel {
        level: index,
        needs,
        since,
    });
    let mut commands = world.commands();
    spawn_loading_screen(&mut commands, &label);
    world.flush();
}

/// Leaving the Loading step (for the level, or back to the menu) ends this level's wait. What
/// was already fetched stays in [`Loaded`], so coming back is instant.
fn forget_loading(mut commands: Commands) {
    commands.remove_resource::<LoadingLevel>();
}

/// Keeps asking (a bundle arrives, then the files in it can load) until the level can start.
fn loading(world: &mut World) {
    let index = world.resource::<CurrentLevel>().0;
    // Another level was asked for while this one was loading: start over on the new one.
    let stale = world
        .get_resource::<LoadingLevel>()
        .is_none_or(|loading| loading.level != index);
    if stale {
        let config = world.resource::<FlowConfig>().clone();
        let needs = level_needs(&config, index);
        let since = world.resource::<Time>().elapsed_secs();
        world.insert_resource(LoadingLevel {
            level: index,
            needs,
            since,
        });
    }
    let Some(loading) = world.get_resource::<LoadingLevel>().cloned() else {
        return;
    };
    let (ready, total, step) = fetch(world, &loading.needs);
    let name = level_name(world, index);
    let label = loading_label(&name, ready, total, step);
    let mut bars = world.query_filtered::<&mut Node, With<LoadingBar>>();
    for mut node in bars.query_mut(world).iter_mut() {
        node.width = percent(if total == 0 {
            100.0
        } else {
            100.0 * ready as f32 / total as f32
        });
    }
    let mut lines = world.query_filtered::<&mut Text, With<LoadingText>>();
    for mut line in lines.query_mut(world).iter_mut() {
        if line.0 != label {
            line.0 = label.clone();
        }
    }
    // A file that cannot be loaded would hold the level for ever: say which, and start anyway.
    let server = world.resource::<AssetServer>().clone();
    let broken = world.resource::<Loaded>().failed(&loading.needs, &server);
    if ready != total && !(ready + broken.len() == total && !broken.is_empty()) {
        return;
    }
    for need in &broken {
        error!(
            "{name}: {} could not be loaded; starting without it",
            need.path
        );
    }
    // How long the wait was, so a level that takes too long says so without a profiler.
    let waited = world.resource::<Time>().elapsed_secs() - loading.since;
    info!("{name}: {total} file(s) ready in {waited:.1} s");
    world.resource_mut::<NextState<Flow>>().set(Flow::Level);
}

/// When the menu opens, fetch the **first** level in the background: it is the one Play and Enter
/// start, and it is the only guess the engine can make that is bounded (one level, not all of
/// them). It matters most on a touch screen, where no pointer ever rests on a button, so the
/// hover prefetch never fires and the player would pay the whole download on the tap.
///
/// The menu is already up and interactive while this runs, so it costs nothing to look at —
/// unlike naming the same bundle in `preload_bundles`, which holds the boot behind it. A game
/// whose first level is heavy and rarely played should simply not have it first.
fn prefetch_first_level(config: Res<FlowConfig>, mut prefetch: MessageWriter<PrefetchLevel>) {
    if !config.settings.levels.is_empty() {
        prefetch.write(PrefetchLevel(0));
    }
}

/// [`PrefetchLevel`]: the level's files are asked for, and nothing else happens.
fn prefetch(world: &mut World) {
    let mut wanted: Vec<usize> = Vec::new();
    {
        let mut messages = world.resource_mut::<Messages<PrefetchLevel>>();
        let mut cursor = messages.get_cursor();
        for PrefetchLevel(level) in cursor.read(&messages) {
            if !wanted.contains(level) {
                wanted.push(*level);
            }
        }
        messages.clear();
    }
    for level in wanted {
        let config = world.resource::<FlowConfig>().clone();
        let needs = level_needs(&config, level);
        if needs.is_empty() {
            continue;
        }
        fetch(world, &needs);
    }
}

fn enter_splash(world: &mut World) {
    let Some(splash) = world.resource::<FlowConfig>().settings.splash.clone() else {
        return;
    };
    world.insert_resource(SplashTimer(Timer::from_seconds(
        splash.seconds.max(0.0),
        TimerMode::Once,
    )));
    let image = splash.image.map(|path| load_image(world, path));
    world.spawn((FlowScreen, Camera2d));
    world.spawn(screen()).with_children(|screen| {
        if let Some(image) = image {
            screen.spawn((
                ImageNode::new(image),
                Node {
                    width: px(160),
                    height: px(160),
                    ..default()
                },
            ));
        }
        screen.spawn(text(splash.title, 34.0, TEXT));
    });
}

fn splash(
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut next: ResMut<NextState<Flow>>,
) {
    timer.0.tick(time.delta());
    let skipped = timer.0.elapsed_secs() >= 0.5
        && (keys.get_just_pressed().next().is_some()
            || mouse.get_just_pressed().next().is_some()
            || touches.any_just_pressed());
    if timer.0.is_finished() || skipped {
        next.set(Flow::MainMenu);
    }
}

fn enter_menu(world: &mut World) {
    let config = world.resource::<FlowConfig>().settings.clone();
    if let Some(scene) = config.main_menu.scene {
        spawn_scene(world, &scene());
        return;
    }
    let background = config
        .main_menu
        .background
        .map(|path| load_image(world, path));
    world.spawn((FlowScreen, Camera2d));
    let mut root = world.spawn(screen());
    if let Some(image) = background {
        root.insert(ImageNode::new(image));
    }
    root.with_children(|menu| {
        menu.spawn(text(config.main_menu.title, 34.0, TEXT));
        menu.spawn(menu_button(MenuButton::Play, "Play", true));
        if config.main_menu.level_select {
            for (i, level) in config.levels.iter().enumerate() {
                menu.spawn(menu_button(MenuButton::Level(i), level.name, false));
            }
        }
        // A web page has nowhere to quit to.
        if !cfg!(target_arch = "wasm32") {
            menu.spawn(menu_button(MenuButton::Quit, "Quit", false));
        }
    });
}

fn despawn_menu_scene(mut commands: Commands, roots: Query<Entity, With<SceneRootObject>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

fn level_buttons(
    buttons: Query<(&Interaction, &LevelButton), Changed<Interaction>>,
    mut load: MessageWriter<LoadLevel>,
    mut prefetch: MessageWriter<PrefetchLevel>,
) {
    for (interaction, LevelButton(level)) in &buttons {
        match interaction {
            Interaction::Pressed => {
                load.write(LoadLevel(*level));
            }
            // The pointer resting on a button is the first hint of where the player is going:
            // fetch that level's files now, so pressing it starts at once.
            Interaction::Hovered => {
                prefetch.write(PrefetchLevel(*level));
            }
            Interaction::None => {}
        }
    }
}

fn exit_buttons(
    buttons: Query<&Interaction, (Changed<Interaction>, With<ExitToMenuButton>)>,
    mut exit: MessageWriter<ExitToMenu>,
) {
    if buttons.iter().any(|i| *i == Interaction::Pressed) {
        exit.write(ExitToMenu);
    }
}

fn menu_button(action: MenuButton, label: &str, primary: bool) -> impl Bundle {
    (
        Button,
        action,
        Node {
            width: px(220),
            padding: UiRect::axes(px(16), px(10)),
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(if primary {
            ACCENT
        } else {
            Color::srgb(0.165, 0.184, 0.216)
        }),
        children![text(
            label.to_string(),
            16.0,
            if primary {
                Color::srgb(0.07, 0.07, 0.07)
            } else {
                TEXT
            }
        )],
    )
}

fn menu_buttons(
    buttons: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut load: MessageWriter<LoadLevel>,
    mut prefetch: MessageWriter<PrefetchLevel>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        load.write(LoadLevel(0));
    }
    for (interaction, button) in &buttons {
        // Hovering a level's button fetches it, as in a scene menu's `LevelButton`.
        if *interaction == Interaction::Hovered {
            match button {
                MenuButton::Play => prefetch.write(PrefetchLevel(0)),
                MenuButton::Level(i) => prefetch.write(PrefetchLevel(*i)),
                MenuButton::Quit => continue,
            };
            continue;
        }
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button {
            MenuButton::Play => {
                load.write(LoadLevel(0));
            }
            MenuButton::Level(i) => {
                load.write(LoadLevel(*i));
            }
            MenuButton::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}

/// Applies the flow messages. A level that does not exist is reported and ignored.
fn navigate(
    mut load: MessageReader<LoadLevel>,
    mut complete: MessageReader<LevelComplete>,
    mut menu: MessageReader<ExitToMenu>,
    config: Res<FlowConfig>,
    mut current: ResMut<CurrentLevel>,
    state: Res<State<Flow>>,
    mut next: ResMut<NextState<Flow>>,
) {
    let levels = config.settings.levels.len();
    let mut target = None;
    if complete.read().count() > 0 && *state.get() == Flow::Level {
        target = Some(if current.0 + 1 < levels {
            Some(current.0 + 1)
        } else {
            None
        });
    }
    if let Some(LoadLevel(n)) = load.read().last() {
        if *n < levels {
            target = Some(Some(*n));
        } else {
            error!("LoadLevel({n}): the project has {levels} level(s)");
        }
    }
    if menu.read().count() > 0 {
        target = Some(None);
    }
    match target {
        Some(Some(level)) => {
            current.0 = level;
            // Always through Loading: it is the one place that knows whether the level's files
            // are in memory, and it passes straight through when they are. `set` re-runs
            // OnExit/OnEnter even when already in Level, so a level restarts.
            next.set(Flow::Loading);
        }
        Some(None) => next.set(Flow::MainMenu),
        None => {}
    }
}

fn enter_level(world: &mut World) {
    let index = world.resource::<CurrentLevel>().0;
    let level = world.resource::<FlowConfig>().settings.levels[index].clone();
    info!(
        "level {}: {} ({})",
        index + 1,
        level.name,
        level.mode.cell_label()
    );
    world.insert_resource(ActiveGameMode(level.mode));
    spawn_scene(world, &(level.scene)());
}

fn exit_level(
    mut commands: Commands,
    roots: Query<Entity, Or<(With<SceneRootObject>, (With<LevelEntity>, Without<ChildOf>))>>,
) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<ActiveGameMode>();
}
