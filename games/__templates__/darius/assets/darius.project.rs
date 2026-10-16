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
            levels: vec![
                LevelDef {
                    name: "Ahriman's Horde",
                    scene: crate::assets::scenes::games::solo::ahrimans_horde_scene::scene,
                    mode: GameMode::grid(Sides::Solo, WinCondition::Survival)
                        .with(Modifier::Permadeath),
                },
                LevelDef {
                    name: "Qanat",
                    scene: crate::assets::scenes::games::solo::qanat_scene::scene,
                    mode: GameMode::grid(Sides::Solo, WinCondition::Objective),
                },
                LevelDef {
                    name: "Chapar",
                    scene: crate::assets::scenes::games::solo::chapar_scene::scene,
                    mode: GameMode::grid(Sides::Solo, WinCondition::Score),
                },
                LevelDef {
                    name: "Tombak Duo",
                    scene: crate::assets::scenes::games::coop::tombak_duo_scene::scene,
                    mode: GameMode::grid(Sides::Coop, WinCondition::Score),
                },
                LevelDef {
                    name: "Bound Bros",
                    scene: crate::assets::scenes::games::coop::bound_bros_scene::scene,
                    mode: GameMode::grid(Sides::Coop, WinCondition::Objective),
                },
                LevelDef {
                    name: "Behistun",
                    scene: crate::assets::scenes::games::coop::behistun_scene::scene,
                    mode: GameMode::grid(Sides::Coop, WinCondition::Role).with(Modifier::FogOfWar),
                },
                LevelDef {
                    name: "Naft",
                    scene: crate::assets::scenes::games::coop::naft_scene::scene,
                    mode: GameMode::grid(Sides::Coop, WinCondition::Survival),
                },
                LevelDef {
                    name: "Sofreh",
                    scene: crate::assets::scenes::games::duel::sofreh_scene::scene,
                    mode: GameMode::grid(Sides::Duel, WinCondition::Score)
                        .with(Modifier::TimeLimit { seconds: 180.0 }),
                },
                LevelDef {
                    name: "Siege of Sardis",
                    scene: crate::assets::scenes::games::duel::siege_of_sardis_scene::scene,
                    mode: GameMode::grid(Sides::Duel, WinCondition::Objective)
                        .with(Modifier::Escort)
                        .with(Modifier::Respawn { seconds: 3.0 })
                        .with(Modifier::TimeLimit { seconds: 300.0 }),
                },
                LevelDef {
                    name: "Karvan",
                    scene: crate::assets::scenes::games::duel::karvan_scene::scene,
                    mode: GameMode::grid(Sides::Duel, WinCondition::Role),
                },
                LevelDef {
                    name: "Mobarezeh",
                    scene: crate::assets::scenes::games::duel::mobarezeh_scene::scene,
                    mode: GameMode::grid(Sides::Duel, WinCondition::Survival)
                        .with(Modifier::Permadeath),
                },
                LevelDef {
                    name: "Zahhak",
                    scene: crate::assets::scenes::games::ngon::zahhak_scene::scene,
                    mode: GameMode::grid(Sides::NGon(4), WinCondition::Score)
                        .with(Modifier::TimeLimit { seconds: 180.0 }),
                },
                LevelDef {
                    name: "Kashi",
                    scene: crate::assets::scenes::games::ngon::kashi_scene::scene,
                    mode: GameMode::grid(Sides::NGon(4), WinCondition::Objective)
                        .with(Modifier::MovingObjective)
                        .with(Modifier::TimeLimit { seconds: 180.0 }),
                },
                LevelDef {
                    name: "The False Smerdis",
                    scene: crate::assets::scenes::games::ngon::false_smerdis_scene::scene,
                    mode: GameMode::grid(Sides::NGon(5), WinCondition::Role)
                        .with(Modifier::FogOfWar),
                },
                LevelDef {
                    name: "Palang",
                    scene: crate::assets::scenes::games::ngon::palang_scene::scene,
                    mode: GameMode::grid(Sides::NGon(4), WinCondition::Survival)
                        .with(Modifier::ShrinkingZone)
                        .with(Modifier::Permadeath),
                },
                LevelDef {
                    name: "Jamshid's Workshop",
                    scene: crate::assets::scenes::games::outside::jamshids_workshop_scene::scene,
                    mode: GameMode::sandbox().with(Modifier::LowGravity),
                },
                LevelDef {
                    name: "Caravanserai",
                    scene: crate::assets::scenes::games::outside::caravanserai_scene::scene,
                    mode: GameMode::new(ModeKind::Progression),
                },
                LevelDef {
                    name: "Naqqal's Teahouse",
                    scene: crate::assets::scenes::games::outside::naqqals_teahouse_scene::scene,
                    mode: GameMode::new(ModeKind::Narrative),
                },
                LevelDef {
                    name: "Persepolis",
                    scene: crate::assets::scenes::games::outside::persepolis_scene::scene,
                    mode: GameMode::new(ModeKind::Composite(vec![
                        ModeKind::Sandbox,
                        ModeKind::Progression,
                        ModeKind::Narrative,
                    ])),
                },
                LevelDef {
                    name: "Zurkhaneh Cup",
                    scene: crate::assets::scenes::games::outside::zurkhaneh_cup_scene::scene,
                    mode: GameMode::new(ModeKind::Tournament(Box::new(ModeKind::Grid {
                        sides: Sides::Duel,
                        win: WinCondition::Survival,
                    }))),
                },
                LevelDef {
                    name: "Maydan",
                    scene: crate::assets::scenes::games::outside::maydan_scene::scene,
                    mode: GameMode::grid(Sides::NGon(8), WinCondition::Score)
                        .with(Modifier::Respawn { seconds: 3.0 }),
                },
            ],
        },
        preload_bundles: vec![],
        input: InputMap::new()
            // Movement: every device drives the one "move" vector. Keyboard WASD and arrows, the
            // gamepad's left stick and D-pad, the floating on-screen stick or the on-screen D-pad
            // (the footer's two controls, `source("move")` and the `dpad_*` buttons).
            .vector2(
                "move",
                [
                    Binding::wasd(),
                    Binding::arrows(),
                    Binding::PadStick(PadStick::Left),
                    Binding::PadDpad,
                    Binding::TouchStick("move"),
                    Binding::TouchDpad {
                        up: "dpad_up",
                        down: "dpad_down",
                        left: "dpad_left",
                        right: "dpad_right",
                    },
                ],
            )
            .vector2(
                "aim",
                [
                    Binding::mouse_look(1.0),
                    Binding::PadStick(PadStick::Right),
                ],
            )
            // Daric: Space / RMB fire; R = Reload-or-Dash; F = Switch / Interact.
            .button(
                "fire",
                [
                    Binding::Key(KeyCode::Space),
                    Binding::Mouse(MouseButton::Right),
                    Binding::Pad(GamepadButton::RightTrigger2),
                    Binding::TouchButton("fire"),
                ],
            )
            .button(
                "interact",
                [
                    Binding::Key(KeyCode::KeyF),
                    Binding::Key(KeyCode::KeyE),
                    Binding::Pad(GamepadButton::West),
                    Binding::TouchButton("interact"),
                ],
            )
            .button(
                "reload_dash",
                [
                    Binding::Key(KeyCode::KeyR),
                    Binding::Pad(GamepadButton::East),
                    Binding::TouchButton("reload_dash"),
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
                "respawn",
                [
                    Binding::Key(KeyCode::Backspace),
                    Binding::Pad(GamepadButton::Select),
                ],
            )
            // Chapar: jump (hold for height) and duck; the footer's two touch buttons.
            .button(
                "jump",
                [
                    Binding::Key(KeyCode::Space),
                    Binding::Key(KeyCode::ArrowUp),
                    Binding::Key(KeyCode::KeyW),
                    Binding::Pad(GamepadButton::South),
                    Binding::TouchButton("jump"),
                ],
            )
            .button(
                "duck",
                [
                    Binding::Key(KeyCode::ArrowDown),
                    Binding::Key(KeyCode::KeyS),
                    Binding::Pad(GamepadButton::East),
                    Binding::TouchButton("duck"),
                ],
            )
            .button("exit_to_menu", [Binding::Key(KeyCode::KeyM), Binding::Pad(GamepadButton::Mode)])
            // Qanat's board: the cursor steps one cell per press (keys and the gamepad D-pad; on a
            // phone the board is tapped), and its five commands.
            .button(
                "cell_left",
                [
                    Binding::Key(KeyCode::ArrowLeft),
                    Binding::Key(KeyCode::KeyA),
                    Binding::Pad(GamepadButton::DPadLeft),
                ],
            )
            .button(
                "cell_right",
                [
                    Binding::Key(KeyCode::ArrowRight),
                    Binding::Key(KeyCode::KeyD),
                    Binding::Pad(GamepadButton::DPadRight),
                ],
            )
            .button(
                "cell_up",
                [
                    Binding::Key(KeyCode::ArrowUp),
                    Binding::Key(KeyCode::KeyW),
                    Binding::Pad(GamepadButton::DPadUp),
                ],
            )
            .button(
                "cell_down",
                [
                    Binding::Key(KeyCode::ArrowDown),
                    Binding::Key(KeyCode::KeyS),
                    Binding::Pad(GamepadButton::DPadDown),
                ],
            )
            .button(
                "turn",
                [Binding::Key(KeyCode::Space), Binding::Pad(GamepadButton::South)],
            )
            .button(
                "undo",
                [Binding::Key(KeyCode::KeyZ), Binding::Pad(GamepadButton::West)],
            )
            .button(
                "clear",
                [Binding::Key(KeyCode::KeyC), Binding::Pad(GamepadButton::North)],
            )
            .button(
                "open",
                [Binding::Key(KeyCode::Enter), Binding::Pad(GamepadButton::East)],
            ),
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
        // Top-down default; Jamshid's Workshop raises gravity with its slider at runtime.
        physics: PhysicsSettings {
            gravity: [0.0, 0.0],
            layers: &["Default", "Toy"],
        },
    }
}

/// The project's logic plugins (`src/logic.rs`), in order. Add-ons the editor adds are listed here.
pub const LOGIC: &[fn(&mut ::bevy::app::App)] = &[
    crate::logic::chapar,
    crate::logic::ahrimans_horde,
    crate::logic::jamshids_workshop,
    crate::logic::qanat,
];
