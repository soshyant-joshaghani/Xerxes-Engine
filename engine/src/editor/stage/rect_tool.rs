//! Unity's Rect tool (T): the selected sprite or UI element gets its rectangle outlined with a
//! handle on every corner and side. Dragging the body moves it, dragging a handle resizes it
//! around the opposite edge, all in the element's own plane, so it works the same in a 2D scene
//! and on a canvas in a 3D one.
//!
//! The drag is previewed live on the entity (a sprite's transform, a UI element's
//! `RectTransform`) and reported to the UI as how far it moved and which handle; the UI turns
//! the final report into an edit of the scene document (`ui_view::drag_rect`, `drag_sprite`),
//! so the whole drag is one history step and the Scene view is not rebuilt every frame.

use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};

use super::camera::{ActiveCamera, EditorCamera, SceneInput, ViewportGizmoSlot};
use super::gizmo::{GizmoState, world_per_pixel};
use super::viewport_gizmos::ViewportGizmoWriters;
use super::{ActiveTool, Selected, StageObject, ViewOpts};
use crate::editor::bridge::ui_view::{NodeEdit, NodeFrame, ViewVal, drag_node};
use crate::editor::protocol::{RectDrag, RectKind, Tool};
use crate::modules::ui::{CanvasSurface, RectTransform};

/// Marks a sprite object of the Scene view (a quad), so the Rect tool can resize it.
#[derive(Component, Debug, Clone, Copy)]
pub struct StageSprite;

/// A handle: the side on each axis (y up), `[0, 0]` the body.
pub type Handle = [i8; 2];

const HANDLES: [Handle; 8] = [
    [-1, 1],
    [1, 1],
    [1, -1],
    [-1, -1],
    [0, 1],
    [1, 0],
    [0, -1],
    [-1, 0],
];

const OUTLINE: Color = Color::srgb(0.35, 0.78, 1.0);
const HOVER: Color = Color::srgb(1.0, 0.86, 0.2);
/// A handle is grabbed within this many logical pixels.
const GRAB_PX: f32 = 9.0;

/// A drag in progress.
#[derive(Debug, Clone, Copy)]
pub struct RectGrab {
    pub(super) handle: Handle,
    pub(super) entity: Entity,
    pub(super) id: u32,
    /// Where it started, on the element's plane.
    pub(super) anchor: Vec3,
    /// The element when it started.
    pub(super) start_transform: Transform,
    pub(super) start_rect: Option<RectTransform>,
    /// A flexbox element (no `RectTransform`): its `Node`'s left, top, right, bottom, width and
    /// height when the drag started.
    pub(super) start_node: Option<[Val; 6]>,
    pub(super) start_size: Vec2,
    /// Top-left corner relative to the parent's, canvas pixels, y down.
    pub(super) offset: Vec2,
    pub(super) ppu: f32,
    pub(super) seq: u64,
}

/// The point of `corners` ([TL, TR, BR, BL]) a handle sits at.
fn handle_point(corners: &[Vec3; 4], handle: Handle) -> Vec3 {
    let (tl, tr, bl) = (corners[0], corners[1], corners[3]);
    let fx = (handle[0] as f32 + 1.0) / 2.0;
    let fy = (1.0 - handle[1] as f32) / 2.0;
    tl + (tr - tl) * fx + (bl - tl) * fy
}

/// Whether `p` is inside the screen-space quad `q` (in order around it).
fn inside(q: &[Vec2; 4], p: Vec2) -> bool {
    let mut sign = 0.0f32;
    for i in 0..4 {
        let (a, b) = (q[i], q[(i + 1) % 4]);
        let cross = (b - a).perp_dot(p - a);
        if cross != 0.0 {
            if sign != 0.0 && cross.signum() != sign {
                return false;
            }
            sign = cross.signum();
        }
    }
    true
}

#[allow(clippy::too_many_arguments)]
pub fn rect_tool(
    mut state: ResMut<GizmoState>,
    tool: Res<ActiveTool>,
    selected: Res<Selected>,
    opts: Res<ViewOpts>,
    input: Res<SceneInput>,
    (buttons, keys): (Res<ButtonInput<MouseButton>>, Res<ButtonInput<KeyCode>>),
    active: Single<(&Camera, &GlobalTransform), With<ActiveCamera>>,
    views: Query<(
        &Camera,
        &GlobalTransform,
        &EditorCamera,
        &Projection,
        &ViewportGizmoSlot,
    )>,
    objects: Query<(
        Entity,
        &StageObject,
        &GlobalTransform,
        Has<StageSprite>,
        Option<&Aabb>,
    )>,
    (parents, surfaces, nodes): (
        Query<&ChildOf>,
        Query<(&CanvasSurface, &GlobalTransform)>,
        Query<(&ComputedNode, &UiGlobalTransform)>,
    ),
    mut rects: Query<&mut RectTransform>,
    mut flex: Query<&mut Node, With<StageObject>>,
    mut transforms: Query<&mut Transform, With<StageObject>>,
    mut gizmos: Gizmos,
    mut vg: ViewportGizmoWriters,
) {
    let (camera, camera_at) = *active;
    let active = tool.0 == Tool::Rect && opts.0.transform_gizmo;
    let handles: &[Handle] = &HANDLES;
    let target = selected.0.filter(|_| active).and_then(|id| {
        objects
            .iter()
            .find(|(_, o, ..)| o.id == id)
            .map(|(e, o, gt, sprite, aabb)| (e, o.id, *gt, sprite, aabb.copied()))
    });
    let Some((entity, id, world, is_sprite, aabb)) = target else {
        state.rect_hover = None;
        state.rect_drag = None;
        return;
    };

    // The element's corners (top-left, top-right, bottom-right, bottom-left), its pixels per
    // world unit and its size.
    let ui_rect = super::ui::world_rect(entity, &parents, &surfaces, &nodes);
    let (corners, ppu, size, ui) = match (ui_rect, is_sprite, aabb) {
        (Some((c, _)), _, _) if rects.contains(entity) || flex.contains(entity) => {
            let ppu = super::ui::surface_ppu(entity, &parents, &surfaces).unwrap_or(1.0);
            let size = Vec2::new(c[0].distance(c[1]), c[0].distance(c[3])) * ppu;
            (c, ppu, size, true)
        }
        (_, true, Some(aabb)) => {
            let h = Vec3::from(aabb.half_extents);
            let m = world.affine();
            let corner =
                |x: f32, y: f32| m.transform_point3(Vec3::from(aabb.center) + Vec3::new(x, y, 0.0));
            let c = [
                corner(-h.x, h.y),
                corner(h.x, h.y),
                corner(h.x, -h.y),
                corner(-h.x, -h.y),
            ];
            let size = Vec2::new(c[0].distance(c[1]), c[0].distance(c[3]));
            (c, 1.0, size, false)
        }
        _ => {
            state.rect_hover = None;
            state.rect_drag = None;
            return;
        }
    };
    let at = world.translation();
    let u = (corners[1] - corners[0]).normalize_or_zero();
    let v = (corners[0] - corners[3]).normalize_or_zero();
    let normal = u.cross(v);
    let ray = input
        .window_cursor
        .and_then(|c| camera.viewport_to_world(camera_at, c).ok());
    let plane_point = |ray: Ray3d| {
        let denom = ray.direction.dot(normal);
        (denom.abs() > 1e-6).then(|| {
            let t = (corners[0] - ray.origin).dot(normal) / denom;
            ray.origin + *ray.direction * t
        })
    };

    let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    if state.rect_drag.is_none() {
        // Hover: the nearest handle within a few pixels, else the body.
        state.rect_hover = if input.over_scene && input.dragging.is_none() {
            // `world_to_viewport` answers in window coordinates, like the window cursor.
            input.window_cursor.and_then(|cursor| {
                let screen = |p: Vec3| camera.world_to_viewport(camera_at, p).ok();
                let near = handles
                    .iter()
                    .filter_map(|h| {
                        screen(handle_point(&corners, *h)).map(|s| (s.distance(cursor), *h))
                    })
                    .filter(|(d, _)| *d <= GRAB_PX)
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                    .map(|(_, h)| h);
                near.or_else(|| {
                    let quad = [
                        screen(corners[0])?,
                        screen(corners[1])?,
                        screen(corners[2])?,
                        screen(corners[3])?,
                    ];
                    inside(&quad, cursor).then_some([0, 0])
                })
            })
        } else {
            None
        };
        if let (Some(handle), Some(ray), true, false) = (
            state.rect_hover,
            ray,
            buttons.just_pressed(MouseButton::Left),
            alt,
        ) {
            if let (Some(anchor), Ok(transform)) = (plane_point(ray), transforms.get(entity)) {
                let rect = rects.get(entity).ok().map(|r| *r);
                // Where the element sits inside its parent, for an out-of-flow node.
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
                state.rect_drag = Some(RectGrab {
                    handle,
                    entity,
                    id,
                    anchor,
                    start_transform: *transform,
                    start_rect: rect,
                    start_node: rect
                        .is_none()
                        .then(|| {
                            flex.get(entity)
                                .ok()
                                .map(|n| [n.left, n.top, n.right, n.bottom, n.width, n.height])
                        })
                        .flatten(),
                    offset,
                    start_size: size,
                    ppu,
                    seq: state.rect_seq,
                });
            }
        }
    } else if !buttons.pressed(MouseButton::Left) {
        // Released: the last report stays, marked as the final one.
        if let Some(drag) = state.rect_drag.take() {
            if let Some(report) = state.rect.as_mut().filter(|r| r.seq == drag.seq) {
                report.ended = true;
            }
        }
    }

    if let (Some(drag), Some(ray)) = (state.rect_drag, ray) {
        if let Some(p) = plane_point(ray) {
            let moved = p - drag.anchor;
            let delta = Vec2::new(moved.dot(u), moved.dot(v));
            preview(&drag, delta, ui, &mut rects, &mut flex, &mut transforms);
            state.rect = Some(RectDrag {
                kind: RectKind::Resize,
                scale: [1.0, 1.0],
                rotation: 0.0,
                id: drag.id,
                handle: drag.handle,
                delta: delta.to_array(),
                seq: drag.seq,
                pixels_per_unit: drag.ppu,
                size: drag.start_size.to_array(),
                ended: false,
                ui,
                offset: drag.offset.to_array(),
            });
        }
    }

    // Outline is world-sized (shared); handle squares are screen-sized per viewport.
    let hot = state.rect_drag.map(|d| d.handle).or(state.rect_hover);
    for i in 0..4 {
        gizmos.line(corners[i], corners[(i + 1) % 4], OUTLINE);
    }
    for (view_cam, view_at, view_ed, view_proj, slot) in &views {
        let view_zoom = view_ed.flat.then_some(view_ed.zoom);
        let half = world_per_pixel(view_cam, view_proj, view_at.translation(), at, view_zoom) * 4.0;
        match slot.0.min(3) {
            0 => draw_rect_handles(&mut vg.g0, &corners, handles, hot, u, v, half),
            1 => draw_rect_handles(&mut vg.g1, &corners, handles, hot, u, v, half),
            2 => draw_rect_handles(&mut vg.g2, &corners, handles, hot, u, v, half),
            _ => draw_rect_handles(&mut vg.g3, &corners, handles, hot, u, v, half),
        }
    }
}

fn draw_rect_handles<Config: bevy::gizmos::config::GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    corners: &[Vec3; 4],
    handles: &[Handle],
    hot: Option<Handle>,
    u: Vec3,
    v: Vec3,
    half: f32,
) {
    for handle in handles.iter().copied() {
        let p = handle_point(corners, handle);
        let color = if hot == Some(handle) { HOVER } else { OUTLINE };
        let (a, b) = (u * half, v * half);
        let q = [p - a + b, p + a + b, p + a - b, p - a - b];
        for i in 0..4 {
            gizmos.line(q[i], q[(i + 1) % 4], color);
        }
    }
}

/// Shows the drag on the entity while it lasts.
pub(super) fn preview(
    drag: &RectGrab,
    delta: Vec2,
    ui: bool,
    rects: &mut Query<&mut RectTransform>,
    flex: &mut Query<&mut Node, With<StageObject>>,
    transforms: &mut Query<&mut Transform, With<StageObject>>,
) {
    if ui && drag.start_rect.is_none() {
        // A flexbox element: its `Node`'s offset and size.
        let (Some(start), Ok(mut node)) = (drag.start_node, flex.get_mut(drag.entity)) else {
            return;
        };
        let [left, top, right, bottom, width, height] = start;
        let frame = NodeFrame {
            absolute: node.position_type == PositionType::Absolute,
            left: view_val(left),
            top: view_val(top),
            right: view_val(right),
            bottom: view_val(bottom),
        };
        let edit = drag_node(
            &frame,
            drag.handle,
            (delta * drag.ppu).to_array(),
            drag.start_size.to_array(),
            drag.offset.to_array(),
        );
        (
            node.left,
            node.top,
            node.right,
            node.bottom,
            node.width,
            node.height,
        ) = (left, top, right, bottom, width, height);
        apply_edit(&mut node, &edit);
    } else if ui {
        let (Some(start), Ok(mut rect)) = (drag.start_rect, rects.get_mut(drag.entity)) else {
            return;
        };
        let moved = crate::editor::bridge::ui_view::drag_rect(
            crate::editor::bridge::ui_view::ViewRect {
                anchor_min: start.anchor_min.to_array(),
                anchor_max: start.anchor_max.to_array(),
                pivot: start.pivot.to_array(),
                position: start.position.to_array(),
                size: start.size.to_array(),
            },
            drag.handle,
            (delta * drag.ppu).to_array(),
            drag.start_size.to_array(),
        );
        rect.position = Vec2::from_array(moved.position);
        rect.size = Vec2::from_array(moved.size);
    } else if let Ok(mut transform) = transforms.get_mut(drag.entity) {
        let (center, size) = crate::editor::bridge::ui_view::drag_sprite(
            [0.0, 0.0],
            drag.start_size.to_array(),
            drag.handle,
            delta.to_array(),
        );
        let scale = Vec2::from_array(size) / drag.start_size.max(Vec2::splat(0.0001));
        transform.translation =
            drag.start_transform.translation + Vec3::new(center[0], center[1], 0.0);
        transform.scale = Vec3::new(
            drag.start_transform.scale.x * scale.x,
            drag.start_transform.scale.y * scale.y,
            drag.start_transform.scale.z,
        );
    }
}

fn view_val(val: Val) -> ViewVal {
    match val {
        Val::Px(v) => ViewVal::Px(v),
        Val::Percent(v) => ViewVal::Percent(v),
        Val::Vw(v) => ViewVal::Vw(v),
        Val::Vh(v) => ViewVal::Vh(v),
        _ => ViewVal::Auto,
    }
}

/// Writes a drag's result into a `Node`.
fn apply_edit(node: &mut Node, edit: &NodeEdit) {
    if let Some(v) = edit.left {
        node.left = Val::Px(v);
    }
    if let Some(v) = edit.top {
        node.top = Val::Px(v);
    }
    if let Some(v) = edit.width {
        node.width = Val::Px(v);
    }
    if let Some(v) = edit.height {
        node.height = Val::Px(v);
    }
    if edit.clear_right {
        node.right = Val::Auto;
    }
    if edit.clear_bottom {
        node.bottom = Val::Auto;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drag_edit_lands_on_the_node() {
        let mut node = Node {
            right: Val::Px(8.0),
            position_type: PositionType::Absolute,
            ..default()
        };
        let frame = NodeFrame {
            absolute: node.position_type == PositionType::Absolute,
            left: view_val(node.left),
            top: view_val(node.top),
            right: view_val(node.right),
            bottom: view_val(node.bottom),
        };
        assert!(frame.absolute);
        assert_eq!(frame.right, ViewVal::Px(8.0));
        let edit = drag_node(&frame, [1, 0], [10.0, 0.0], [50.0, 20.0], [4.0, 6.0]);
        apply_edit(&mut node, &edit);
        assert_eq!((node.left, node.top), (Val::Px(4.0), Val::Px(6.0)));
        assert_eq!((node.width, node.height), (Val::Px(60.0), Val::Px(20.0)));
        assert_eq!(node.right, Val::Auto);
    }

    #[test]
    fn handles_sit_on_the_corners_and_sides() {
        let corners = [
            Vec3::new(-2.0, 1.0, 0.0),
            Vec3::new(2.0, 1.0, 0.0),
            Vec3::new(2.0, -1.0, 0.0),
            Vec3::new(-2.0, -1.0, 0.0),
        ];
        assert_eq!(handle_point(&corners, [-1, 1]), Vec3::new(-2.0, 1.0, 0.0));
        assert_eq!(handle_point(&corners, [1, -1]), Vec3::new(2.0, -1.0, 0.0));
        assert_eq!(handle_point(&corners, [0, 1]), Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(handle_point(&corners, [-1, 0]), Vec3::new(-2.0, 0.0, 0.0));
    }

    #[test]
    fn a_point_is_inside_a_convex_quad() {
        let q = [
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        assert!(inside(&q, Vec2::new(5.0, 5.0)));
        assert!(!inside(&q, Vec2::new(11.0, 5.0)));
    }
}
