//! Cross-platform **safe area**: the insets (in logical px) a device reserves at the edges of
//! the window for system UI it draws on top — the navigation bar, and a display cutout (camera
//! notch) the app draws into. Interactive UI should stay out of these bands.
//!
//! The engine draws edge-to-edge on Android (the game fills the cutout, see each game's
//! `styles.xml` / `windowLayoutInDisplayCutoutMode`), so content is responsible for keeping its
//! buttons clear of the nav bar and the cutout. [`SafeArea`] gives games and the editor the insets
//! to pad or shift by; it is `0` on web and desktop, where there is nothing to inset.
//!
//! On Android the insets come from `AndroidApp::content_rect()` (the area the window's content
//! should sit in; `bevy::android::ANDROID_APP` is the same handle `bevy_winit` uses), compared to
//! the window's physical size, divided by the scale factor. The content rect changes when the
//! system bars or the cutout change (orientation, gesture vs 3-button nav), so [`refresh`] reads
//! it every frame.

use bevy::prelude::*;

/// The device-reserved insets at the window edges, in **logical px**. `0` on web and desktop;
/// on Android, the space taken by the navigation bar and the display cutout. A game pads its HUD
/// (or shifts its world) by these; the editor insets its stage the same way.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct SafeArea {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Installs [`SafeArea`] and keeps it current. Added by [`crate::modules::ui::UiPlugin`], so every
/// game (`launch_project`) and the editor's stage get it.
pub struct SafeAreaPlugin;

impl Plugin for SafeAreaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SafeArea>()
            .add_systems(PreUpdate, refresh);
    }
}

/// Recomputes [`SafeArea`] from the platform. A no-op off Android (the resource stays `0`).
#[cfg(target_os = "android")]
fn refresh(windows: Query<&Window, With<bevy::window::PrimaryWindow>>, mut safe: ResMut<SafeArea>) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(app) = bevy::android::ANDROID_APP.get() else {
        return;
    };
    let rect = app.content_rect();
    let (w, h) = (
        window.physical_width() as i32,
        window.physical_height() as i32,
    );
    // Before the first layout the content rect can be empty (0,0,0,0): a valid rect has a
    // positive area. Until then (and off a cutout/nav device) the insets stay 0.
    let next = if rect.right > rect.left && rect.bottom > rect.top {
        let scale = window.scale_factor().max(f32::EPSILON);
        let logical = |px: i32| (px.max(0) as f32) / scale;
        SafeArea {
            left: logical(rect.left),
            top: logical(rect.top),
            right: logical(w - rect.right),
            bottom: logical(h - rect.bottom),
        }
    } else {
        SafeArea::default()
    };
    if *safe != next {
        *safe = next;
    }
}

/// Off Android there is nothing to inset: [`SafeArea`] stays at its `0` default.
#[cfg(not(target_os = "android"))]
fn refresh() {}
