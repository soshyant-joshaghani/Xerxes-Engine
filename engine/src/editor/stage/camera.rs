//! The editor-default camera: the editor's own, never in the Hierarchy and never saved.
//! It renders into the Viewport area (solved from the workspace, `editor::layout`) and moves
//! like Unreal Engine's viewport camera:
//!
//! | Input                 | Action                                   |
//! |-----------------------|------------------------------------------|
//! | RMB + mouse           | look around                              |
//! | RMB + W/A/S/D, Q/E    | fly; RMB + wheel sets the fly speed      |
//! | MMB + mouse           | pan                                      |
//! | wheel                 | dolly                                    |
//! | Alt + LMB / RMB / MMB | orbit the pivot / dolly to it / pan      |
//! | F                     | frame the selection                      |
//!
//! A drag only starts with the pointer over the Scene view (and no window over it), so the
//! panels keep their input on every platform. Touch gestures are in `touch`.

use bevy::camera::Viewport;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use std::collections::HashMap;

use crate::editor::layout::{self, AreaId, EditorKind, Workspace, edge, header};
use crate::editor::protocol::ViewAction;
use crate::editor::theme::{Insets, Layout};

/// The editor camera: an orbit pivot, the distance to it, and the view direction. For a 2D
/// scene it is `flat`: orthographic, looking down -Z at the XY plane, panned and zoomed.
#[derive(Component, Debug, Clone, Copy)]
pub struct EditorCamera {
    pub flat: bool,
    /// World units per pixel in the flat (2D) view.
    pub zoom: f32,
    pub pivot: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    /// Fly speed in units per second.
    pub speed: f32,
}

impl Default for EditorCamera {
    fn default() -> Self {
        // A diagonal view from above, like a new Unity scene (and away from where gameplay
        // cameras usually sit, straight behind the origin).
        Self {
            flat: false,
            zoom: 1.0,
            pivot: Vec3::ZERO,
            distance: 24.0,
            yaw: 0.75,
            pitch: -0.55,
            speed: 8.0,
        }
    }
}

impl EditorCamera {
    pub fn rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }

    pub fn position(&self) -> Vec3 {
        self.pivot - self.rotation() * Vec3::NEG_Z * self.distance
    }

    pub fn transform(&self) -> Transform {
        if self.flat {
            return Transform::from_xyz(self.pivot.x, self.pivot.y, 1000.0);
        }
        Transform::from_translation(self.position()).with_rotation(self.rotation())
    }

    pub fn projection(&self) -> Projection {
        if self.flat {
            Projection::Orthographic(OrthographicProjection {
                scale: self.zoom,
                ..OrthographicProjection::default_2d()
            })
        } else {
            Projection::Perspective(PerspectiveProjection::default())
        }
    }

    /// Switches between the 3D (perspective) and 2D (flat) views for a scene.
    pub fn set_flat(&mut self, flat: bool) {
        if self.flat != flat {
            *self = Self {
                flat,
                ..Self::default()
            };
        }
    }
}

/// Where the pointer is and what may use it this frame.
#[derive(Resource, Debug, Default)]
pub struct SceneInput {
    /// The pointer is over the Scene view and no window covers it.
    pub over_scene: bool,
    /// A window (Project Settings, a menu) covers the Scene view.
    pub modal: bool,
    /// The mouse button whose drag the camera owns, if any.
    pub dragging: Option<MouseButton>,
    /// Cursor position in the Scene view (logical, from its top-left).
    pub cursor: Option<Vec2>,
    /// Cursor position in the window (logical): what `Camera::viewport_to_world` takes (it
    /// subtracts the viewport's origin itself).
    pub window_cursor: Option<Vec2>,
}

/// The editor's frame this frame (see `theme::Layout`), published to the UI.
#[derive(Resource, Debug, Default)]
pub struct ActiveLayout(pub Layout);

/// The workspace the UI shows (it sends every change). Starts as the default layout, which is
/// also what the UI starts with.
#[derive(Resource, Debug)]
pub struct ActiveWorkspace(pub Workspace);

impl Default for ActiveWorkspace {
    fn default() -> Self {
        Self(layout::layout())
    }
}

/// What the system draws over the window, in logical pixels: Android reports the area free of
/// its bars (`content_rect`); elsewhere the whole window is ours.
fn safe_area(window: &Window) -> Insets {
    #[cfg(target_os = "android")]
    if let Some(app) = bevy::android::ANDROID_APP.get() {
        let rect = app.content_rect();
        let scale = window.scale_factor();
        let (w, h) = (
            window.physical_width() as i32,
            window.physical_height() as i32,
        );
        if rect.right > rect.left && rect.bottom > rect.top {
            let px = |v: i32| (v.max(0) as f32) / scale;
            return Insets {
                left: px(rect.left),
                top: px(rect.top),
                right: px(w - rect.right),
                bottom: px(h - rect.bottom),
            };
        }
    }
    let _ = window;
    Insets::default()
}

/// An editor camera belongs to one Viewport area.
#[derive(Component, Debug, Clone, Copy)]
pub struct ViewportArea(pub AreaId);

/// On the camera of the Viewport the pointer works in: the one under it, or the last one it
/// was over. Navigation, picking and the gizmos use this camera.
#[derive(Component)]
pub struct ActiveCamera;

#[derive(Resource, Default)]
pub struct ViewState {
    /// The Viewport the pointer works in.
    pub active: Option<AreaId>,
    /// The scene is 2D (flat cameras) or 3D.
    pub flat: bool,
    /// The layout the cameras on screen belong to.
    owner: String,
    /// Poses of cameras that went away (another editor was picked in their area, or another
    /// layout is showing), so they come back as they were. Kept for the session.
    poses: HashMap<(String, AreaId), EditorCamera>,
}

/// Applies a View menu command to a camera.
pub fn view_action(
    cam: &mut EditorCamera,
    action: ViewAction,
    selected: &super::Selected,
    objects: &Query<(&GlobalTransform, &super::StageObject)>,
) {
    match action {
        ViewAction::Frame => frame(cam, selected, objects),
        ViewAction::Reset => {
            let flat = cam.flat;
            *cam = EditorCamera { flat, ..default() };
        }
        // The axis views make no sense for a flat (2D) camera.
        ViewAction::Top | ViewAction::Front | ViewAction::Right if cam.flat => {}
        // Not exactly vertical: the camera's pitch is limited, so it never flips over.
        ViewAction::Top => {
            cam.yaw = 0.0;
            cam.pitch = -1.54;
        }
        ViewAction::Front => {
            cam.yaw = 0.0;
            cam.pitch = 0.0;
        }
        ViewAction::Right => {
            cam.yaw = std::f32::consts::FRAC_PI_2;
            cam.pitch = 0.0;
        }
    }
}

/// Keeps one camera per Viewport area, each with its viewport on its area, and tracks the
/// pointer. With no Viewport area on screen (it was switched to another editor, or another
/// area is maximized) there is no camera and nothing is over the Scene view.
#[allow(clippy::too_many_arguments)]
pub fn fit_viewport(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cameras: Query<(
        Entity,
        &ViewportArea,
        &mut Camera,
        &EditorCamera,
        Has<ActiveCamera>,
    )>,
    mut input: ResMut<SceneInput>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch: Res<super::touch::TouchPointer>,
    mut layout: ResMut<ActiveLayout>,
    workspace: Res<ActiveWorkspace>,
    mut views: ResMut<ViewState>,
) {
    let size = Vec2::new(window.width(), window.height());
    layout.0 = Layout::for_window(size, safe_area(&window));
    let content = layout.0.content();
    // The areas are solved relative to the content's corner; the window adds it back.
    let local = layout::Rect::new(0.0, 0.0, content.w, content.h);
    let solved = layout::solve(&workspace.0, local);
    let viewports: Vec<(AreaId, Rect)> = solved
        .areas
        .iter()
        .filter(|p| p.area.editor == EditorKind::Viewport)
        .map(|p| {
            let b = p.rect.below(header()).moved(content.x, content.y);
            (
                p.area.id,
                Rect::from_corners(Vec2::new(b.x, b.y), Vec2::new(b.right(), b.bottom())),
            )
        })
        .collect();

    // The mouse, or the finger acting as the pointer (touch screens; a mouse on Android).
    let cursor = window.cursor_position().or(touch.position);
    input.window_cursor = cursor;

    // Another layout is showing: its Viewports are other cameras. Keep the old poses, start
    // the new cameras next frame.
    if views.owner != workspace.0.name {
        let owner = std::mem::replace(&mut views.owner, workspace.0.name.clone());
        for (entity, area, _, cam, _) in &cameras {
            views.poses.insert((owner.clone(), area.0), *cam);
            commands.entity(entity).despawn();
        }
        views.active = None;
        input.cursor = None;
        input.over_scene = false;
        return;
    }
    let owner = views.owner.clone();
    for (entity, area, _, cam, _) in &cameras {
        if !viewports.iter().any(|(id, _)| *id == area.0) {
            views.poses.insert((owner.clone(), area.0), *cam);
            commands.entity(entity).despawn();
        }
    }

    // The Viewport under the pointer takes over, unless a button is held (a drag or a gizmo
    // belongs to the Viewport it started in).
    let hovered = if input.modal {
        None
    } else {
        cursor.and_then(|c| {
            viewports
                .iter()
                .find(|(_, r)| r.inflate(-edge() / 2.0).contains(c))
                .map(|(id, _)| *id)
        })
    };
    let busy = input.dragging.is_some() || buttons.get_pressed().next().is_some();
    let current = views
        .active
        .filter(|a| viewports.iter().any(|(id, _)| id == a));
    let active = if busy {
        current.or(hovered)
    } else {
        hovered.or(current)
    }
    .or(viewports.first().map(|(id, _)| *id));
    views.active = active;

    let scale = window.scale_factor();
    let viewport_of = |rect: Rect| Viewport {
        physical_position: (rect.min * scale).as_uvec2(),
        physical_size: (rect.size() * scale).max(Vec2::ONE).as_uvec2(),
        ..default()
    };
    let same = |a: &Viewport, b: &Viewport| {
        (a.physical_position, a.physical_size) == (b.physical_position, b.physical_size)
    };

    for (id, rect) in &viewports {
        if cameras.iter().any(|(_, a, ..)| a.0 == *id) {
            continue;
        }
        let pose = views
            .poses
            .remove(&(owner.clone(), *id))
            .unwrap_or_else(|| {
                let mut pose = EditorCamera::default();
                pose.set_flat(views.flat);
                pose
            });
        let mut entity = commands.spawn((
            Name::new("Editor Camera"),
            Camera3d::default(),
            Camera {
                order: id.0 as isize,
                viewport: Some(viewport_of(*rect)),
                ..default()
            },
            pose.transform(),
            pose.projection(),
            pose,
            ViewportArea(*id),
        ));
        if Some(*id) == active {
            entity.insert(ActiveCamera);
        }
    }
    for (entity, area, mut camera, _, is_active) in &mut cameras {
        let Some((_, rect)) = viewports.iter().find(|(id, _)| *id == area.0) else {
            continue;
        };
        let wanted = viewport_of(*rect);
        if !camera.viewport.as_ref().is_some_and(|v| same(v, &wanted)) {
            camera.viewport = Some(wanted);
        }
        camera.order = area.0.0 as isize;
        let should = Some(area.0) == active;
        if should && !is_active {
            commands.entity(entity).insert(ActiveCamera);
        } else if !should && is_active {
            commands.entity(entity).remove::<ActiveCamera>();
        }
    }

    let active_rect = active
        .and_then(|a| viewports.iter().find(|(id, _)| *id == a))
        .map(|(_, r)| *r);
    input.cursor = cursor.zip(active_rect).map(|(c, r)| c - r.min);
    // The edge grab zones overlap a viewport's border: the UI owns them.
    input.over_scene = hovered.is_some() && hovered == active;
}

pub fn control_camera(
    time: Res<Time>,
    mut input: ResMut<SceneInput>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    selected: Res<super::Selected>,
    objects: Query<(&GlobalTransform, &super::StageObject)>,
    mut camera: Single<(&mut Transform, &mut EditorCamera, &mut Projection), With<ActiveCamera>>,
) {
    let (transform, cam, projection) = &mut *camera;
    let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);

    // A drag starts over the Scene view only, and keeps going until its button is released.
    if let Some(button) = input.dragging {
        if !buttons.pressed(button) {
            input.dragging = None;
        }
    }
    if input.dragging.is_none() && input.over_scene {
        for button in [MouseButton::Right, MouseButton::Middle, MouseButton::Left] {
            // A plain left click selects (and drags gizmos); only Alt + LMB moves the camera.
            if buttons.just_pressed(button) && (button != MouseButton::Left || alt) {
                input.dragging = Some(button);
            }
        }
    }

    let delta = motion.delta;
    let wheel = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / 40.0,
    };
    let look = 0.004;
    let pan = cam.distance.max(1.0) * 0.0015;

    if cam.flat {
        // 2D: RMB or MMB drag pans, the wheel zooms, F frames.
        if matches!(
            input.dragging,
            Some(MouseButton::Right | MouseButton::Middle)
        ) {
            let zoom = cam.zoom;
            cam.pivot += Vec3::new(-delta.x, delta.y, 0.0) * zoom;
        }
        if input.over_scene && wheel != 0.0 && input.dragging.is_none() {
            cam.zoom = (cam.zoom * 0.88f32.powf(wheel)).clamp(0.01, 100.0);
        }
        if input.over_scene && keys.just_pressed(KeyCode::KeyF) {
            frame(cam, &selected, &objects);
        }
        **transform = cam.transform();
        **projection = cam.projection();
        return;
    }

    match (input.dragging, alt) {
        // Alt + LMB: orbit the pivot (Maya tumble).
        (Some(MouseButton::Left), true) => {
            cam.yaw -= delta.x * look;
            cam.pitch = (cam.pitch - delta.y * look).clamp(-1.54, 1.54);
        }
        // Alt + RMB: dolly toward / away from the pivot.
        (Some(MouseButton::Right), true) => {
            cam.distance = (cam.distance * (1.0 + (delta.y - delta.x) * 0.005)).clamp(0.2, 2000.0);
        }
        // MMB (with or without Alt): pan.
        (Some(MouseButton::Middle), _) => {
            let right = cam.rotation() * Vec3::X;
            let up = cam.rotation() * Vec3::Y;
            cam.pivot += (-right * delta.x + up * delta.y) * pan;
        }
        // RMB: look around in place, fly with WASD/QE, wheel sets the speed.
        (Some(MouseButton::Right), false) => {
            let position = cam.position();
            cam.yaw -= delta.x * look;
            cam.pitch = (cam.pitch - delta.y * look).clamp(-1.54, 1.54);
            let forward = cam.rotation() * Vec3::NEG_Z;
            let right = cam.rotation() * Vec3::X;
            let axis = |neg: KeyCode, pos: KeyCode| {
                keys.pressed(pos) as i8 as f32 - keys.pressed(neg) as i8 as f32
            };
            let fly = forward * axis(KeyCode::KeyS, KeyCode::KeyW)
                + right * axis(KeyCode::KeyA, KeyCode::KeyD)
                + Vec3::Y * axis(KeyCode::KeyQ, KeyCode::KeyE);
            if wheel != 0.0 {
                cam.speed = (cam.speed * 1.2f32.powf(wheel)).clamp(0.5, 500.0);
            }
            // Looking turns around the camera, not the pivot: keep the position.
            cam.pivot = position + forward * cam.distance + fly * cam.speed * time.delta_secs();
        }
        _ => {
            // Wheel: dolly forward, moving the pivot with the camera when it gets close.
            if input.over_scene && wheel != 0.0 {
                let step = cam.distance.max(2.0) * 0.12 * wheel;
                if cam.distance - step > 0.5 {
                    cam.distance -= step;
                } else {
                    let forward = cam.rotation() * Vec3::NEG_Z;
                    cam.pivot += forward * step;
                }
            }
            if input.over_scene && keys.just_pressed(KeyCode::KeyF) {
                frame(cam, &selected, &objects);
            }
        }
    }
    **transform = cam.transform();
}

/// Puts the selected object in the middle of the view.
pub fn frame(
    cam: &mut EditorCamera,
    selected: &super::Selected,
    objects: &Query<(&GlobalTransform, &super::StageObject)>,
) {
    let Some(id) = selected.0 else { return };
    if let Some((at, _)) = objects.iter().find(|(_, o)| o.id == id) {
        cam.pivot = at.translation();
        cam.distance = 6.0;
        if cam.flat {
            cam.pivot.z = 0.0;
        }
    }
}
