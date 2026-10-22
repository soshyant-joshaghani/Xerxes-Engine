//! The game flow, headless: Preload → Splash → MainMenu → Loading → Level, level changes despawn
//! the previous level, the last LevelComplete returns to the menu; game modes validate against
//! the taxonomy.
//!
//! Every level goes through `Flow::Loading`, which is where the engine makes sure the files the
//! scene draws are in memory. A level with nothing to load passes straight through it (one frame),
//! which is why the counts below are what they are; a level that needs art waits there, and the
//! scene is not spawned until it can be whole.

use std::collections::HashMap;
use std::time::Duration;

use bevy::asset::io::memory::Dir;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use xerxes_engine::modules::flow::{FlowConfig, FlowPlugin};
use xerxes_engine::prelude::*;

fn level_one() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D3,
        objects: vec![Object::new("Level One Thing")],
    }
}

fn level_two() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D3,
        objects: vec![Object::new("Level Two Thing")],
    }
}

fn settings(splash: bool) -> FlowSettings {
    FlowSettings {
        splash: splash.then_some(SplashDef {
            title: "Studio",
            image: None,
            seconds: 1.0,
        }),
        main_menu: MainMenuDef {
            title: "Game",
            background: None,
            level_select: true,
            scene: None,
        },
        levels: vec![
            LevelDef {
                name: "One",
                scene: level_one,
                mode: GameMode::grid(Sides::Solo, WinCondition::Objective)
                    .with(Modifier::TimeLimit { seconds: 120.0 }),
            },
            LevelDef {
                name: "Two",
                scene: level_two,
                mode: GameMode::sandbox(),
            },
        ],
    }
}

fn app(splash: bool) -> App {
    app_with(settings(splash))
}

fn app_with(settings: FlowSettings) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        InputPlugin,
        AssetPlugin::default(),
    ))
    .init_asset::<Image>()
    .add_message::<LoadBundle>()
    .add_message::<BundleReady>()
    .insert_resource(Bundles::new(Dir::default(), &[], HashMap::new()))
    // Every frame advances 0.4 s, so the 1 s splash takes a few frames.
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        400,
    )))
    .add_plugins(FlowPlugin(FlowConfig {
        settings,
        preload_bundles: Vec::new(),
    }));
    app
}

fn run(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

fn state(app: &App) -> Flow {
    *app.world().resource::<State<Flow>>().get()
}

fn has(app: &mut App, name: &str) -> bool {
    let world = app.world_mut();
    world
        .query::<&Name>()
        .iter(world)
        .any(|n| n.as_str() == name)
}

#[test]
fn preload_then_splash_then_menu() {
    let mut app = app(true);
    app.update();
    assert_eq!(state(&app), Flow::Preload);
    run(&mut app, 2);
    assert_eq!(state(&app), Flow::Splash);
    run(&mut app, 1);
    assert_eq!(
        state(&app),
        Flow::Splash,
        "the splash stays for its seconds"
    );
    run(&mut app, 4);
    assert_eq!(state(&app), Flow::MainMenu);

    // Without a splash, preload goes straight to the menu.
    let mut app = app_without_splash();
    run(&mut app, 3);
    assert_eq!(state(&app), Flow::MainMenu);
}

fn app_without_splash() -> App {
    app(false)
}

#[test]
fn levels_load_complete_and_return_to_the_menu() {
    let mut app = app(false);
    run(&mut app, 3);
    assert_eq!(state(&app), Flow::MainMenu);

    app.world_mut().write_message(LoadLevel(0));
    // Frame 1: the message is read and the flow goes to Loading. Frame 2: Loading sees there is
    // nothing to fetch and asks for Level. Frame 3: the level is spawned.
    run(&mut app, 3);
    assert_eq!(state(&app), Flow::Level);
    assert!(has(&mut app, "Level One Thing"));
    let mode = app.world().resource::<ActiveGameMode>().0.clone();
    assert_eq!(mode.cell_label(), "Challenge clear");

    // Something a logic module spawned for the level goes with it.
    app.world_mut().spawn((LevelEntity, Name::new("HUD root")));

    app.world_mut().write_message(LevelComplete);
    run(&mut app, 3);
    assert_eq!(state(&app), Flow::Level);
    assert_eq!(app.world().resource::<CurrentLevel>().0, 1);
    assert!(
        !has(&mut app, "Level One Thing"),
        "the previous level was despawned"
    );
    assert!(!has(&mut app, "HUD root"));
    assert!(has(&mut app, "Level Two Thing"));

    // The last level's completion returns to the menu and clears the level.
    app.world_mut().write_message(LevelComplete);
    run(&mut app, 2);
    assert_eq!(state(&app), Flow::MainMenu);
    assert!(!has(&mut app, "Level Two Thing"));
    assert!(app.world().get_resource::<ActiveGameMode>().is_none());

    // A level that does not exist is ignored.
    app.world_mut().write_message(LoadLevel(9));
    run(&mut app, 2);
    assert_eq!(state(&app), Flow::MainMenu);
}

fn menu_scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Menu Title"),
            Object::new("Play Button").with(Button).with(LevelButton(1)),
        ],
    }
}

#[test]
fn a_scene_can_be_the_main_menu() {
    let mut flow = settings(false);
    flow.main_menu.scene = Some(menu_scene);
    let mut app = app_with(flow);
    run(&mut app, 3);
    assert_eq!(state(&app), Flow::MainMenu);
    assert!(has(&mut app, "Menu Title"), "the menu scene is spawned");

    // Pressing its button starts that level, and the menu goes away.
    let world = app.world_mut();
    let mut buttons = world.query::<(Entity, &LevelButton)>();
    let button = buttons.iter(world).next().unwrap().0;
    world.entity_mut(button).insert(Interaction::Pressed);
    run(&mut app, 4);
    assert_eq!(state(&app), Flow::Level);
    assert_eq!(app.world().resource::<CurrentLevel>().0, 1);
    assert!(!has(&mut app, "Menu Title"), "the menu scene was despawned");
    assert!(has(&mut app, "Level Two Thing"));

    // Back at the menu it is spawned again.
    app.world_mut().write_message(ExitToMenu);
    run(&mut app, 3);
    assert!(has(&mut app, "Menu Title"));
}

#[test]
fn a_back_button_in_a_level_returns_to_the_menu() {
    let mut app = app(false);
    run(&mut app, 3);
    app.world_mut().write_message(LoadLevel(1));
    run(&mut app, 3);
    app.world_mut()
        .spawn((Button, ExitToMenuButton, Interaction::Pressed));
    run(&mut app, 2);
    assert_eq!(state(&app), Flow::MainMenu);
    assert!(!has(&mut app, "Level Two Thing"));
}

#[test]
fn exit_to_menu_leaves_the_level() {
    let mut app = app(false);
    run(&mut app, 3);
    app.world_mut().write_message(LoadLevel(1));
    run(&mut app, 3);
    assert!(has(&mut app, "Level Two Thing"));
    app.world_mut().write_message(ExitToMenu);
    run(&mut app, 2);
    assert_eq!(state(&app), Flow::MainMenu);
    assert!(!has(&mut app, "Level Two Thing"));
}

#[test]
fn modes_follow_the_taxonomy() {
    assert!(
        GameMode::grid(Sides::Solo, WinCondition::Role)
            .validate()
            .is_err(),
        "Solo × Role is empty"
    );
    assert!(
        GameMode::grid(Sides::NGon(2), WinCondition::Score)
            .validate()
            .is_err()
    );
    assert!(
        GameMode::grid(Sides::NGon(4), WinCondition::Survival)
            .validate()
            .is_ok()
    );
    assert!(
        GameMode::new(ModeKind::Tournament(Box::new(ModeKind::Sandbox)))
            .validate()
            .is_err()
    );
    assert!(
        GameMode::new(ModeKind::Composite(vec![]))
            .validate()
            .is_err()
    );
    assert!(
        GameMode::sandbox()
            .with(Modifier::TimeLimit { seconds: 0.0 })
            .validate()
            .is_err()
    );
    assert!(
        GameMode::grid(Sides::Solo, WinCondition::Score)
            .asymmetric()
            .validate()
            .is_err()
    );
    assert!(
        GameMode::grid(Sides::Duel, WinCondition::Role)
            .asymmetric()
            .validate()
            .is_ok()
    );

    assert_eq!(
        GameMode::grid(Sides::Duel, WinCondition::Survival).cell_label(),
        "Round elimination"
    );
    assert_eq!(
        GameMode::grid(Sides::NGon(3), WinCondition::Role).cell_label(),
        "Hidden role / social deduction"
    );
    let open_world = GameMode::new(ModeKind::Composite(vec![
        ModeKind::Grid {
            sides: Sides::Solo,
            win: WinCondition::Objective,
        },
        ModeKind::Sandbox,
        ModeKind::Progression,
    ]));
    assert_eq!(
        open_world.cell_label(),
        "Challenge clear + Sandbox + Progression"
    );
    assert!(!open_world.needs_multiplayer());
    assert!(GameMode::grid(Sides::Coop, WinCondition::Objective).needs_multiplayer());
}

#[test]
fn flow_settings_name_the_bad_level() {
    let mut flow = settings(false);
    flow.levels[1].mode = GameMode::grid(Sides::Solo, WinCondition::Role);
    let err = flow.validate().unwrap_err();
    assert!(err.contains("level `Two`"), "{err}");
    flow.levels.clear();
    assert!(flow.validate().is_err());
}

/// A scene that draws a file (an image, a model) does not start until that file is in memory:
/// the flow waits in Loading, behind its screen, and asks for the bundle the file ships in.
/// This is what keeps a heavy model from popping into a level that has already begun.
#[test]
fn a_level_waits_for_the_files_its_scene_draws() {
    fn heavy() -> SceneAsset {
        SceneAsset {
            dimension: Dimension::D3,
            objects: vec![
                Object::new("Knight").with(ModelDef::new("meshes/chr/knight.glb")),
                Object::new("Banner").with(SpriteDef {
                    image: Some("art/banner.png"),
                    color: Color::WHITE,
                    size: None,
                }),
            ],
        }
    }

    // What the scene asks for, read off its objects: no list to keep by hand.
    let needs = scene_needs(&heavy());
    assert_eq!(needs.len(), 2);
    assert!(needs.contains(&AssetNeed::model("meshes/chr/knight.glb", 0)));
    assert!(needs.contains(&AssetNeed::image("art/banner.png")));

    let mut flow = settings(false);
    flow.levels[0].scene = heavy;
    let mut app = app_with(flow);
    run(&mut app, 3);
    assert_eq!(state(&app), Flow::MainMenu);

    app.world_mut().write_message(LoadLevel(0));
    run(&mut app, 6);
    assert_eq!(
        state(&app),
        Flow::Loading,
        "the level waits for its files (this app has no bundles to give it)"
    );
    assert!(
        !has(&mut app, "Knight"),
        "nothing of the level is spawned while it waits"
    );
    // The bundle the files ship in was asked for, and the loading screen is up.
    let world = app.world_mut();
    let asked = world
        .resource::<Messages<LoadBundle>>()
        .iter_current_update_messages()
        .count();
    assert!(asked > 0, "the flow asked for the bundle");
    // The screen says which level, and which step it is on. This app has no bundles to give, so
    // it never leaves the downloading step.
    let screen: Vec<String> = world
        .query::<&Text>()
        .iter(world)
        .map(|text| text.0.clone())
        .collect();
    assert!(
        screen
            .iter()
            .any(|line| line.starts_with("Downloading One")),
        "the loading screen names the level and the step: {screen:?}"
    );

    // Leaving is still possible: a level that cannot load must not trap the player.
    app.world_mut().write_message(ExitToMenu);
    run(&mut app, 3);
    assert_eq!(state(&app), Flow::MainMenu);
    assert!(!has(&mut app, "Knight"));
}

/// A prefetch asks for a level's files without going to it, so pressing the button later starts
/// at once. The engine sends one for the level button under the pointer, which is the first hint
/// of where the player is going.
#[test]
fn a_prefetch_fetches_a_level_without_leaving_the_menu() {
    fn arty() -> SceneAsset {
        SceneAsset {
            dimension: Dimension::D2,
            objects: vec![Object::new("Backdrop").with(SpriteDef {
                image: Some("art/backdrop.png"),
                color: Color::WHITE,
                size: None,
            })],
        }
    }

    fn asked_for_a_bundle(app: &mut App) -> bool {
        app.world()
            .resource::<Messages<LoadBundle>>()
            .iter_current_update_messages()
            .next()
            .is_some()
    }

    let mut flow = settings(false);
    flow.levels[0].scene = arty;
    flow.levels[1].scene = arty;
    flow.main_menu.scene = Some(menu_scene);
    let mut app = app_with(flow);
    run(&mut app, 2);
    assert_eq!(state(&app), Flow::MainMenu);
    // The menu is up: the first level (what Play and Enter start) is fetched in the background.
    // On a touch screen this is the only head start there is, since no pointer rests on a button.
    assert!(
        asked_for_a_bundle(&mut app),
        "opening the menu fetches the first level"
    );
    run(&mut app, 1);

    app.world_mut().write_message(PrefetchLevel(1));
    run(&mut app, 1);
    assert_eq!(state(&app), Flow::MainMenu, "a prefetch moves nothing");
    assert!(
        asked_for_a_bundle(&mut app),
        "the level's bundle was asked for"
    );

    // The pointer resting on the menu's `LevelButton(1)` does the same thing.
    let world = app.world_mut();
    let mut buttons = world.query::<(Entity, &LevelButton)>();
    let button = buttons
        .iter(world)
        .find(|(_, LevelButton(level))| *level == 1)
        .expect("the menu scene has a level button")
        .0;
    world.entity_mut(button).insert(Interaction::Hovered);
    run(&mut app, 1);
    assert_eq!(state(&app), Flow::MainMenu, "hovering starts nothing");
    assert!(
        asked_for_a_bundle(&mut app),
        "hovering a level button fetches that level"
    );
}

/// **The ready beat.** Reported from a phone: Siavash was already running before the player could
/// take in the lane, and the first goon took the run. Every level now opens with a beat — the
/// engine holds the `Play` set back — and the beat is counted in frames the player could watch,
/// because a level's first frames are its slowest and one of them must not swallow it.
#[test]
fn a_level_gives_the_player_a_beat_before_play_starts() {
    #[derive(Resource, Default)]
    struct Steps(u32);

    let mut app = app_without_splash();
    app.init_resource::<Steps>()
        .insert_resource(LevelReady::of(1.0))
        .add_systems(
            Update,
            (|mut steps: ResMut<Steps>| steps.0 += 1).in_set(Play),
        );

    app.world_mut().write_message(LoadLevel(0));
    run(&mut app, 3);
    assert_eq!(state(&app), Flow::Level);
    assert_eq!(
        app.world().resource::<Steps>().0,
        0,
        "the level is up, and the player has not been given their beat yet"
    );

    // One very slow frame — a model going into the world — may spend only its share of the beat.
    let before = app.world().resource::<LevelReady>().left();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(2)));
    app.update();
    let spent = before - app.world().resource::<LevelReady>().left();
    assert!(
        spent <= LevelReady::MOST_PER_FRAME + 1e-4,
        "a two-second frame spent {spent} of the beat"
    );
    assert_eq!(app.world().resource::<Steps>().0, 0);

    // Frames the player can watch: the beat is spent, and then the game is theirs.
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )));
    run(&mut app, (1.0_f32 / 0.016).ceil() as usize + 2);
    assert!(app.world().resource::<LevelReady>().is_live());
    assert!(app.world().resource::<Steps>().0 > 0, "and then play runs");

    // A game that restarts a run inside the level gives the beat again.
    app.world_mut().resource_mut::<LevelReady>().restart();
    assert!(!app.world().resource::<LevelReady>().is_live());
}
