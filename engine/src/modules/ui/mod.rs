//! UI for games, the way Unity does it: a **Canvas** holds the UI, and it is either drawn over
//! the screen, drawn by a camera, or placed in the 3D world.
//!
//! ```ignore
//! Object::new("HUD")
//!     .with(Canvas::overlay())                    // or Canvas::camera(), Canvas::world(w, h, ppu)
//!     .child(Object::new("Title").with(Text::new("Darius")))
//!     .child(Object::new("Quality").with(Dropdown::new(["Low", "High"])))
//! ```
//!
//! Every element is a plain Bevy component (`Node`, `Text`, `Button`, `BackgroundColor`,
//! `ImageNode`) or one of the widgets here (`Toggle`, `Slider`, `Dropdown`), so a scene's UI is
//! typed `.rs` data like everything else, and the editor's Scene view shows it through the same
//! code the game runs (a world-space canvas is a UI camera rendering into an image shown on a
//! quad). A root `Node` with no `Canvas` is a screen-space overlay, as before.
//!
//! - `bands`    the header / content / footer rows of a game screen ([`HeaderBand`], ...)
//! - `canvas`   [`Canvas`], its three render modes, the world-space surface, [`UiSettings`]
//! - `rect`     [`RectTransform`] and [`Anchor`]: anchoring, like Unity's RectTransform
//! - `widgets`  [`Toggle`], [`Slider`], [`Dropdown`] and the [`WidgetChanged`] message
//! - `touch`    mobile inputs: [`VirtualJoystick`] (floating analog stick), [`TouchPad`] and
//!               [`TouchButton`], bound to actions by id (see `modules::input`)

mod bands;
mod canvas;
mod rect;
mod touch;
mod widgets;

use bevy::prelude::*;

pub use bands::{
    Band, BandContent, BandSettings, BandToggle, BandToggleLabel, Bands, BandsPlugin, CameraBase,
    ContentBand, FooterBand, HeaderBand,
};
pub use canvas::{Canvas, CanvasCamera, CanvasMode, CanvasSurface, UiSettings};
pub use rect::{Anchor, RectTransform};
pub use touch::{TouchButton, TouchInputSet, TouchPad, TouchSeen, VirtualJoystick};
pub use widgets::{Dropdown, Slider, Toggle, WidgetChanged};

/// Installs canvases and widgets. Added by `launch_project` and by the editor's stage.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::modules::safe_area::SafeAreaPlugin)
            .add_message::<WidgetChanged>()
            .init_resource::<UiSettings>()
            .init_resource::<TouchSeen>()
            .add_systems(
                PreUpdate,
                touch::drive
                    .in_set(TouchInputSet)
                    .after(bevy::input::InputSystems),
            )
            .add_systems(
                Update,
                (touch::build_sticks, touch::tint_zones, touch::show_sticks).chain(),
            )
            .add_systems(
                Update,
                // Before layout (PostUpdate): the anchors become Node fields.
                rect::apply_rect_transforms,
            )
            .add_systems(
                Update,
                (
                    canvas::attach_camera,
                    canvas::build_surface,
                    canvas::order_overlay,
                ),
            )
            .add_systems(
                Update,
                (
                    widgets::build_toggle,
                    widgets::build_slider,
                    widgets::build_dropdown,
                    widgets::press_toggle,
                    widgets::drag_slider,
                    widgets::press_dropdown,
                    widgets::refresh_toggle,
                    widgets::refresh_slider,
                    widgets::refresh_dropdown,
                )
                    .chain(),
            );
    }
}
