//! The shown scene in the Scene view: spawning it, the images its materials use, picking
//! by click, and the icons and outline the editor draws over it.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::Aabb;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};

use super::camera::{ActiveCamera, SceneInput};
use super::{Picked, Selected, StageObject};
use crate::editor::view::{SceneView, ViewCamera, ViewLight, ViewMesh, ViewObject};
use crate::modules::ui::CanvasSurface;

/// Images by authored path, as the UI sent them.
#[derive(Resource, Default)]
pub struct Textures(HashMap<String, Handle<Image>>);

/// The scene's materials waiting for an image that has not arrived yet.
#[derive(Component)]
struct WantsTexture(String);

/// The scene view last shown, to tell a transform-only change from a new scene.
#[derive(Resource, Default)]
pub struct LastView(Option<SceneView>);

/// Shows `view`. When only transforms changed (an Inspector edit, a gizmo drag), the objects
/// are moved in place; otherwise the scene is spawned again.
pub fn show(world: &mut World, view: &SceneView) {
    let same_objects = world.resource::<LastView>().0.as_ref().is_some_and(|last| {
        last.objects.len() == view.objects.len()
            && last.objects.iter().zip(&view.objects).all(|(a, b)| {
                let mut a = a.clone();
                a.transform = b.transform;
                a == *b
            })
    });
    world.resource_mut::<LastView>().0 = Some(view.clone());
    if same_objects {
        let moved: Vec<(Entity, Transform)> = world
            .query::<(Entity, &StageObject)>()
            .iter(world)
            .filter_map(|(e, o)| {
                view.objects
                    .iter()
                    .find(|v| v.id == o.id)
                    .map(|v| (e, transform_of(v)))
            })
            .collect();
        for (entity, transform) in moved {
            if let Some(mut t) = world.get_mut::<Transform>(entity) {
                if *t != transform {
                    *t = transform;
                }
            }
        }
        return;
    }
    respawn(world, view);
}

fn transform_of(object: &ViewObject) -> Transform {
    let t = object.transform;
    let rotation = Quat::from_euler(
        EulerRot::YXZ,
        t.rotation[1].to_radians(),
        t.rotation[0].to_radians(),
        t.rotation[2].to_radians(),
    );
    Transform {
        translation: Vec3::from_array(t.translation),
        rotation,
        scale: Vec3::from_array(t.scale),
    }
}

fn respawn(world: &mut World, view: &SceneView) {
    debug!("stage: show {} objects", view.objects.len());
    let old: Vec<Entity> = world
        .query_filtered::<Entity, With<StageObject>>()
        .iter(world)
        .collect();
    for entity in old {
        if let Ok(e) = world.get_entity_mut(entity) {
            e.despawn();
        }
    }
    let mut entities: HashMap<u32, Entity> = HashMap::new();
    for object in &view.objects {
        let entity = spawn(
            world,
            object,
            object.parent.and_then(|p| entities.get(&p).copied()),
            view.dimension == crate::editor::project::codec::Dimension::D2,
            view.reference(),
        );
        entities.insert(object.id, entity);
    }
    tag_cameras(world, view);
}

fn spawn(
    world: &mut World,
    object: &ViewObject,
    parent: Option<Entity>,
    flat: bool,
    reference: [u32; 2],
) -> Entity {
    let transform = transform_of(object);
    let mut entity = world.spawn((
        Name::new(object.name.clone()),
        StageObject {
            id: object.id,
            select: object.select,
        },
        transform,
        Visibility::default(),
    ));
    if let Some(parent) = parent {
        entity.insert(ChildOf(parent));
    }
    let id = entity.id();

    if let Some(mesh) = object.mesh {
        let mesh: Mesh = match mesh {
            ViewMesh::Cube(size) => Cuboid::from_length(size).into(),
            ViewMesh::Cuboid(size) => Cuboid::from_size(Vec3::from_array(size)).into(),
            ViewMesh::Sphere(radius) => Sphere::new(radius).into(),
            ViewMesh::Plane(size) => Plane3d::default().mesh().size(size, size).into(),
            ViewMesh::Quad([w, h]) => Rectangle::new(w, h).into(),
            // A model's mesh is inside its file, which the editor does not load yet: a box one
            // unit tall, standing on the object's origin the way a glTF model stands on its own,
            // so the object can be seen, picked and moved where it belongs.
            ViewMesh::Model => {
                Mesh::from(Cuboid::new(0.8, 1.0, 0.8)).translated_by(Vec3::new(0.0, 0.5, 0.0))
            }
        };
        let [r, g, b, a] = object.color;
        let texture = object
            .texture
            .as_ref()
            .and_then(|p| world.resource::<Textures>().0.get(p).cloned());
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgba(r, g, b, a),
                base_color_texture: texture.clone(),
                unlit: matches!(object.mesh, Some(ViewMesh::Quad(_))),
                ..default()
            });
        let mut e = world.entity_mut(id);
        e.insert((Mesh3d(mesh), MeshMaterial3d(material)));
        if matches!(object.mesh, Some(ViewMesh::Quad(_))) {
            e.insert(super::rect_tool::StageSprite);
        }
        if let (Some(path), None) = (&object.texture, texture) {
            e.insert(WantsTexture(path.clone()));
        }
    }
    match object.light {
        Some(ViewLight::Directional { shadows }) => {
            world.entity_mut(id).insert(DirectionalLight {
                shadows_enabled: shadows,
                ..default()
            });
        }
        Some(ViewLight::Point) => {
            world.entity_mut(id).insert(PointLight::default());
        }
        Some(ViewLight::Spot) => {
            world.entity_mut(id).insert(SpotLight::default());
        }
        None => {}
    }
    if object.hidden {
        world.entity_mut(id).insert(Visibility::Hidden);
    }
    if let Some(ui) = &object.ui {
        // A UI element with no UI element above it is a canvas.
        let is_root = parent.is_none_or(|p| world.get::<Node>(p).is_none());
        super::ui::insert_ui(world, id, ui, is_root, flat, reference);
    }
    id
}

/// Decodes an image the UI fetched and puts it on every material waiting for it.
pub fn add_texture(world: &mut World, path: String, bytes: &[u8]) {
    let extension = path.rsplit('.').next().unwrap_or("png").to_string();
    let image = match Image::from_buffer(
        bytes,
        ImageType::Extension(&extension),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::nearest(),
        RenderAssetUsages::default(),
    ) {
        Ok(image) => image,
        Err(err) => {
            warn!("editor: cannot show {path}: {err}");
            return;
        }
    };
    let handle = world.resource_mut::<Assets<Image>>().add(image);
    world
        .resource_mut::<Textures>()
        .0
        .insert(path.clone(), handle.clone());
    let waiting: Vec<(Entity, Handle<StandardMaterial>)> = world
        .query::<(Entity, &WantsTexture, &MeshMaterial3d<StandardMaterial>)>()
        .iter(world)
        .filter(|(_, wants, _)| wants.0 == path)
        .map(|(e, _, m)| (e, m.0.clone()))
        .collect();
    for (entity, material) in waiting {
        if let Some(material) = world
            .resource_mut::<Assets<StandardMaterial>>()
            .get_mut(&material)
        {
            material.base_color_texture = Some(handle.clone());
        }
        world.entity_mut(entity).remove::<WantsTexture>();
    }
}

/// Where a left press started, to tell a click from a drag.
#[derive(Default)]
pub struct PressAt(Option<Vec2>);

/// A plain left click in the Scene view selects the nearest object under the pointer (or
/// clears the selection on empty space).
#[allow(clippy::too_many_arguments)]
pub fn pick(
    mut press: Local<PressAt>,
    mut seq: Local<u64>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    input: Res<SceneInput>,
    camera: Single<(&Camera, &GlobalTransform), With<ActiveCamera>>,
    objects: Query<(&GlobalTransform, &StageObject, Option<&Aabb>, Has<Node>)>,
    ui_objects: Query<(Entity, &StageObject), With<Node>>,
    layers: Query<
        (&crate::modules::ui::Canvas, &StageObject, &GlobalTransform),
        Without<CanvasSurface>,
    >,
    parents: Query<&ChildOf>,
    surfaces: Query<(&CanvasSurface, &GlobalTransform)>,
    ui_nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut selected: ResMut<Selected>,
    mut picked: ResMut<Picked>,
    gizmo: Res<super::gizmo::GizmoState>,
) {
    let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    // A press on a gizmo handle is a drag, not a pick.
    if buttons.just_pressed(MouseButton::Left)
        && input.over_scene
        && !alt
        && !gizmo.hovering()
        && !gizmo.dragging()
    {
        press.0 = input.cursor;
    }
    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let (Some(start), Some(end)) = (press.0.take(), input.cursor) else {
        return;
    };
    if start.distance(end) > 4.0 {
        return;
    }
    let (camera, camera_at) = *camera;
    let Some(window_cursor) = input.window_cursor else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_at, window_cursor) else {
        return;
    };

    // (distance, area in px for UI so a button beats the panel behind it, what it selects)
    let mut best: Option<(f32, f32, u32)> = None;
    let mut consider = |t: f32, area: f32, id: u32| {
        let better =
            best.is_none_or(|(d, a, _)| t < d - 1e-3 || ((t - d).abs() <= 1e-3 && area < a));
        if better {
            best = Some((t, area, id));
        }
    };
    for (at, object, aabb, is_ui) in &objects {
        if is_ui {
            continue;
        }
        let hit = match aabb {
            Some(aabb) => ray_box(ray, at, aabb),
            // Cameras, lights and empties: a small sphere at their position.
            None => ray_sphere(ray, at.translation(), 0.35),
        };
        if let Some(t) = hit {
            consider(t, f32::MAX, object.select);
        }
    }
    // UI elements: their laid-out rectangles on the canvas.
    for (entity, object) in &ui_objects {
        if let Some((corners, area)) = super::ui::world_rect(entity, &parents, &surfaces, &ui_nodes)
        {
            if let Some(t) = super::ui::ray_quad(ray, &corners) {
                consider(t, area, object.select);
            }
        }
    }
    // A canvas that only groups world objects: its rectangle, behind everything in it.
    for (canvas, object, at) in &layers {
        if let crate::modules::ui::CanvasMode::WorldSpace {
            size,
            pixels_per_unit,
        } = canvas.mode
        {
            let half = size.as_vec2() / pixels_per_unit / 2.0;
            let corners = [
                at.transform_point(Vec3::new(-half.x, half.y, 0.0)),
                at.transform_point(Vec3::new(half.x, half.y, 0.0)),
                at.transform_point(Vec3::new(half.x, -half.y, 0.0)),
                at.transform_point(Vec3::new(-half.x, -half.y, 0.0)),
            ];
            if let Some(t) = super::ui::ray_quad(ray, &corners) {
                // Considered last with the largest area: whatever is in it wins a tie.
                consider(t, f32::MAX, object.select);
            }
        }
    }
    let additive = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
    ]);
    let hit = best.map(|(_, _, id)| id);
    // Ctrl or Shift adds to the selection (or takes away): the UI owns the set.
    if !additive {
        selected.0 = hit;
        selected.1.clear();
    }
    *seq += 1;
    picked.0 = Some((hit, *seq, additive));
}

/// Distance along `ray` to the object's box, if it hits.
fn ray_box(ray: Ray3d, at: &GlobalTransform, aabb: &Aabb) -> Option<f32> {
    let to_local = at.affine().inverse();
    let origin = to_local.transform_point3(ray.origin);
    let direction = to_local.transform_vector3(*ray.direction);
    let (min, max) = (Vec3::from(aabb.min()), Vec3::from(aabb.max()));
    let mut near = f32::NEG_INFINITY;
    let mut far = f32::INFINITY;
    for axis in 0..3 {
        let (o, d) = (origin[axis], direction[axis]);
        if d.abs() < 1e-8 {
            if o < min[axis] || o > max[axis] {
                return None;
            }
            continue;
        }
        let (a, b) = ((min[axis] - o) / d, (max[axis] - o) / d);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far && far >= 0.0).then(|| {
        // Back to world distance (the local direction may be scaled).
        let local_hit = origin + direction * near.max(0.0);
        at.transform_point(local_hit).distance(ray.origin)
    })
}

fn ray_sphere(ray: Ray3d, center: Vec3, radius: f32) -> Option<f32> {
    let to = center - ray.origin;
    let along = to.dot(*ray.direction);
    let closest = to.length_squared() - along * along;
    (along > 0.0 && closest <= radius * radius).then_some(along)
}

/// Gameplay cameras (as frustums) and lights (as markers): they are scene objects, not the
/// editor's camera.
pub fn draw_icons(
    mut gizmos: Gizmos,
    opts: Res<super::ViewOpts>,
    objects: Query<
        (
            &GlobalTransform,
            Option<&DirectionalLight>,
            Option<&PointLight>,
            Option<&SpotLight>,
        ),
        With<StageObject>,
    >,
) {
    if !opts.0.icons {
        return;
    }
    for (at, directional, point, spot) in &objects {
        let color = Color::srgb(1.0, 0.85, 0.35);
        if directional.is_some() {
            let from = at.translation();
            gizmos.sphere(Isometry3d::from_translation(from), 0.25, color);
            gizmos.arrow(from, from + at.forward() * 1.5, color);
        } else if point.is_some() || spot.is_some() {
            gizmos.sphere(Isometry3d::from_translation(at.translation()), 0.25, color);
        }
    }
}

/// What an orthographic game camera frames (see `ViewObject::ortho`).
#[derive(Component, Debug, Clone, Copy)]
pub struct StageCameraView(pub crate::editor::view::ViewOrtho);

/// The part of the world each orthographic game camera shows on the project's screen: its
/// frame, so the level (and what is on a canvas) can be placed against what a player sees.
pub fn draw_camera_frames(
    mut gizmos: Gizmos,
    opts: Res<super::ViewOpts>,
    last: Res<LastView>,
    cameras: Query<(&GlobalTransform, &StageCameraView)>,
) {
    if !opts.0.icons {
        return;
    }
    let Some(view) = last.0.as_ref() else { return };
    let screen = view.reference();
    for (at, camera) in &cameras {
        let [w, h] = camera.0.visible(screen);
        let (hw, hh) = (w / 2.0, h / 2.0);
        let corners = [
            at.transform_point(Vec3::new(-hw, hh, 0.0)) + Vec3::Z * 5.0,
            at.transform_point(Vec3::new(hw, hh, 0.0)) + Vec3::Z * 5.0,
            at.transform_point(Vec3::new(hw, -hh, 0.0)) + Vec3::Z * 5.0,
            at.transform_point(Vec3::new(-hw, -hh, 0.0)) + Vec3::Z * 5.0,
        ];
        for i in 0..4 {
            gizmos.line(
                corners[i],
                corners[(i + 1) % 4],
                Color::srgb(0.45, 0.8, 1.0),
            );
        }
    }
}

/// The project's screen (its window size) as a lighter area behind the scene in a 2D scene:
/// where an overlay canvas lives, so a HUD is placed against the window it will be drawn in.
#[derive(Component)]
pub struct ScreenFrame;

pub fn update_screen_frame(
    mut commands: Commands,
    opts: Res<super::ViewOpts>,
    last: Res<LastView>,
    mut frames: Query<(&mut Transform, &mut Visibility), With<ScreenFrame>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let flat = last
        .0
        .as_ref()
        .is_some_and(|v| v.dimension == crate::editor::project::codec::Dimension::D2);
    let size = last
        .0
        .as_ref()
        .map(|v| v.reference())
        .unwrap_or([1280, 720]);
    let Ok((mut transform, mut visibility)) = frames.single_mut() else {
        commands.spawn((
            ScreenFrame,
            Name::new("Screen Frame"),
            Mesh3d(meshes.add(Rectangle::new(1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.235, 0.24, 0.26),
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, -50.0),
            Visibility::Hidden,
        ));
        return;
    };
    let scale = Vec3::new(size[0] as f32, size[1] as f32, 1.0);
    if transform.scale != scale {
        transform.scale = scale;
    }
    let wanted = if flat && opts.0.canvases {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if *visibility != wanted {
        *visibility = wanted;
    }
}

/// Marks a gameplay camera object in the Scene view (drawn as a frustum; never active).
#[derive(Component, Debug, Clone, Copy)]
pub struct StageCameraKind(#[allow(dead_code)] pub ViewCamera);

/// The selection: its box (or a marker for objects without a mesh).
#[allow(clippy::too_many_arguments)]
pub fn draw_selection(
    mut gizmos: Gizmos,
    opts: Res<super::ViewOpts>,
    selected: Res<Selected>,
    objects: Query<(
        Entity,
        &GlobalTransform,
        &StageObject,
        Option<&Aabb>,
        Has<Node>,
    )>,
    parents: Query<&ChildOf>,
    surfaces: Query<(&CanvasSurface, &GlobalTransform)>,
    ui_nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
) {
    let ids: Vec<u32> = selected
        .0
        .into_iter()
        .chain(selected.1.iter().copied())
        .collect();
    if !opts.0.selection || ids.is_empty() {
        return;
    }
    let color = Color::srgb(1.0, 0.55, 0.1);
    for (entity, at, object, aabb, is_ui) in &objects {
        if !ids.contains(&object.select) {
            continue;
        }
        if is_ui {
            // A UI element: its rectangle on the canvas.
            if let Some((c, _)) = super::ui::world_rect(entity, &parents, &surfaces, &ui_nodes) {
                for i in 0..4 {
                    gizmos.line(c[i], c[(i + 1) % 4], color);
                }
            }
            continue;
        }
        match aabb {
            Some(aabb) => {
                let center = Vec3::from(aabb.center);
                let size = Vec3::from(aabb.half_extents) * 2.0;
                let transform =
                    at.compute_transform() * Transform::from_translation(center).with_scale(size);
                gizmos.cube(transform, color);
            }
            None if ids.contains(&object.id) => {
                gizmos.sphere(Isometry3d::from_translation(at.translation()), 0.4, color);
            }
            None => {}
        }
    }
}

/// Draws a gameplay camera's frustum.
pub fn draw_cameras(
    mut gizmos: Gizmos,
    opts: Res<super::ViewOpts>,
    cameras: Query<(&GlobalTransform, &StageCameraKind)>,
) {
    if !opts.0.icons {
        return;
    }
    for (at, _) in &cameras {
        let o = at.translation();
        let (f, r, u) = (at.forward() * 1.2, at.right() * 0.6, at.up() * 0.4);
        let corners = [o + f + r + u, o + f - r + u, o + f - r - u, o + f + r - u];
        let color = Color::srgb(0.75, 0.75, 0.8);
        for (i, corner) in corners.iter().enumerate() {
            gizmos.line(o, *corner, color);
            gizmos.line(*corner, corners[(i + 1) % 4], color);
        }
    }
}

/// Tags gameplay camera objects after a scene is shown.
pub fn tag_cameras(world: &mut World, view: &SceneView) {
    let ids: HashMap<u32, ViewCamera> = view
        .objects
        .iter()
        .filter_map(|o| o.camera.map(|c| (o.id, c)))
        .collect();
    let tagged: Vec<(Entity, ViewCamera)> = world
        .query::<(Entity, &StageObject)>()
        .iter(world)
        .filter_map(|(e, o)| ids.get(&o.id).map(|c| (e, *c)))
        .collect();
    for (entity, kind) in tagged {
        world.entity_mut(entity).insert(StageCameraKind(kind));
    }
    let framed: Vec<(Entity, crate::editor::view::ViewOrtho)> = world
        .query::<(Entity, &StageObject)>()
        .iter(world)
        .filter_map(|(e, o)| {
            view.objects
                .iter()
                .find(|v| v.id == o.id)
                .and_then(|v| v.ortho)
                .map(|ortho| (e, ortho))
        })
        .collect();
    for (entity, ortho) in framed {
        world.entity_mut(entity).insert(StageCameraView(ortho));
    }
}
