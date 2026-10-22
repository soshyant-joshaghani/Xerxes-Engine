//! Gameplay and editor viewport cameras.
//!
//! - [`GameCamera2D`] / [`GameCamera3D`]: authored in scenes for actual gameplay. Inserting them
//!   pulls in Bevy's `Camera2d` / `Camera3d` via required components. Projection
//!   (perspective or orthographic) stays a separate `Projection` component.
//! - [`ScalingMode`]: how an orthographic game camera frames the world (`AutoMin`, `Fixed`, …);
//!   re-exported so scenes use the prelude, not `bevy::camera`.
//! - [`Sky`]: the colour a game camera clears to — a 3D scene's sky. Without one a 3D camera
//!   keeps Bevy's default grey, which is why an empty scene looks unfinished.
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

/// The colour a game camera clears to: the sky of a 3D scene, the backdrop of a 2D one.
/// Authored as `.with(Sky(Color::srgb(…)))`, usually on the camera prefab so every scene of a
/// game starts with the same sky.
///
/// It is not the bands' [`frame`](crate::modules::ui::BandSettings::frame): that one is the
/// letterbox a zoomed-out 2D camera leaves around the game, and a scene with a `Sky` keeps its
/// own colour instead.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Sky(pub Color);

/// Editor Scene-view camera in the flat (2D) mode.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ViewportCamera2D;

/// Editor Scene-view camera in the 3D (perspective or ortho) mode.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ViewportCamera3D;

/// Installed by `launch_project` and the editor stage: keeps every camera clearing to its
/// [`Sky`].
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_sky);
    }
}

/// A camera with a [`Sky`] clears to it, in the game and in the editor's Scene view alike.
fn apply_sky(mut cameras: Query<(&Sky, &mut Camera), Changed<Sky>>) {
    // `Changed<Sky>` is the guard: `ClearColorConfig` is not comparable, so the write cannot be
    // skipped by looking at it, and it must not run every frame (it would wake change detection
    // on every camera).
    for (Sky(colour), mut camera) in &mut cameras {
        camera.clear_color = bevy::camera::ClearColorConfig::Custom(*colour);
    }
}
