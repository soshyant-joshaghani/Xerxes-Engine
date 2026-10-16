//! Project settings: a game's top-level config (`assets/<name>.project.rs`), like Unity's
//! or Unreal's Project Settings (title, flow and levels, preload bundles, input, platform
//! options, icon, the game's backend), and the input manager it configures.
//!
//! ```ignore
//! pub fn settings() -> ProjectSettings {
//!     ProjectSettings {
//!         title: "My Game",
//!         flow: FlowSettings {
//!             splash: Some(SplashDef { title: "My Game", image: None, seconds: 2.0 }),
//!             main_menu: MainMenuDef { title: "My Game", background: None, level_select: true, scene: None },
//!             levels: vec![LevelDef {
//!                 name: "Level 1",
//!                 scene: crate::assets::scenes::main_scene::scene,
//!                 mode: GameMode::grid(Sides::Solo, WinCondition::Objective),
//!             }],
//!         },
//!         preload_bundles: vec!["core"],
//!         input: InputMap::new()
//!             .vector2("move", [Binding::wasd(), Binding::arrows(), Binding::PadStick(PadStick::Left)])
//!             .button("pause", [Binding::Key(KeyCode::Escape), Binding::Pad(GamepadButton::Start)]),
//!         window: WindowSettings::default(),
//!         icon: None,
//!         backend: BackendSettings::default(),
//!         physics: PhysicsSettings::default(),
//!     }
//! }
//! ```
//!
//! Gameplay reads input only through named actions (`actions.vector("move")`), so each game maps
//! them to the keyboard, mouse, gamepad and touch controls in this one place, on every platform.
//! The input manager itself is `modules::input`.

use bevy::prelude::*;

use crate::modules::flow::FlowSettings;
pub use crate::modules::input::InputMap;

pub struct ProjectSettings {
    /// Window title on desktop and Android; the web page title comes from the game's Dioxus.toml.
    pub title: &'static str,
    /// Splash, main menu and the ordered levels (each a scene and its game mode).
    pub flow: FlowSettings,
    /// Bundles downloaded and extracted before the start scene, on every platform.
    pub preload_bundles: Vec<&'static str>,
    /// Named actions and their bindings.
    pub input: InputMap,
    /// Desktop window options. The web canvas fills the page; Android is fullscreen.
    pub window: WindowSettings,
    /// The game icon (an image in a bundle), for platforms that show one. Not applied yet.
    pub icon: Option<&'static str>,
    /// Where the game's backend is (accounts, multiplayer, leaderboards).
    pub backend: BackendSettings,
    /// Default gravity and named collision layers for games that use physics.
    pub physics: PhysicsSettings,
}

/// Project-wide physics defaults. A logic module still chooses whether to add
/// [`crate::modules::physics::PhysicsPlugin`]; these numbers are the project's starting
/// gravity and the names of the collision layers (index `i` is bit `1 << i`). A level can
/// change gravity at runtime (Jamshid's Workshop's slider).
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct PhysicsSettings {
    /// World gravity in units per second squared. Top-down: `[0.0, 0.0]`. Side-view pixels:
    /// `[0.0, -980.0]`.
    pub gravity: [f32; 2],
    /// Named layers, up to 32. Empty means the engine default (`Default` on bit 0).
    pub layers: &'static [&'static str],
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: [0.0, 0.0],
            layers: &["Default"],
        }
    }
}

/// The game's backend: the Xerxes backend (`backend/`) or the game's own deployment of it.
/// The game reads the URL for its build from the [`GameBackend`] resource.
#[derive(Debug, Clone, PartialEq)]
pub struct BackendSettings {
    /// Development builds (`game dev`): the local dev backend. `127.0.0.1` works on desktop and,
    /// through `adb reverse tcp:8000 tcp:8000` (which `game dev <x> android` runs), on a phone.
    pub dev: &'static str,
    /// Release builds (`game publish`): the deployed backend. Empty: the game runs offline.
    pub publish: &'static str,
}

impl Default for BackendSettings {
    fn default() -> Self {
        Self {
            dev: "http://127.0.0.1:8000",
            publish: "",
        }
    }
}

impl BackendSettings {
    /// The URL for this build: `dev` in debug builds, `publish` in release builds. `None`
    /// when it is empty (no backend).
    pub fn url(&self) -> Option<&'static str> {
        let url = if cfg!(debug_assertions) {
            self.dev
        } else {
            self.publish
        };
        (!url.is_empty()).then_some(url)
    }
}

/// The backend this build of the game talks to (from [`BackendSettings`]).
#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct GameBackend(pub Option<&'static str>);

#[derive(Debug, Clone, PartialEq)]
pub struct WindowSettings {
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            resizable: true,
        }
    }
}
