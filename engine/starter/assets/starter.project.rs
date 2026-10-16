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
        // as soon as you add one. Gameplay only names the actions; this is where each device
        // (keyboard, mouse, gamepad, on-screen touch controls) is mapped to them. Mouse, keyboard
        // and gamepads work on phones too.
        input: InputMap::new()
            .vector2(
                "move",
                [
                    Binding::wasd(),
                    Binding::arrows(),
                    Binding::PadStick(PadStick::Left),
                    Binding::PadDpad,
                    // The on-screen stick whose source is "move" (a VirtualJoystick on a Canvas).
                    Binding::TouchStick("move"),
                ],
            )
            .button(
                "respawn",
                [
                    Binding::Key(KeyCode::KeyR),
                    Binding::Pad(GamepadButton::Select),
                ],
            )
            .button(
                "pause",
                [
                    Binding::Key(KeyCode::Escape),
                    Binding::Key(KeyCode::KeyP),
                    Binding::Pad(GamepadButton::Start),
                ],
            )
            .button(
                "exit_to_menu",
                [Binding::Key(KeyCode::KeyM), Binding::Pad(GamepadButton::Mode)],
            ),
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
        physics: PhysicsSettings::default(),
    }
}

/// The project's logic plugins, in order. The editor lists the add-ons it adds here (`src/addons/`).
pub const LOGIC: &[fn(&mut ::bevy::app::App)] = &[];
