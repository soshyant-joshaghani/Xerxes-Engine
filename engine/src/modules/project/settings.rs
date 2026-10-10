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
//!             .action("move_up", [Binding::Key(KeyCode::KeyW), Binding::Key(KeyCode::ArrowUp)])
//!             .action("pause", [Binding::Key(KeyCode::Escape)]),
//!         window: WindowSettings::default(),
//!         icon: None,
//!         backend: BackendSettings::default(),
//!     }
//! }
//! ```
//!
//! Gameplay reads input only through named actions (`actions.pressed("move_up")`), so
//! remapping happens in one place on every platform.

use bevy::prelude::*;
use std::collections::HashSet;

use crate::modules::flow::FlowSettings;

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

/// What can trigger an action.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Binding {
    Key(KeyCode),
    Mouse(MouseButton),
}

/// Named actions and their bindings, in declaration order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InputMap {
    pub actions: Vec<(&'static str, Vec<Binding>)>,
}

impl InputMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds (or replaces) an action.
    pub fn action(
        mut self,
        name: &'static str,
        bindings: impl IntoIterator<Item = Binding>,
    ) -> Self {
        self.actions.retain(|(n, _)| *n != name);
        self.actions.push((name, bindings.into_iter().collect()));
        self
    }
}

/// The actions' state this frame. Read it from gameplay systems.
#[derive(Resource, Debug, Default)]
pub struct Actions {
    map: InputMap,
    pressed: HashSet<&'static str>,
    just_pressed: HashSet<&'static str>,
    just_released: HashSet<&'static str>,
}

impl Actions {
    pub fn new(map: InputMap) -> Self {
        Self { map, ..default() }
    }

    pub fn pressed(&self, action: &str) -> bool {
        self.pressed.contains(action)
    }

    pub fn just_pressed(&self, action: &str) -> bool {
        self.just_pressed.contains(action)
    }

    pub fn just_released(&self, action: &str) -> bool {
        self.just_released.contains(action)
    }

    /// -1, 0 or 1 from a pair of actions.
    pub fn axis(&self, negative: &str, positive: &str) -> f32 {
        self.pressed(positive) as i8 as f32 - self.pressed(negative) as i8 as f32
    }

    /// Recomputes the state from raw input. `pressed(binding)` / `just_*` answer per binding.
    pub fn update(
        &mut self,
        pressed: impl Fn(Binding) -> bool,
        just_pressed: impl Fn(Binding) -> bool,
        just_released: impl Fn(Binding) -> bool,
    ) {
        self.pressed.clear();
        self.just_pressed.clear();
        self.just_released.clear();
        for (name, bindings) in &self.map.actions {
            let any = |test: &dyn Fn(Binding) -> bool| bindings.iter().any(|b| test(*b));
            if any(&pressed) {
                self.pressed.insert(name);
            }
            if any(&just_pressed) {
                self.just_pressed.insert(name);
            }
            if any(&just_released) {
                self.just_released.insert(name);
            }
        }
    }
}

/// Updates [`Actions`] from keyboard and mouse, before gameplay runs.
pub(super) fn update_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut actions: ResMut<Actions>,
) {
    let read = |state: fn(&ButtonInput<KeyCode>, KeyCode) -> bool,
                mouse_state: fn(&ButtonInput<MouseButton>, MouseButton) -> bool| {
        let keys = &keys;
        let mouse = &mouse;
        move |binding: Binding| match binding {
            Binding::Key(key) => state(keys, key),
            Binding::Mouse(button) => mouse_state(mouse, button),
        }
    };
    let pressed = read(|i, k| i.pressed(k), |i, b| i.pressed(b));
    let just_pressed = read(|i, k| i.just_pressed(k), |i, b| i.just_pressed(b));
    let just_released = read(|i, k| i.just_released(k), |i, b| i.just_released(b));
    actions.update(pressed, just_pressed, just_released);
}
