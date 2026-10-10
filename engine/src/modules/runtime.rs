//! The Bevy side shared by games and the editor: one window setup for every platform.

use bevy::prelude::*;
use bevy::window::WindowPlugin;

/// The primary window. On the web, `canvas` is a CSS selector for an existing `<canvas>`
/// (the editor's Dioxus host renders one); with `None` Bevy adds its own canvas to the page.
/// Either way it fits the canvas to its parent. On windows and android `canvas` is ignored and
/// a regular window opens.
pub fn primary_window(title: impl Into<String>, canvas: Option<String>) -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: title.into(),
            canvas,
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }
}
