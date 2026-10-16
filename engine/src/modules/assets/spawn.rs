//! Spawning scenes and prefabs into the world. The game and the editor's viewport use the
//! same functions, so a scene looks the same in both.

use bevy::prelude::*;

use super::defs::{Object, SceneAsset};

/// Marks the root entities of a spawned scene (despawn them to unload the scene).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneRootObject;

/// Spawns every object of `scene`; returns the root entities.
pub fn spawn_scene(world: &mut World, scene: &SceneAsset) -> Vec<Entity> {
    scene
        .objects
        .iter()
        .map(|object| {
            let entity = spawn_object(world, object, None);
            world.entity_mut(entity).insert(SceneRootObject);
            entity
        })
        .collect()
}

/// Spawns one object (and its prefab, components and children) under `parent`.
pub fn spawn_object(world: &mut World, object: &Object, parent: Option<Entity>) -> Entity {
    let mut entity = world.spawn((
        Name::new(object.name.clone()),
        Transform::default(),
        Visibility::default(),
    ));
    if let Some(parent) = parent {
        entity.insert(ChildOf(parent));
    }
    let id = entity.id();
    build(world, id, object);
    id
}

/// Applies `object` to `entity`: first its prefab (recursively), then its own components,
/// which replace the prefab's components of the same type, then its children.
fn build(world: &mut World, entity: Entity, object: &Object) {
    if let Some(prefab) = object.prefab {
        build(world, entity, &prefab().root);
    }
    {
        let mut target = world.entity_mut(entity);
        if !object.name.is_empty() {
            target.insert(Name::new(object.name.clone()));
        }
        for component in &object.components {
            component.insert(&mut target);
        }
    }
    for child in &object.children {
        spawn_object(world, child, Some(entity));
    }
}
