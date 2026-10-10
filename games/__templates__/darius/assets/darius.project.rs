//! Project settings: title, flow (splash, main menu, levels with their game modes), bundles
//! to preload, input actions, window options and icon.

use xerxes_engine::prelude::*;

pub fn settings() -> ProjectSettings {
    ProjectSettings {
        title: "Darius",
        flow: FlowSettings {
            splash: None,
            main_menu: MainMenuDef {
                title: "Darius",
                background: None,
                level_select: false,
                scene: Some(crate::assets::scenes::menu_scene::scene),
            },
            levels: vec![LevelDef {
                name: "Coin Dash",
                scene: crate::assets::scenes::arena_scene::scene,
                mode: GameMode::grid(Sides::Solo, WinCondition::Objective)
                    .with(Modifier::TimeLimit { seconds: 90.0 }),
            }],
        },
        preload_bundles: vec![],
        input: InputMap::new()
            .action(
                "move_left",
                [
                    Binding::Key(KeyCode::KeyA),
                    Binding::Key(KeyCode::ArrowLeft),
                ],
            )
            .action(
                "move_right",
                [
                    Binding::Key(KeyCode::KeyD),
                    Binding::Key(KeyCode::ArrowRight),
                ],
            )
            .action(
                "move_up",
                [Binding::Key(KeyCode::KeyW), Binding::Key(KeyCode::ArrowUp)],
            )
            .action(
                "move_down",
                [
                    Binding::Key(KeyCode::KeyS),
                    Binding::Key(KeyCode::ArrowDown),
                ],
            )
            .action("respawn", [Binding::Key(KeyCode::KeyR)])
            .action(
                "pause",
                [Binding::Key(KeyCode::Escape), Binding::Key(KeyCode::KeyP)],
            )
            .action("exit_to_menu", [Binding::Key(KeyCode::KeyM)]),
        window: WindowSettings {
            width: 1280,
            height: 720,
            resizable: true,
        },
        icon: None,
        backend: BackendSettings {
            dev: "http://127.0.0.1:8000",
            publish: "",
        },
    }
}
