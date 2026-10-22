//! Scenes: a game's Bevy world. A game implements [`Scene`] and calls [`launch`].
//!
//! Games are Bevy only: their HUD and menus are bevy_ui, built in [`Scene::build`] like any
//! other system. Dioxus is the engine editor's UI and never ships in a game.
//!
//! ```text
//! Bevy App: DefaultPlugins (window / canvas) + Scene::build (world, systems, bevy_ui)
//! ```

use bevy::prelude::*;

use super::runtime::primary_window;

/// One playable scene.
pub trait Scene: 'static {
    /// Window title (windows, android). On the web the page title comes from the game's Dioxus.toml.
    const TITLE: &'static str;

    /// Adds the scene's resources, systems, startup and bevy_ui to the app.
    fn build(app: &mut App);
}

/// The Bevy app for a scene, with the window set up for every platform.
pub fn app<S: Scene>() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(primary_window(S::TITLE, None)));
    S::build(&mut app);
    app
}

/// Runs a scene: a Bevy window natively, a Bevy canvas on the web.
pub fn launch<S: Scene>() {
    app::<S>().run();
}
