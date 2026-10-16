//! The transform gizmos (Move W, Rotate E, Scale R) of a UI element. A UI element has no
//! Transform: it is laid out by its parent, so the gizmo works on the element's plane (the
//! canvas), and what it edits is the layout and the element's `UiTransform`:
//!
//! | Tool   | Handles                               | Writes                                     |
//! |--------|---------------------------------------|--------------------------------------------|
//! | Move   | X and Y arrows, and the XY square     | the offset (`left` / `top`) or the anchors |
//! | Rotate | a ring                                | `UiTransform.rotation`                     |
//! | Scale  | X and Y boxes, and the centre box     | `UiTransform.scale`                        |
//!
//! Like the Rect tool, the drag is previewed on the entity and reported as a [`RectDrag`]; the
//! Properties side turns the final report into one edit of the scene document.

use bevy::gizmos::config::GizmoConfigGroup;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};

use super::camera::{ActiveCamera, EditorCamera, SceneInput, ViewportGizmoSlot};
use super::gizmo::{
    GRAB_PX_MOUSE, GizmoState, distance_ray_point, line_param, ray_plane, size, world_per_pixel,
};
use super::rect_tool::{RectGrab, preview};
use super::viewport_gizmos::ViewportGizmoWriters;
use super::{ActiveTool, Selected, StageObject, ViewOpts};
use crate::editor::protocol::{RectDrag, RectKind, Tool};
use crate::modules::ui::{Canvas, CanvasSurface, RectTransform};

const X: Color = Color::srgb(0.93, 0.27, 0.27);
const Y: Color = Color::srgb(0.38, 0.85, 0.3);
const Z: Color = Color::srgb(0.3, 0.52, 0.98);
const HOVER: Color = Color::srgb(1.0, 0.86, 0.2);

/// A handle of the UI gizmo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiHandle {
    /// Move or scale along the element's X (0) or Y (1).
    Axis(usize),
    /// Move freely in the plane.
    Plane,
    /// Rotate.
    Ring,
    /// Scale uniformly.
    Center,
}

/// A drag in progress.
#[derive(Debug, Clone, Copy)]
pub struct UiGrab {
    handle: UiHandle,
    anchor: Vec3,
    cursor: Vec2,
    center: Vec3,
    axes: [Vec3; 3],
    /// What the element was when the drag started (for the layout preview).
    base: RectGrab,
    /// Its `UiTransform` rotation (degrees) and scale then.
    start_ui: (f32, Vec2),
    seq: u64,
}

/// The handle under `ray` for `tool`, at `at` with gizmo size `s` (`grab` is the least hit
/// distance in world units).
fn hit(tool: Tool, axes: [Vec3; 3], ray: Ray3d, at: Vec3, s: f32, grab: f32) -> Option<UiHandle> {
    let tolerance = (s * 0.09).max(grab);
    if tool == Tool::Scale && distance_ray_point(ray, at) < (s * 0.14).max(grab) {
        return Some(UiHandle::Center);
    }
    let mut best: Option<(f32, UiHandle)> = None;
    let mut consider = |d: f32, h: UiHandle| {
        if best.is_none_or(|(b, _)| d < b) {
            best = Some((d, h));
        }
    };
    match tool {
        Tool::Move | Tool::Scale => {
            for i in 0..2 {
                if let Some(t) = line_param(ray, at, axes[i]) {
                    if (0.0..=s * 1.1).contains(&t) {
                        let d = distance_ray_point(ray, at + axes[i] * t);
                        if d < tolerance {
                            consider(d, UiHandle::Axis(i));
                        }
                    }
                }
            }
            if tool == Tool::Move {
                if let Some(p) = ray_plane(ray, at, axes[2]) {
                    let (a, b) = ((p - at).dot(axes[0]), (p - at).dot(axes[1]));
                    if (s * 0.2..=s * 0.42).contains(&a) && (s * 0.2..=s * 0.42).contains(&b) {
                        consider(tolerance * 0.5, UiHandle::Plane);
                    }
                }
            }
        }
        Tool::Rotate => {
            if let Some(p) = ray_plane(ray, at, axes[2]) {
                let d = ((p - at).length() - s).abs();
                if d < tolerance * 1.4 {
                    consider(d, UiHandle::Ring);
                }
            }
        }
        _ => {}
    }
    best.map(|(_, h)| h)
}

fn draw<Config: GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    tool: Tool,
    axes: [Vec3; 3],
    at: Vec3,
    s: f32,
    hot: Option<UiHandle>,
) {
    let pick = |h: UiHandle, c: Color| if hot == Some(h) { HOVER } else { c };
    match tool {
        Tool::Move => {
            for (i, c) in [X, Y].into_iter().enumerate() {
                gizmos
                    .arrow(at, at + axes[i] * s, pick(UiHandle::Axis(i), c))
                    .with_tip_length(s * 0.22);
            }
            let (a, b) = (s * 0.2, s * 0.42);
            let q = [
                at + axes[0] * a + axes[1] * a,
                at + axes[0] * b + axes[1] * a,
                at + axes[0] * b + axes[1] * b,
                at + axes[0] * a + axes[1] * b,
            ];
            for k in 0..4 {
                gizmos.line(q[k], q[(k + 1) % 4], pick(UiHandle::Plane, Z));
            }
        }
        Tool::Rotate => {
            let iso = Isometry3d::new(at, Quat::from_rotation_arc(Vec3::Z, axes[2]));
            gizmos
                .circle(iso, s, pick(UiHandle::Ring, Z))
                .resolution(64);
        }
        Tool::Scale => {
            let turn = Quat::from_mat3(&Mat3::from_cols(axes[0], axes[1], axes[2]));
            for (i, c) in [X, Y].into_iter().enumerate() {
                let end = at + axes[i] * s;
                gizmos.line(at, end, pick(UiHandle::Axis(i), c));
                gizmos.cube(
                    Transform::from_translation(end)
                        .with_rotation(turn)
                        .with_scale(Vec3::splat(s * 0.12)),
                    pick(UiHandle::Axis(i), c),
                );
            }
            gizmos.cube(
                Transform::from_translation(at)
                    .with_rotation(turn)
                    .with_scale(Vec3::splat(s * 0.16)),
                pick(UiHandle::Center, Color::srgb(0.85, 0.85, 0.85)),
            );
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ui_gizmo(
    mut state: ResMut<GizmoState>,
    tool: Res<ActiveTool>,
    selected: Res<Selected>,
    opts: Res<ViewOpts>,
    input: Res<SceneInput>,
    (buttons, keys): (Res<ButtonInput<MouseButton>>, Res<ButtonInput<KeyCode>>),
    active: Single<(&Camera, &GlobalTransform, &EditorCamera, &Projection), With<ActiveCamera>>,
    views: Query<(&GlobalTransform, &EditorCamera, &ViewportGizmoSlot)>,
    objects: Query<(Entity, &StageObject)>,
    (parents, surfaces, nodes): (
        Query<&ChildOf>,
        Query<(&CanvasSurface, &GlobalTransform)>,
        Query<(&ComputedNode, &UiGlobalTransform)>,
    ),
    canvases: Query<(), With<Canvas>>,
    mut rects: Query<&mut RectTransform>,
    mut flex: Query<&mut Node, With<StageObject>>,
    mut transforms: Query<&mut Transform, With<StageObject>>,
    mut xforms: Query<&mut UiTransform, With<StageObject>>,
    mut vg: ViewportGizmoWriters,
) {
    let reset = |state: &mut GizmoState| {
        state.ui_hover = None;
        state.ui_drag = None;
    };
    let wanted = matches!(tool.0, Tool::Move | Tool::Rotate | Tool::Scale);
    let target = selected
        .0
        .filter(|_| wanted && opts.0.transform_gizmo)
        .and_then(|id| objects.iter().find(|(_, o)| o.id == id))
        .filter(|(e, _)| flex.contains(*e) && !canvases.contains(*e));
    let Some((entity, object)) = target else {
        reset(&mut state);
        return;
    };
    let Some((corners, _)) = super::ui::world_rect(entity, &parents, &surfaces, &nodes) else {
        reset(&mut state);
        return;
    };
    let (camera, camera_at, editor_camera, projection) = *active;
    let at = (corners[0] + corners[2]) / 2.0;
    let u = (corners[1] - corners[0]).normalize_or_zero();
    let v = (corners[0] - corners[3]).normalize_or_zero();
    let axes = [u, v, u.cross(v)];
    let flat_zoom = editor_camera.flat.then_some(editor_camera.zoom);
    let s = size(camera_at, at, flat_zoom);
    let ppu = super::ui::surface_ppu(entity, &parents, &surfaces).unwrap_or(1.0);
    let ray = input
        .window_cursor
        .and_then(|c| camera.viewport_to_world(camera_at, c).ok());
    let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);

    if state.ui_drag.is_none() {
        state.ui_hover = if input.over_scene && input.dragging.is_none() {
            let grab = GRAB_PX_MOUSE
                * world_per_pixel(camera, projection, camera_at.translation(), at, flat_zoom);
            ray.and_then(|r| hit(tool.0, axes, r, at, s, grab))
        } else {
            None
        };
        if let (Some(handle), Some(ray), Some(cursor), true, false) = (
            state.ui_hover,
            ray,
            input.window_cursor,
            buttons.just_pressed(MouseButton::Left),
            alt,
        ) {
            let anchor = match handle {
                UiHandle::Axis(i) => line_param(ray, at, axes[i]).map(|t| at + axes[i] * t),
                UiHandle::Plane | UiHandle::Ring => ray_plane(ray, at, axes[2]),
                UiHandle::Center => Some(at),
            };
            if let Some(anchor) = anchor {
                let rect = rects.get(entity).ok().map(|r| *r);
                let node = flex.get(entity).ok();
                let offset = parents
                    .get(entity)
                    .ok()
                    .and_then(|p| super::ui::world_rect(p.parent(), &parents, &surfaces, &nodes))
                    .map(|(pc, _)| {
                        let d = corners[0] - pc[0];
                        Vec2::new(d.dot(u), -d.dot(v)) * ppu
                    })
                    .unwrap_or(Vec2::ZERO);
                state.rect_seq += 1;
                let base = RectGrab {
                    handle: [0, 0],
                    entity,
                    id: object.id,
                    anchor,
                    start_transform: Transform::IDENTITY,
                    start_rect: rect,
                    start_node: rect.is_none().then(|| {
                        node.map(|n| [n.left, n.top, n.right, n.bottom, n.width, n.height])
                    })
                    .flatten(),
                    start_size: Vec2::new(
                        corners[0].distance(corners[1]),
                        corners[0].distance(corners[3]),
                    ) * ppu,
                    offset,
                    ppu,
                    seq: state.rect_seq,
                };
                state.ui_drag = Some(UiGrab {
                    handle,
                    anchor,
                    cursor,
                    center: at,
                    axes,
                    base,
                    start_ui: xforms
                        .get(entity)
                        .map(|x| (x.rotation.as_degrees(), x.scale))
                        .unwrap_or((0.0, Vec2::ONE)),
                    seq: state.rect_seq,
                });
            }
        }
    } else if !buttons.pressed(MouseButton::Left) {
        if let Some(drag) = state.ui_drag.take() {
            if let Some(report) = state.rect.as_mut().filter(|r| r.seq == drag.seq) {
                report.ended = true;
            }
        }
    }

    if let (Some(drag), Some(ray), Some(cursor)) = (state.ui_drag, ray, input.window_cursor) {
        let mut report = RectDrag {
            id: drag.base.id,
            handle: [0, 0],
            delta: [0.0, 0.0],
            seq: drag.seq,
            pixels_per_unit: drag.base.ppu,
            size: drag.base.start_size.to_array(),
            ended: false,
            ui: true,
            kind: RectKind::Resize,
            scale: [1.0, 1.0],
            rotation: 0.0,
            offset: drag.base.offset.to_array(),
        };
        let start_at = drag.center;
        match (tool.0, drag.handle) {
            (Tool::Move, UiHandle::Axis(i)) => {
                if let Some(t) = line_param(ray, start_at, drag.axes[i]) {
                    let from = (drag.anchor - start_at).dot(drag.axes[i]);
                    let d = t - from;
                    report.delta = if i == 0 { [d, 0.0] } else { [0.0, d] };
                }
            }
            (Tool::Move, UiHandle::Plane) => {
                if let Some(p) = ray_plane(ray, start_at, drag.axes[2]) {
                    let m = p - drag.anchor;
                    report.delta = [m.dot(drag.axes[0]), m.dot(drag.axes[1])];
                }
            }
            (Tool::Scale, UiHandle::Axis(i)) => {
                if let Some(t) = line_param(ray, start_at, drag.axes[i]) {
                    let from = (drag.anchor - start_at).dot(drag.axes[i]);
                    if from.abs() > 1e-4 {
                        report.scale[i] = (t / from).clamp(0.05, 20.0);
                    }
                }
                report.kind = RectKind::Scale;
            }
            (Tool::Scale, UiHandle::Center) => {
                let f = (1.0 + (drag.cursor.y - cursor.y) * 0.01).clamp(0.05, 20.0);
                report.scale = [f, f];
                report.kind = RectKind::Scale;
            }
            (Tool::Rotate, UiHandle::Ring) => {
                if let Some(p) = ray_plane(ray, start_at, drag.axes[2]) {
                    let (a, b) = (drag.anchor - start_at, p - start_at);
                    let (ax, ay) = (a.dot(drag.axes[0]), a.dot(drag.axes[1]));
                    let (bx, by) = (b.dot(drag.axes[0]), b.dot(drag.axes[1]));
                    // Counter-clockwise on the canvas is negative in bevy_ui's y-down space.
                    let angle = (ax * by - ay * bx).atan2(ax * bx + ay * by).to_degrees();
                    report.rotation = -angle;
                }
                report.kind = RectKind::Rotate;
            }
            _ => {}
        }
        match report.kind {
            RectKind::Resize => {
                let delta = Vec2::from_array(report.delta);
                preview(&drag.base, delta, true, &mut rects, &mut flex, &mut transforms);
            }
            _ => {
                if let Ok(mut x) = xforms.get_mut(drag.base.entity) {
                    x.scale = drag.start_ui.1 * Vec2::from_array(report.scale);
                    x.rotation = Rot2::degrees(drag.start_ui.0 + report.rotation);
                }
            }
        }
        state.rect = Some(report);
    }

    let hot = state.ui_drag.map(|d| d.handle).or(state.ui_hover);
    for (view_at, view_cam, slot) in &views {
        let view_zoom = view_cam.flat.then_some(view_cam.zoom);
        let view_s = size(view_at, at, view_zoom);
        match slot.0.min(3) {
            0 => draw(&mut vg.g0, tool.0, axes, at, view_s, hot),
            1 => draw(&mut vg.g1, tool.0, axes, at, view_s, hot),
            2 => draw(&mut vg.g2, tool.0, axes, at, view_s, hot),
            _ => draw(&mut vg.g3, tool.0, axes, at, view_s, hot),
        }
    }
}
