//! Transform gizmos on the selected object. Move and rotate use the world's axes (Global) or
//! the object's own (Local), toggled with X like Unity; scale always uses the object's axes,
//! because a scale is along the object's orientation.
//!
//! | Tool (key) | Handles                                                    |
//! |------------|------------------------------------------------------------|
//! | Move (W)   | X/Y/Z arrows, and a square per plane (XY, YZ, XZ)          |
//! | Rotate (E) | a ring per axis                                            |
//! | Scale (R)  | X/Y/Z boxes, and a center box for uniform scale            |
//!
//! Handles keep a constant size on screen and light up under the pointer. A drag moves the
//! object live and reports its new transform to the UI every frame (`EditorSnapshot::dragged`),
//! which writes it into the scene document. Gizmo handles win over picking.

use bevy::prelude::*;

use super::camera::{ActiveCamera, EditorCamera, SceneInput};
use super::{ActiveSpace, ActiveTool, Selected, StageObject};
use crate::editor::project::codec::TransformData;
use crate::editor::protocol::{Space, Tool};

const AXES: [Vec3; 3] = [Vec3::X, Vec3::Y, Vec3::Z];
const COLORS: [Color; 3] = [
    Color::srgb(0.93, 0.27, 0.27),
    Color::srgb(0.38, 0.85, 0.3),
    Color::srgb(0.3, 0.52, 0.98),
];
const HOVER: Color = Color::srgb(1.0, 0.86, 0.2);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Handle {
    /// Move or scale along an axis.
    Axis(usize),
    /// Move in the plane whose normal is this axis.
    Plane(usize),
    /// Rotate about an axis.
    Ring(usize),
    /// Scale uniformly.
    Center,
}

#[derive(Debug, Clone, Copy)]
struct Drag {
    handle: Handle,
    entity: Entity,
    /// The world transform at the start (to work in world space).
    start_world: GlobalTransform,
    /// Where the drag started: a point on the handle's line/plane, and the cursor.
    anchor: Vec3,
    cursor: Vec2,
    /// The handle axes for the whole drag (world or the object's, at the start).
    axes: [Vec3; 3],
}

/// The gizmo's axes: the object's own for Local (and always for scale), else the world's.
pub fn axes_for(tool: Tool, space: Space, rotation: Quat) -> [Vec3; 3] {
    if tool == Tool::Scale || space == Space::Local {
        AXES.map(|a| rotation * a)
    } else {
        AXES
    }
}

/// The gizmo's state. `hover` is read by picking: a click on a handle is not a pick.
#[derive(Resource, Debug, Default)]
pub struct GizmoState {
    pub hover: Option<Handle>,
    drag: Option<Drag>,
    /// Drags seen so far (each one gets its own sequence number in the snapshot).
    pub seq: u64,
    /// The dragged object's id and transform this frame, for the UI.
    pub dragged: Option<(u32, TransformData, u64)>,
}

impl GizmoState {
    pub fn dragging(&self) -> bool {
        self.drag.is_some()
    }
}

/// Size of the gizmo at `at` so it covers the same part of the screen at any distance (or,
/// in the flat 2D view, at any zoom).
fn size(camera: &GlobalTransform, at: Vec3, flat_zoom: Option<f32>) -> f32 {
    match flat_zoom {
        Some(zoom) => zoom * 90.0,
        None => camera.translation().distance(at).max(0.5) * 0.16,
    }
}

/// Closest point on the line `origin + t * dir` to the ray; returns `t`.
fn line_param(ray: Ray3d, origin: Vec3, dir: Vec3) -> Option<f32> {
    let d = *ray.direction;
    let w = origin - ray.origin;
    let (a, b, c) = (dir.dot(dir), dir.dot(d), d.dot(d));
    let (dd, e) = (dir.dot(w), d.dot(w));
    let denom = a * c - b * b;
    (denom.abs() > 1e-6).then(|| (b * e - c * dd) / denom)
}

fn distance_ray_point(ray: Ray3d, point: Vec3) -> f32 {
    let along = (point - ray.origin).dot(*ray.direction).max(0.0);
    (ray.origin + *ray.direction * along).distance(point)
}

fn ray_plane(ray: Ray3d, origin: Vec3, normal: Vec3) -> Option<Vec3> {
    let denom = ray.direction.dot(normal);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = (origin - ray.origin).dot(normal) / denom;
    (t > 0.0).then(|| ray.origin + *ray.direction * t)
}

/// Hit radius in screen pixels, at least: a mouse is precise, a finger is not.
const GRAB_PX_MOUSE: f32 = 10.0;
const GRAB_PX_TOUCH: f32 = 22.0;

/// World units per logical pixel at `at`: from the 2D zoom, or the perspective camera's field
/// of view and the Scene view's height.
fn world_per_pixel(
    camera: &Camera,
    projection: &Projection,
    from: Vec3,
    at: Vec3,
    flat_zoom: Option<f32>,
) -> f32 {
    if let Some(zoom) = flat_zoom {
        return zoom;
    }
    let height = camera
        .logical_viewport_size()
        .map_or(600.0, |s| s.y.max(1.0));
    let fov = match projection {
        Projection::Perspective(p) => p.fov,
        _ => std::f32::consts::FRAC_PI_4,
    };
    2.0 * from.distance(at).max(0.1) * (fov * 0.5).tan() / height
}

/// The handle under the ray, for the tool, at `at` with gizmo size `s`. `grab` is the least
/// hit distance in world units (a few screen pixels), so handles stay easy to hit on small
/// screens and with a finger.
fn hit(tool: Tool, axes: [Vec3; 3], ray: Ray3d, at: Vec3, s: f32, grab: f32) -> Option<Handle> {
    let tolerance = (s * 0.09).max(grab);
    let mut best: Option<(f32, Handle)> = None;
    let mut consider = |distance: f32, handle: Handle| {
        if best.is_none_or(|(d, _)| distance < d) {
            best = Some((distance, handle));
        }
    };
    // Scale's center box sits where the three axes meet: it wins there.
    if tool == Tool::Scale && distance_ray_point(ray, at) < (s * 0.14).max(grab) {
        return Some(Handle::Center);
    }
    match tool {
        Tool::Move | Tool::Scale => {
            for (i, axis) in axes.iter().enumerate() {
                if let Some(t) = line_param(ray, at, *axis) {
                    if (0.0..=s * 1.1).contains(&t) {
                        let d = distance_ray_point(ray, at + *axis * t);
                        if d < tolerance {
                            consider(d, Handle::Axis(i));
                        }
                    }
                }
            }
            if tool == Tool::Move {
                for i in 0..3 {
                    let (u, v) = (axes[(i + 1) % 3], axes[(i + 2) % 3]);
                    if let Some(p) = ray_plane(ray, at, axes[i]) {
                        let (a, b) = ((p - at).dot(u), (p - at).dot(v));
                        if (s * 0.2..=s * 0.42).contains(&a) && (s * 0.2..=s * 0.42).contains(&b) {
                            consider(tolerance * 0.5, Handle::Plane(i));
                        }
                    }
                }
            }
        }
        Tool::Rotate => {
            for i in 0..3 {
                if let Some(p) = ray_plane(ray, at, axes[i]) {
                    let d = ((p - at).length() - s).abs();
                    if d < tolerance * 1.4 {
                        consider(d, Handle::Ring(i));
                    }
                }
            }
        }
    }
    best.map(|(_, h)| h)
}

fn to_data(t: &Transform) -> TransformData {
    let (y, x, z) = t.rotation.to_euler(EulerRot::YXZ);
    let round = |v: f32| (v * 1000.0).round() / 1000.0;
    TransformData {
        translation: t.translation.to_array().map(round),
        rotation: [x, y, z].map(|r| round(r.to_degrees())),
        scale: t.scale.to_array().map(round),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn gizmo(
    mut state: ResMut<GizmoState>,
    tool: Res<ActiveTool>,
    space: Res<ActiveSpace>,
    selected: Res<Selected>,
    input: Res<SceneInput>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    camera: Single<(&Camera, &GlobalTransform, &EditorCamera, &Projection), With<ActiveCamera>>,
    touch: Res<super::touch::TouchPointer>,
    mut objects: Query<(
        Entity,
        &StageObject,
        &GlobalTransform,
        &mut Transform,
        Option<&ChildOf>,
    )>,
    parents: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    state.dragged = None;
    let (camera, camera_at, editor_camera, projection) = *camera;
    let flat_zoom = editor_camera.flat.then_some(editor_camera.zoom);
    let Some(id) = selected.0 else {
        state.hover = None;
        state.drag = None;
        return;
    };
    let Some((entity, _, world, _, _)) = objects.iter().find(|(_, o, ..)| o.id == id) else {
        return;
    };
    let at = world.translation();
    let s = size(camera_at, at, flat_zoom);
    let axes = match state.drag {
        Some(drag) => drag.axes,
        None => axes_for(tool.0, space.0, world.rotation()),
    };
    let ray = input
        .window_cursor
        .and_then(|c| camera.viewport_to_world(camera_at, c).ok());

    // Start, continue or end a drag.
    let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    if state.drag.is_none() {
        state.hover = if input.over_scene && input.dragging.is_none() {
            let px = if touch.touching() {
                GRAB_PX_TOUCH
            } else {
                GRAB_PX_MOUSE
            };
            let grab =
                px * world_per_pixel(camera, projection, camera_at.translation(), at, flat_zoom);
            ray.and_then(|r| hit(tool.0, axes, r, at, s, grab))
        } else {
            None
        };
        if let (Some(handle), Some(ray), Some(cursor), true, false) = (
            state.hover,
            ray,
            input.window_cursor,
            buttons.just_pressed(MouseButton::Left),
            alt,
        ) {
            let anchor = match handle {
                Handle::Axis(i) => line_param(ray, at, axes[i]).map(|t| at + axes[i] * t),
                Handle::Plane(i) | Handle::Ring(i) => ray_plane(ray, at, axes[i]),
                Handle::Center => Some(at),
            };
            if let (Some(anchor), Ok((_, _, world, _, _))) = (anchor, objects.get(entity)) {
                state.seq += 1;
                state.drag = Some(Drag {
                    handle,
                    entity,
                    start_world: *world,
                    anchor,
                    cursor,
                    axes,
                });
            }
        }
    } else if !buttons.pressed(MouseButton::Left) {
        state.drag = None;
    }

    if let (Some(drag), Some(ray), Some(cursor)) = (state.drag, ray, input.window_cursor) {
        let world_t = drag_to(tool.0, &drag, ray, cursor);
        // Back to the object's parent space.
        let (_, object, _, mut local, child_of) =
            objects.get_mut(drag.entity).expect("dragged object");
        let new_local = match child_of.and_then(|p| parents.get(p.parent()).ok()) {
            Some(parent) => GlobalTransform::from(world_t).reparented_to(parent),
            None => world_t,
        };
        *local = new_local;
        state.dragged = Some((object.id, to_data(&new_local), state.seq));
    }

    draw(
        &mut gizmos,
        tool.0,
        axes,
        at,
        s,
        state.hover.or(state.drag.map(|d| d.handle)),
    );
}

/// The dragged object's world transform for the pointer's `ray` (and `cursor`, for uniform
/// scale) during `drag`.
fn drag_to(tool: Tool, drag: &Drag, ray: Ray3d, cursor: Vec2) -> Transform {
    let axes = drag.axes;
    let start_at = drag.start_world.translation();
    let mut world_t = drag.start_world.compute_transform();
    match drag.handle {
        Handle::Axis(i) if tool == Tool::Move => {
            if let Some(t) = line_param(ray, start_at, axes[i]) {
                let from = (drag.anchor - start_at).dot(axes[i]);
                world_t.translation = start_at + axes[i] * (t - from);
            }
        }
        Handle::Plane(i) => {
            if let Some(p) = ray_plane(ray, start_at, axes[i]) {
                world_t.translation = start_at + (p - drag.anchor);
            }
        }
        Handle::Ring(i) => {
            if let Some(p) = ray_plane(ray, start_at, axes[i]) {
                let (a, b) = (
                    (drag.anchor - start_at).normalize_or_zero(),
                    (p - start_at).normalize_or_zero(),
                );
                let angle = a.cross(b).dot(axes[i]).atan2(a.dot(b));
                world_t.rotation =
                    Quat::from_axis_angle(axes[i], angle) * drag.start_world.rotation();
            }
        }
        Handle::Axis(i) => {
            // Scale along the object's axis i: how far along it the pointer is, against where
            // it started.
            if let Some(t) = line_param(ray, start_at, axes[i]) {
                let from = (drag.anchor - start_at).dot(axes[i]);
                if from.abs() > 1e-4 {
                    world_t.scale[i] = (drag.start_world.scale()[i] * t / from).max(0.01);
                }
            }
        }
        Handle::Center => {
            let factor = (1.0 + (drag.cursor.y - cursor.y) * 0.01).max(0.01);
            world_t.scale = drag.start_world.scale() * factor;
        }
    }
    world_t
}

fn draw(
    gizmos: &mut Gizmos,
    tool: Tool,
    axes: [Vec3; 3],
    at: Vec3,
    s: f32,
    active: Option<Handle>,
) {
    let color = |handle: Handle, i: usize| {
        if active == Some(handle) {
            HOVER
        } else {
            COLORS[i]
        }
    };
    match tool {
        Tool::Move => {
            for (i, axis) in axes.iter().enumerate() {
                gizmos
                    .arrow(at, at + *axis * s, color(Handle::Axis(i), i))
                    .with_tip_length(s * 0.22);
                let (u, v) = (axes[(i + 1) % 3], axes[(i + 2) % 3]);
                let (a, b) = (s * 0.2, s * 0.42);
                let corners = [
                    at + u * a + v * a,
                    at + u * b + v * a,
                    at + u * b + v * b,
                    at + u * a + v * b,
                ];
                for k in 0..4 {
                    gizmos.line(corners[k], corners[(k + 1) % 4], color(Handle::Plane(i), i));
                }
            }
        }
        Tool::Rotate => {
            for (i, axis) in axes.iter().enumerate() {
                let iso = Isometry3d::new(at, Quat::from_rotation_arc(Vec3::Z, *axis));
                gizmos
                    .circle(iso, s, color(Handle::Ring(i), i))
                    .resolution(64);
            }
        }
        Tool::Scale => {
            // The boxes turn with the object (its axes).
            let turn = Quat::from_mat3(&Mat3::from_cols(axes[0], axes[1], axes[2]));
            for (i, axis) in axes.iter().enumerate() {
                let end = at + *axis * s;
                gizmos.line(at, end, color(Handle::Axis(i), i));
                gizmos.cube(
                    Transform::from_translation(end)
                        .with_rotation(turn)
                        .with_scale(Vec3::splat(s * 0.12)),
                    color(Handle::Axis(i), i),
                );
            }
            let center = if active == Some(Handle::Center) {
                HOVER
            } else {
                Color::srgb(0.85, 0.85, 0.85)
            };
            gizmos.cube(
                Transform::from_translation(at)
                    .with_rotation(turn)
                    .with_scale(Vec3::splat(s * 0.16)),
                center,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ray from above-front, through `target`.
    fn ray_to(target: Vec3) -> Ray3d {
        let origin = Vec3::new(0.0, 10.0, 10.0);
        Ray3d::new(origin, Dir3::new(target - origin).unwrap())
    }

    fn drag(handle: Handle, at: Vec3, anchor: Vec3) -> Drag {
        Drag {
            handle,
            entity: Entity::PLACEHOLDER,
            start_world: GlobalTransform::from_translation(at),
            anchor,
            cursor: Vec2::ZERO,
            axes: AXES,
        }
    }

    /// `hit` with world axes and no extra grab radius (the old exact rule).
    fn hit_test(tool: Tool, ray: Ray3d, at: Vec3, s: f32) -> Option<Handle> {
        hit(tool, AXES, ray, at, s, 0.0)
    }

    #[test]
    fn a_grab_radius_makes_handles_easier_to_hit() {
        // 0.2 units beside the X arrow: a miss with the exact rule, a hit with a finger's radius.
        let near = ray_to(Vec3::new(0.7, 0.0, 0.2));
        assert_eq!(hit_test(Tool::Move, near, Vec3::ZERO, 1.0), None);
        assert_eq!(
            hit(Tool::Move, AXES, near, Vec3::ZERO, 1.0, 0.25),
            Some(Handle::Axis(0))
        );
    }

    #[test]
    fn handles_are_hit_where_they_are_drawn() {
        let at = Vec3::ZERO;
        let s = 1.0;
        assert_eq!(
            hit_test(Tool::Move, ray_to(Vec3::new(0.7, 0.0, 0.0)), at, s),
            Some(Handle::Axis(0))
        );
        assert_eq!(
            hit_test(Tool::Move, ray_to(Vec3::new(0.0, 0.7, 0.0)), at, s),
            Some(Handle::Axis(1))
        );
        assert_eq!(
            hit_test(Tool::Move, ray_to(Vec3::new(0.3, 0.0, 0.3)), at, s),
            Some(Handle::Plane(1)),
            "the XZ square"
        );
        assert_eq!(
            hit_test(Tool::Move, ray_to(Vec3::new(3.0, 0.0, 3.0)), at, s),
            None
        );
        assert_eq!(
            hit_test(Tool::Rotate, ray_to(Vec3::new(1.0, 0.0, 0.0)), at, s),
            Some(Handle::Ring(1)),
            "the Y ring lies in XZ"
        );
        assert_eq!(
            hit_test(Tool::Scale, ray_to(Vec3::ZERO), at, s),
            Some(Handle::Center)
        );
    }

    #[test]
    fn dragging_an_axis_moves_along_it_only() {
        let start = Vec3::new(1.0, 0.0, 0.0);
        let d = drag(Handle::Axis(0), start, Vec3::new(1.5, 0.0, 0.0));
        // The pointer now over x = 3.5 on the axis: the object follows by the same 2 units.
        let moved = drag_to(Tool::Move, &d, ray_to(Vec3::new(3.5, 0.0, 0.0)), Vec2::ZERO);
        assert!(
            (moved.translation.x - 3.0).abs() < 1e-3,
            "{}",
            moved.translation
        );
        assert_eq!((moved.translation.y, moved.translation.z), (0.0, 0.0));
    }

    #[test]
    fn dragging_the_xz_square_moves_on_the_ground() {
        let d = drag(Handle::Plane(1), Vec3::ZERO, Vec3::new(0.3, 0.0, 0.3));
        let moved = drag_to(
            Tool::Move,
            &d,
            ray_to(Vec3::new(2.3, 0.0, -0.7)),
            Vec2::ZERO,
        );
        assert!(
            moved.translation.distance(Vec3::new(2.0, 0.0, -1.0)) < 1e-3,
            "{}",
            moved.translation
        );
    }

    #[test]
    fn dragging_a_ring_rotates_about_its_axis() {
        let d = drag(Handle::Ring(1), Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
        let turned = drag_to(
            Tool::Rotate,
            &d,
            ray_to(Vec3::new(0.0, 0.0, -1.0)),
            Vec2::ZERO,
        );
        let (yaw, pitch, roll) = turned.rotation.to_euler(EulerRot::YXZ);
        assert!(
            (yaw.to_degrees() - 90.0).abs() < 0.1,
            "yaw {}",
            yaw.to_degrees()
        );
        assert!(pitch.abs() < 1e-3 && roll.abs() < 1e-3);
    }

    #[test]
    fn local_axes_follow_the_object_and_scale_is_always_local() {
        let turned = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        // Global move: world axes, whatever the object's rotation.
        assert_eq!(axes_for(Tool::Move, Space::Global, turned), AXES);
        // Local move: the object's X now points along world -Z.
        let local = axes_for(Tool::Move, Space::Local, turned);
        assert!(local[0].distance(Vec3::NEG_Z) < 1e-5, "{}", local[0]);
        // Scale ignores Global: it is along the object's axes.
        assert_eq!(axes_for(Tool::Scale, Space::Global, turned), local);

        // Dragging the local X arrow moves along the object's X (world -Z).
        let mut d = drag(Handle::Axis(0), Vec3::ZERO, Vec3::new(0.0, 0.0, -0.5));
        d.axes = local;
        let moved = drag_to(
            Tool::Move,
            &d,
            ray_to(Vec3::new(0.0, 0.0, -2.5)),
            Vec2::ZERO,
        );
        assert!(
            moved.translation.distance(Vec3::new(0.0, 0.0, -2.0)) < 1e-3,
            "{}",
            moved.translation
        );
    }

    #[test]
    fn dragging_scale_handles_scales() {
        let d = drag(Handle::Axis(0), Vec3::ZERO, Vec3::new(0.5, 0.0, 0.0));
        let scaled = drag_to(
            Tool::Scale,
            &d,
            ray_to(Vec3::new(1.0, 0.0, 0.0)),
            Vec2::ZERO,
        );
        assert!(
            (scaled.scale.x - 2.0).abs() < 1e-3 && scaled.scale.y == 1.0,
            "{}",
            scaled.scale
        );
        let d = drag(Handle::Center, Vec3::ZERO, Vec3::ZERO);
        let scaled = drag_to(Tool::Scale, &d, ray_to(Vec3::ZERO), Vec2::new(0.0, -100.0));
        assert!((scaled.scale - Vec3::splat(2.0)).length() < 1e-3);
    }
}
