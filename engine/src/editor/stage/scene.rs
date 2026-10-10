//! The shown scene in the Scene view: spawning it, the images its materials use, picking
//! by click, and the icons and outline the editor draws over it.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::Aabb;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;

use super::camera::{ActiveCamera, SceneInput};
use super::{Picked, Selected, StageObject};
use crate::editor::view::{SceneView, ViewCamera, ViewLight, ViewMesh, ViewObject};

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
        );
        entities.insert(object.id, entity);
    }
    tag_cameras(world, view);
}

fn spawn(world: &mut World, object: &ViewObject, parent: Option<Entity>) -> Entity {
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
            ViewMesh::Sphere(radius) => Sphere::new(radius).into(),
            ViewMesh::Plane(size) => Plane3d::default().mesh().size(size, size).into(),
            ViewMesh::Quad([w, h]) => Rectangle::new(w, h).into(),
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
    objects: Query<(&GlobalTransform, &StageObject, Option<&Aabb>)>,
    mut selected: ResMut<Selected>,
    mut picked: ResMut<Picked>,
    gizmo: Res<super::gizmo::GizmoState>,
) {
    let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    // A press on a gizmo handle is a drag, not a pick.
    if buttons.just_pressed(MouseButton::Left)
        && input.over_scene
        && !alt
        && gizmo.hover.is_none()
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

    let mut best: Option<(f32, u32)> = None;
    for (at, object, aabb) in &objects {
        let hit = match aabb {
            Some(aabb) => ray_box(ray, at, aabb),
            // Cameras, lights and empties: a small sphere at their position.
            None => ray_sphere(ray, at.translation(), 0.35),
        };
        if let Some(t) = hit {
            if best.is_none_or(|(d, _)| t < d) {
                best = Some((t, object.select));
            }
        }
    }
    selected.0 = best.map(|(_, id)| id);
    *seq += 1;
    picked.0 = Some((selected.0, *seq));
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

/// Marks a gameplay camera object in the Scene view (drawn as a frustum; never active).
#[derive(Component, Debug, Clone, Copy)]
pub struct StageCameraKind(#[allow(dead_code)] pub ViewCamera);

/// The selection: its box (or a marker for objects without a mesh).
pub fn draw_selection(
    mut gizmos: Gizmos,
    selected: Res<Selected>,
    objects: Query<(&GlobalTransform, &StageObject, Option<&Aabb>)>,
) {
    let Some(id) = selected.0 else { return };
    let color = Color::srgb(1.0, 0.55, 0.1);
    for (at, object, aabb) in &objects {
        if object.select != id {
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
            None if object.id == id => {
                gizmos.sphere(Isometry3d::from_translation(at.translation()), 0.4, color);
            }
            None => {}
        }
    }
}

/// Draws a gameplay camera's frustum.
pub fn draw_cameras(mut gizmos: Gizmos, cameras: Query<(&GlobalTransform, &StageCameraKind)>) {
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
}
