//! Spawning scenes and prefabs into the world. The game and the editor's viewport use the
//! same functions, so a scene looks the same in both.
//!
//! A game that makes objects while it runs ([`spawn`]) uses the same prefabs its scene does, so
//! there is one place that says what a thing looks like:
//!
//! ```ignore
//! let goon = Object::prefab(crate::assets::prefabs::games::solo::siavash::goon_prefab::prefab)
//!     .with(Transform::from_xyz(x, y, z));
//! let entity = spawn(&mut commands, goon, Some(road));
//! commands.entity(entity).insert(FoeView { at, lane });
//! ```

use bevy::prelude::*;

use super::defs::{AssetNeed, Object, SceneAsset};

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

/// Spawns `object` under `parent` **from a system**, through `Commands`: the entity is returned at
/// once (ready for the game's own components), and the object's prefab, components and children
/// arrive when the command runs. The same prefabs a scene uses are therefore the ones a game
/// spawns while it plays.
pub fn spawn(commands: &mut Commands, object: Object, parent: Option<Entity>) -> Entity {
    let entity = commands.spawn_empty().id();
    commands.queue(move |world: &mut World| {
        if world.get_entity(entity).is_err() {
            return;
        }
        {
            let mut target = world.entity_mut(entity);
            target.insert((
                Name::new(object.name.clone()),
                Transform::default(),
                Visibility::default(),
            ));
            if let Some(parent) = parent {
                target.insert(ChildOf(parent));
            }
        }
        build(world, entity, &object);
    });
    entity
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

/// Every file a scene's objects load, prefabs resolved, each one once. The flow asks for these
/// before it spawns the scene, so a level never starts with its art missing.
pub fn scene_needs(scene: &SceneAsset) -> Vec<AssetNeed> {
    let mut out = Vec::new();
    for object in &scene.objects {
        collect_needs(object, 0, &mut out);
    }
    out
}

/// Every file one object (its prefab, its components and its children) loads.
pub fn object_needs(object: &Object) -> Vec<AssetNeed> {
    let mut out = Vec::new();
    collect_needs(object, 0, &mut out);
    out
}

/// A prefab that somehow holds itself would recurse for ever; stop as the editor's view does.
const MAX_PREFAB_DEPTH: usize = 8;

fn collect_needs(object: &Object, depth: usize, out: &mut Vec<AssetNeed>) {
    if depth > MAX_PREFAB_DEPTH {
        return;
    }
    if let Some(prefab) = object.prefab {
        collect_needs(&prefab().root, depth + 1, out);
    }
    for component in &object.components {
        for need in component.needs() {
            if !out.contains(&need) {
                out.push(need);
            }
        }
    }
    for child in &object.children {
        collect_needs(child, depth, out);
    }
}
