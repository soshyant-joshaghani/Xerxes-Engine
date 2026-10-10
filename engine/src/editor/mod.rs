//! The engine app: the editor where games are made, laid out like Unity (toolbar,
//! Hierarchy, Scene view, Inspector, Project / Console). Dioxus draws the editor UI, Bevy
//! runs the Scene view, and the two only talk through the bridge.
//!
//! Games never use this module: their UI is bevy_ui (see [`crate::Scene`] and
//! `modules::assets`).
//!
//! - `bridge`    UI → runtime commands, runtime → UI snapshots (no Bevy, no Dioxus), with the
//!               editor's `protocol` (commands, snapshot) and `view` (what the Scene view draws)
//! - `layout`    the Blender-style screen: workspaces of areas, each showing one editor (plain
//!               data and a solver, shared by the UI and the Scene view)
//! - `host`      where Dioxus runs: the DOM on the web, Blitz over the Bevy window natively
//! - `project`   the project store, the backend client, the `.rs` asset codec, the browser model
//! - `stage`     the Scene view (Bevy): editor camera, scene objects, picking, and `runtime`,
//!               the bridge's Bevy end (a resource, and commands as messages)
//! - `ui`        the panels (Dioxus)
//! - `theme`     the look, and the panel sizes shared by the UI and the Scene view

pub mod bridge;
pub mod history;
pub mod host;
pub mod keymap;
pub mod layout;
pub mod prefs;
pub mod project;
pub mod theme;
pub mod touch;

mod stage;
mod ui;

// Where the bridge's data and the Bevy end live (kept at their short paths).
pub use bridge::{protocol, view};
pub use stage::runtime;

use bevy::prelude::*;

use crate::modules::runtime::primary_window;
pub use bridge::{RuntimePort, UiPort, bridge};
use protocol::{EditorCommand, EditorSnapshot};
pub use runtime::{UiBridge, UiBridgePlugin, UiCommand};

const TITLE: &str = "Xerxes Engine";

/// The editor's Bevy app. `canvas` is the host canvas selector on the web, `None` natively.
fn app(port: RuntimePort<EditorCommand, EditorSnapshot>, canvas: Option<String>) -> App {
    #[allow(unused_mut)]
    let mut window = primary_window(TITLE, canvas);
    // Dev/QA hook (native): `XERXES_WINDOW=1100x519` opens a window of that logical size, to
    // check a phone-sized editor on a desktop.
    #[cfg(not(target_arch = "wasm32"))]
    if let (Some(size), Some(primary)) = (
        std::env::var("XERXES_WINDOW").ok().and_then(|text| {
            let (w, h) = text.split_once('x')?;
            Some((w.trim().parse::<u32>().ok()?, h.trim().parse::<u32>().ok()?))
        }),
        window.primary_window.as_mut(),
    ) {
        primary.resolution = size.into();
    }
    let plugins = DefaultPlugins.set(window);
    // On the web the Dioxus host already installs the global logger.
    #[cfg(target_arch = "wasm32")]
    let plugins = plugins.disable::<bevy::log::LogPlugin>();
    // Blitz 0.2 builds the corner arcs of a zero-width border with NaN whenever an element has
    // a border-radius; vello_cpu drops that (invisible) path and warns on every repaint.
    #[cfg(not(target_arch = "wasm32"))]
    let plugins = plugins.set(bevy::log::LogPlugin {
        filter: format!("{},vello_common::flatten=error", bevy::log::DEFAULT_FILTER),
        ..default()
    });

    let mut app = App::new();
    app.add_plugins((plugins, UiBridgePlugin::new(port), stage::StagePlugin));
    // Closing the editor stops the jobs it started (a dev server, a build).
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Last, |mut exit: MessageReader<AppExit>| {
        if exit.read().next().is_some() {
            project::jobs::local::stop_all();
        }
    });
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(path) = std::env::var("XERXES_SCREENSHOT") {
        app.add_systems(Update, screenshot_and_exit(path));
    }
    app
}

/// Dev/QA hook (native): `XERXES_SCREENSHOT=<file.png>` saves the editor window (world and
/// UI, as rendered) after it settles, then exits. Used to check native rendering without an
/// OS screen capture.
#[cfg(not(target_arch = "wasm32"))]
fn screenshot_and_exit(path: String) -> impl FnMut(Commands, Res<Time>, MessageWriter<AppExit>) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    let mut taken = false;
    move |mut commands, time, mut exit| {
        let seconds = time.elapsed_secs();
        if !taken && seconds > 8.0 {
            taken = true;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path.clone()));
        }
        if seconds > 10.0 {
            exit.write(AppExit::Success);
        }
    }
}

/// Runs the editor: the Dioxus page with Bevy in a canvas on the web; a Bevy window with the
/// Dioxus UI drawn over it on Windows, macOS, Linux and Android.
pub fn launch() {
    host::launch(app, ui::root);
}
