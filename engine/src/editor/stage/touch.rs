//! Touch in the Scene view: phones and tablets on every platform (Android, a tablet's
//! browser). On Android a mouse also arrives as touch (winit reports no buttons there).
//!
//! | Touch                     | Action                                          |
//! |---------------------------|-------------------------------------------------|
//! | tap                       | select (like a click)                           |
//! | drag a gizmo handle       | move / rotate / scale                           |
//! | one-finger drag elsewhere | orbit the pivot (3D), pan (2D)                  |
//! | two fingers               | pan; pinch to zoom                              |
//!
//! The first finger is the pointer: it sets the cursor and acts as the left mouse button,
//! so picking and gizmos work through their usual mouse path.

use bevy::input::touch::Touches;
use bevy::prelude::*;

use super::camera::{ActiveCamera, EditorCamera, SceneInput};
use super::gizmo::GizmoState;

/// The first touch on the screen turns touch sizes on when the choice is Auto (and they stay: a
/// device with a touch screen and a mouse is used with both).
pub fn touch_mode(touches: Res<Touches>) {
    if crate::editor::layout::touch_auto()
        && !crate::editor::layout::touch()
        && touches.iter().next().is_some()
    {
        crate::editor::layout::set_touch(true);
    }
}

/// The finger acting as the pointer, and where it is.
#[derive(Resource, Debug, Default)]
pub struct TouchPointer {
    /// The first finger down (until it lifts).
    primary: Option<u64>,
    /// Its position in the window (logical); kept after it lifts, for the release frame.
    pub position: Option<Vec2>,
    /// The one-finger drag started over the Scene view (else it belongs to the panels).
    on_scene: bool,
}

impl TouchPointer {
    /// A finger is down (handles are easier to hit then).
    pub fn touching(&self) -> bool {
        self.primary.is_some()
    }
}

/// Runs right after Bevy's input: the first finger becomes the cursor and the left button.
pub fn touch_as_pointer(
    touches: Res<Touches>,
    mut pointer: ResMut<TouchPointer>,
    mut buttons: ResMut<ButtonInput<MouseButton>>,
) {
    if pointer.primary.is_none() {
        if let Some(touch) = touches.iter_just_pressed().next() {
            pointer.primary = Some(touch.id());
            pointer.position = Some(touch.position());
            buttons.press(MouseButton::Left);
        }
    }
    let Some(id) = pointer.primary else { return };
    if let Some(touch) = touches.get_pressed(id) {
        pointer.position = Some(touch.position());
        return;
    }
    let lifted = touches
        .iter_just_released()
        .chain(touches.iter_just_canceled())
        .find(|t| t.id() == id);
    if let Some(touch) = lifted {
        pointer.position = Some(touch.position());
    }
    pointer.primary = None;
    buttons.release(MouseButton::Left);
}

/// One finger orbits (pans in 2D) unless it is dragging a gizmo; two fingers pan and pinch.
pub fn touch_camera(
    touches: Res<Touches>,
    mut pointer: ResMut<TouchPointer>,
    input: Res<SceneInput>,
    gizmo: Res<GizmoState>,
    mut camera: Single<(&mut Transform, &mut EditorCamera, &mut Projection), With<ActiveCamera>>,
) {
    if touches.iter_just_pressed().next().is_some() && touches.iter().count() == 1 {
        pointer.on_scene = input.over_scene;
    }
    if !pointer.on_scene || input.modal {
        return;
    }
    let (transform, cam, projection) = &mut *camera;
    let fingers: Vec<_> = touches.iter().collect();
    match fingers.as_slice() {
        // A press on a handle starts a gizmo drag that same frame: then the finger is the gizmo's.
        [one] if !gizmo.dragging() => {
            let delta = one.delta();
            if cam.flat {
                let zoom = cam.zoom;
                cam.pivot += Vec3::new(-delta.x, delta.y, 0.0) * zoom;
            } else {
                let look = 0.006;
                cam.yaw -= delta.x * look;
                cam.pitch = (cam.pitch - delta.y * look).clamp(-1.54, 1.54);
            }
        }
        [a, b, ..] => {
            let delta = (a.delta() + b.delta()) * 0.5;
            let now = a.position().distance(b.position());
            let before = (a.position() - a.delta()).distance(b.position() - b.delta());
            let pinch = if now > 1.0 && before > 1.0 {
                before / now
            } else {
                1.0
            };
            if cam.flat {
                let zoom = cam.zoom;
                cam.pivot += Vec3::new(-delta.x, delta.y, 0.0) * zoom;
                cam.zoom = (cam.zoom * pinch).clamp(0.01, 100.0);
            } else {
                let pan = cam.distance.max(1.0) * 0.0015;
                let right = cam.rotation() * Vec3::X;
                let up = cam.rotation() * Vec3::Y;
                cam.pivot += (-right * delta.x + up * delta.y) * pan;
                cam.distance = (cam.distance * pinch).clamp(0.2, 2000.0);
            }
        }
        _ => return,
    }
    **transform = cam.transform();
    **projection = cam.projection();
}
