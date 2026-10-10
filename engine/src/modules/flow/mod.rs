//! The game flow every game runs: `Preload → Splash → MainMenu → Level`.
//!
//! - **Preload** downloads and extracts the preload bundles behind a progress bar.
//! - **Splash** (optional) shows a title and/or image for a few seconds; a click or key skips it.
//! - **MainMenu** is an engine bevy_ui menu: Play, an optional level list, Quit (not on the web).
//! - **Level** spawns the current level's scene and sets its [`ActiveGameMode`].
//!
//! Templates and games move the flow with messages; the engine owns the transitions:
//! [`LoadLevel`] (any level), [`LevelComplete`] (next level, or the menu after the last one),
//! [`ExitToMenu`].

use bevy::prelude::*;

use crate::modules::assets::defs::SceneAsset;
use crate::modules::assets::spawn::{SceneRootObject, spawn_scene};
use crate::modules::bundles::{Bundles, LoadBundle, load_image};

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

/// A button (give it `Button` and a `Node`) that starts level `n` when pressed. Menu scenes
/// use it; it works in the main menu and in levels (a "next level" button, say).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct LevelButton(pub usize);

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

/// Installs the flow. The app must have `Bundles` and the bundle messages (`launch_project`
/// sets them up).
pub struct FlowPlugin(pub FlowConfig);

impl Plugin for FlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Flow>()
            .insert_resource(self.0.clone())
            .init_resource::<CurrentLevel>()
            .add_message::<LoadLevel>()
            .add_message::<LevelComplete>()
            .add_message::<ExitToMenu>()
            .add_systems(OnEnter(Flow::Preload), enter_preload)
            .add_systems(OnEnter(Flow::Splash), enter_splash)
            .add_systems(OnEnter(Flow::MainMenu), enter_menu)
            .add_systems(OnEnter(Flow::Level), enter_level)
            .add_systems(OnExit(Flow::Preload), despawn_screen)
            .add_systems(OnExit(Flow::Splash), despawn_screen)
            .add_systems(OnExit(Flow::MainMenu), (despawn_screen, despawn_menu_scene))
            .add_systems(OnExit(Flow::Level), exit_level)
            .add_systems(
                Update,
                (
                    preload.run_if(in_state(Flow::Preload)),
                    splash.run_if(in_state(Flow::Splash)),
                    menu_buttons.run_if(in_state(Flow::MainMenu)),
                    level_buttons,
                    navigate,
                ),
            );
    }
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

fn enter_preload(
    mut commands: Commands,
    config: Res<FlowConfig>,
    mut requests: MessageWriter<LoadBundle>,
) {
    for id in &config.preload_bundles {
        requests.write(LoadBundle(id));
    }
    commands.spawn((FlowScreen, Camera2d));
    commands.spawn(screen()).with_children(|screen| {
        screen.spawn((LoadingText, text("Loading…", 15.0, DIM)));
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
    for mut node in &mut bar {
        node.width = percent(if total == 0 {
            100.0
        } else {
            100.0 * ready as f32 / total as f32
        });
    }
    for mut label in &mut label {
        label.0 = format!("Loading… {ready} / {total}");
    }
    if ready == total {
        next.set(if config.settings.splash.is_some() {
            Flow::Splash
        } else {
            Flow::MainMenu
        });
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
) {
    for (interaction, LevelButton(level)) in &buttons {
        if *interaction == Interaction::Pressed {
            load.write(LoadLevel(*level));
        }
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
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        load.write(LoadLevel(0));
    }
    for (interaction, button) in &buttons {
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
            // `set` re-runs OnExit/OnEnter even when already in Level, so a level restarts.
            next.set(Flow::Level);
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
