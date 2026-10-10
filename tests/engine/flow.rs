//! The game flow, headless: Preload → Splash → MainMenu → Level, level changes despawn the
//! previous level, the last LevelComplete returns to the menu; game modes validate against
//! the taxonomy.

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
    run(&mut app, 2);
    assert_eq!(state(&app), Flow::Level);
    assert!(has(&mut app, "Level One Thing"));
    let mode = app.world().resource::<ActiveGameMode>().0.clone();
    assert_eq!(mode.cell_label(), "Challenge clear");

    // Something a logic module spawned for the level goes with it.
    app.world_mut().spawn((LevelEntity, Name::new("HUD root")));

    app.world_mut().write_message(LevelComplete);
    run(&mut app, 2);
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
    run(&mut app, 3);
    assert_eq!(state(&app), Flow::Level);
    assert_eq!(app.world().resource::<CurrentLevel>().0, 1);
    assert!(!has(&mut app, "Menu Title"), "the menu scene was despawned");
    assert!(has(&mut app, "Level Two Thing"));

    // Back at the menu it is spawned again.
    app.world_mut().write_message(ExitToMenu);
    run(&mut app, 2);
    assert!(has(&mut app, "Menu Title"));
}

#[test]
fn exit_to_menu_leaves_the_level() {
    let mut app = app(false);
    run(&mut app, 3);
    app.world_mut().write_message(LoadLevel(1));
    run(&mut app, 2);
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
