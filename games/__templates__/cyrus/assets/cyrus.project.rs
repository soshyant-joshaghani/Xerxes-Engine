//! Project settings: title, flow (splash, main menu, levels with their game modes), bundles
//! to preload, input actions, window options and icon.

use xerxes_engine::prelude::*;

pub fn settings() -> ProjectSettings {
    ProjectSettings {
        title: "Cyrus",
        flow: FlowSettings {
            splash: None,
            main_menu: MainMenuDef {
                title: "Cyrus",
                background: None,
                level_select: false,
                scene: Some(crate::assets::scenes::menu_scene::scene),
            },
            levels: vec![
                LevelDef {
                    name: "Siavash",
                    scene: crate::assets::scenes::games::solo::siavash_scene::scene,
                    mode: GameMode::grid(Sides::Solo, WinCondition::Score),
                },
                LevelDef {
                    name: "Arash's Arrow",
                    scene: crate::assets::scenes::games::solo::arashs_arrow_scene::scene,
                    mode: GameMode::grid(Sides::Solo, WinCondition::Objective),
                },
                LevelDef {
                    name: "Atash",
                    scene: crate::assets::scenes::games::solo::atash_scene::scene,
                    mode: GameMode::grid(Sides::Solo, WinCondition::Survival),
                },
                LevelDef {
                    name: "Nowruz Feast",
                    scene: crate::assets::scenes::games::coop::nowruz_feast_scene::scene,
                    mode: GameMode::grid(Sides::Coop, WinCondition::Score)
                        .with(Modifier::TimeLimit { seconds: 240.0 }),
                },
                LevelDef {
                    name: "Croesus' Gold",
                    scene: crate::assets::scenes::games::coop::croesus_gold_scene::scene,
                    mode: GameMode::grid(Sides::Coop, WinCondition::Objective)
                        .with(Modifier::Escort),
                },
                LevelDef {
                    name: "Pasargadae Depths",
                    scene: crate::assets::scenes::games::coop::pasargadae_depths_scene::scene,
                    mode: GameMode::grid(Sides::Coop, WinCondition::Role).with(Modifier::FogOfWar),
                },
                LevelDef {
                    name: "The Immortals",
                    scene: crate::assets::scenes::games::coop::the_immortals_scene::scene,
                    mode: GameMode::grid(Sides::Coop, WinCondition::Survival)
                        .with(Modifier::Respawn { seconds: 10.0 }),
                },
                LevelDef {
                    name: "Farr",
                    scene: crate::assets::scenes::games::duel::farr_scene::scene,
                    mode: GameMode::grid(Sides::Duel, WinCondition::Score)
                        .with(Modifier::TimeLimit { seconds: 360.0 }),
                },
                LevelDef {
                    name: "Derafsh",
                    scene: crate::assets::scenes::games::duel::derafsh_scene::scene,
                    mode: GameMode::grid(Sides::Duel, WinCondition::Objective)
                        .with(Modifier::Respawn { seconds: 5.0 }),
                },
                LevelDef {
                    name: "Atashkadeh",
                    scene: crate::assets::scenes::games::duel::atashkadeh_scene::scene,
                    mode: GameMode::grid(Sides::Duel, WinCondition::Role).asymmetric(),
                },
                LevelDef {
                    name: "Ramz",
                    scene: crate::assets::scenes::games::duel::ramz_scene::scene,
                    mode: GameMode::grid(Sides::Duel, WinCondition::Survival)
                        .with(Modifier::Permadeath),
                },
                LevelDef {
                    name: "Kuchulu Cup",
                    scene: crate::assets::scenes::games::ngon::kuchulu_cup_scene::scene,
                    mode: GameMode::grid(Sides::NGon(4), WinCondition::Score)
                        .with(Modifier::LowGravity),
                },
                LevelDef {
                    name: "Arzhang",
                    scene: crate::assets::scenes::games::ngon::arzhang_scene::scene,
                    mode: GameMode::grid(Sides::NGon(3), WinCondition::Objective)
                        .with(Modifier::MovingObjective),
                },
                LevelDef {
                    name: "Writing on the Wall",
                    scene: crate::assets::scenes::games::ngon::writing_on_the_wall_scene::scene,
                    mode: GameMode::grid(Sides::NGon(5), WinCondition::Role)
                        .with(Modifier::FogOfWar),
                },
                LevelDef {
                    name: "Sandstorm",
                    scene: crate::assets::scenes::games::ngon::sandstorm_scene::scene,
                    mode: GameMode::grid(Sides::NGon(8), WinCondition::Survival)
                        .with(Modifier::ShrinkingZone)
                        .with(Modifier::Permadeath),
                },
                LevelDef {
                    name: "Pairidaeza",
                    scene: crate::assets::scenes::games::outside::pairidaeza_scene::scene,
                    mode: GameMode::sandbox(),
                },
                LevelDef {
                    name: "Haft Khan",
                    scene: crate::assets::scenes::games::outside::haft_khan_scene::scene,
                    mode: GameMode::new(ModeKind::Progression),
                },
                LevelDef {
                    name: "The Dream of Astyages",
                    scene: crate::assets::scenes::games::outside::dream_of_astyages_scene::scene,
                    mode: GameMode::new(ModeKind::Narrative),
                },
                LevelDef {
                    name: "Ecbatana",
                    scene: crate::assets::scenes::games::outside::ecbatana_scene::scene,
                    mode: GameMode::new(ModeKind::Composite(vec![
                        ModeKind::Sandbox,
                        ModeKind::Progression,
                        ModeKind::Narrative,
                    ])),
                },
                LevelDef {
                    name: "Farr League",
                    scene: crate::assets::scenes::games::outside::farr_league_scene::scene,
                    mode: GameMode::new(ModeKind::Tournament(Box::new(ModeKind::Grid {
                        sides: Sides::Duel,
                        win: WinCondition::Score,
                    }))),
                },
                LevelDef {
                    name: "Tirgan",
                    scene: crate::assets::scenes::games::outside::tirgan_scene::scene,
                    mode: GameMode::grid(Sides::NGon(8), WinCondition::Score)
                        .with(Modifier::Respawn { seconds: 3.0 }),
                },
            ],
        },
        // Nothing here: the menu needs no art of its own, and every level's files come with
        // that level (the flow's Loading step), so a web build downloads what is played.
        preload_bundles: vec![],
        input: InputMap::new()
            // The character's movement and the camera look, from every device. Cyrus is a 3D
            // game: the mouse and the right stick (or the on-screen look pad) turn the view. Siavash
            // reads this same move axis: W jumps, S rolls or dives, A and D switch lanes.
            .vector2(
                "move",
                [
                    Binding::wasd(),
                    Binding::arrows(),
                    Binding::PadStick(PadStick::Left),
                    Binding::PadDpad,
                    Binding::TouchStick("move"),
                ],
            )
            .vector2(
                "look",
                [
                    Binding::mouse_look(0.12),
                    Binding::PadStick(PadStick::Right),
                    Binding::TouchLook {
                        id: "look",
                        scale: 0.2,
                    },
                ],
            )
            // The global jump: Space, the pad's South, the jump touch button, and a double tap on the
            // content (the mouse or a finger; not from the header: the top 16%). Siavash's special
            // ability is this same press.
            .button(
                "jump",
                [
                    Binding::Key(KeyCode::Space),
                    Binding::Pad(GamepadButton::South),
                    Binding::TouchButton("jump"),
                    Binding::DoubleTap { top_inset: 0.16 },
                ],
            )
            // Siavash's drags, by the mouse or a finger (not from the header: the header is the top 16%).
            // A swipe up jumps, a swipe down rolls or dives, a swipe side switches lane.
            .button(
                "swipe_left",
                [Binding::Swipe {
                    dir: Swipe::Left,
                    top_inset: 0.16,
                }],
            )
            .button(
                "swipe_right",
                [Binding::Swipe {
                    dir: Swipe::Right,
                    top_inset: 0.16,
                }],
            )
            .button(
                "swipe_up",
                [Binding::Swipe {
                    dir: Swipe::Up,
                    top_inset: 0.16,
                }],
            )
            .button(
                "swipe_down",
                [Binding::Swipe {
                    dir: Swipe::Down,
                    top_inset: 0.16,
                }],
            )
            .button(
                "sprint",
                [
                    Binding::Key(KeyCode::ShiftLeft),
                    Binding::Pad(GamepadButton::LeftThumb),
                    Binding::TouchButton("sprint"),
                ],
            )
            .button(
                "interact",
                [
                    Binding::Key(KeyCode::KeyE),
                    Binding::Pad(GamepadButton::West),
                    Binding::TouchButton("interact"),
                ],
            )
            .button(
                "fire",
                [
                    Binding::Mouse(MouseButton::Left),
                    Binding::Pad(GamepadButton::RightTrigger2),
                    Binding::TouchButton("fire"),
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
                [
                    Binding::Key(KeyCode::KeyM),
                    Binding::Pad(GamepadButton::Mode),
                ],
            )
            // Arash's Arrow diorama (and later board games): cursor + sluice commands.
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
                [
                    Binding::Key(KeyCode::Space),
                    Binding::Pad(GamepadButton::South),
                ],
            )
            .button(
                "undo",
                [
                    Binding::Key(KeyCode::KeyZ),
                    Binding::Pad(GamepadButton::West),
                ],
            )
            .button(
                "clear",
                [
                    Binding::Key(KeyCode::KeyC),
                    Binding::Pad(GamepadButton::North),
                ],
            )
            .button(
                "open",
                [
                    Binding::Key(KeyCode::Enter),
                    Binding::Pad(GamepadButton::East),
                ],
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
        physics: PhysicsSettings::default(),
    }
}

/// The project's logic plugins (`src/logic.rs`), in order. Add-ons the editor adds are listed here.
pub const LOGIC: &[fn(&mut ::bevy::app::App)] = &[
    crate::logic::arashs_arrow,
    crate::logic::pairidaeza,
    crate::logic::siavash,
];
