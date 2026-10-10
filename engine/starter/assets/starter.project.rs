//! Project settings: title, flow (splash, main menu, levels with their game modes), bundles
//! to preload, input actions, window options and icon.

use xerxes_engine::prelude::*;

pub fn settings() -> ProjectSettings {
    ProjectSettings {
        title: "Starter",
        flow: FlowSettings {
            splash: None,
            main_menu: MainMenuDef {
                title: "Starter",
                background: None,
                level_select: false,
                scene: None,
            },
            levels: vec![LevelDef {
                name: "Main",
                scene: crate::assets::scenes::main_scene::scene,
                mode: GameMode::sandbox(),
            }],
        },
        preload_bundles: vec![],
        // The actions the engine templates (camera, character, pause, HUD) read, so they work
        // as soon as you add one. Change the keys here; gameplay only names the actions.
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
        // The game's backend: the local dev backend in dev builds; set `publish` to the
        // deployed one (empty: the release runs offline).
        backend: BackendSettings {
            dev: "http://127.0.0.1:8000",
            publish: "",
        },
    }
}
