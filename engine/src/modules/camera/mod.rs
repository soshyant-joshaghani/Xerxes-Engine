//! Gameplay and editor viewport cameras.
//!
//! - [`GameCamera2D`] / [`GameCamera3D`]: authored in scenes for actual gameplay. Inserting them
//!   pulls in Bevy's `Camera2d` / `Camera3d` via required components. Projection
//!   (perspective or orthographic) stays a separate `Projection` component.
//! - [`ScalingMode`]: how an orthographic game camera frames the world (`AutoMin`, `Fixed`, …);
//!   re-exported so scenes use the prelude, not `bevy::camera`.
//! - [`ViewportCamera2D`] / [`ViewportCamera3D`]: markers on the editor's Scene-view cameras
//!   only (never saved in a scene file).

use bevy::prelude::*;

pub use bevy::camera::ScalingMode;

/// Gameplay camera for a 2D scene. Authored as `.with(GameCamera2D)`.
#[derive(Component, Debug, Clone, Copy, Default)]
#[require(Camera2d)]
pub struct GameCamera2D;

/// Gameplay camera for a 3D scene. Authored as `.with(GameCamera3D)`.
#[derive(Component, Debug, Clone, Copy, Default)]
#[require(Camera3d)]
pub struct GameCamera3D;

/// Editor Scene-view camera in the flat (2D) mode.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ViewportCamera2D;

/// Editor Scene-view camera in the 3D (perspective or ortho) mode.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ViewportCamera3D;

/// Installed by `launch_project` and the editor stage (markers need no systems today).
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, _app: &mut App) {}
}
