//! 2D physics in Bevy: the `xerxes_sim` world ([`Physics2d`]) stepped on a fixed clock, with
//! entities as the bodies. The rules and the numbers are the sim crate's, so a client, the
//! multiplayer server and a replay agree; this module only connects them to entities.
//!
//! - Give an entity [`Body2d`] (in a scene, a prefab or code): it becomes a body at its
//!   `Transform` (a root entity: bodies are not parented) and, unless it is fixed, the `Transform` follows the body.
//! - A kinematic body with a [`Mover`] walks where the game's `wish` says, sliding along walls.
//! - Sensors and touching bodies send [`ContactEvent`] messages.
//! - Despawn the entity and its body goes with it.
//!
//! The plugin is optional: a game adds [`PhysicsPlugin`] from a logic module, so a game with no
//! physics (a visual novel, a puzzle) pays nothing.

use std::collections::HashMap;

use bevy::prelude::*;
use xerxes_sim::{BodyDef, BodyId, Kind, Physics2d, V2};

pub use xerxes_sim::{Contact, Layers, Shape};

/// The fixed physics rate, per second.
pub const TICKS_PER_SECOND: f64 = 60.0;

/// An entity that is a physics body: its shape and how it behaves. The position comes from the
/// entity's `Transform`, so `def.position` is ignored.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Body2d(pub BodyDef);

impl Body2d {
    pub fn fixed(shape: Shape) -> Self {
        Self(BodyDef::fixed(shape, [0.0, 0.0]))
    }

    pub fn dynamic(shape: Shape) -> Self {
        Self(BodyDef::dynamic(shape, [0.0, 0.0]))
    }

    pub fn kinematic(shape: Shape) -> Self {
        Self(BodyDef::kinematic(shape, [0.0, 0.0]))
    }

    pub fn sensor(mut self) -> Self {
        self.0.sensor = true;
        self
    }

    pub fn layers(mut self, layers: Layers) -> Self {
        self.0.layers = layers;
        self
    }
}

/// A box shape from its full size.
pub fn rect(size: Vec2) -> Shape {
    Shape::Box {
        half: [size.x / 2.0, size.y / 2.0],
    }
}

/// The body an entity has in the world (added by the plugin).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicsBody(pub BodyId);

/// A kinematic body that moves by itself: set `wish`, the displacement per second the game
/// wants (a top-down character's input times speed); the plugin moves it, blocked by walls.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Mover {
    pub wish: Vec2,
}

/// Two entities started or stopped touching (or overlapping, with a sensor).
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContactEvent {
    pub a: Entity,
    pub b: Entity,
    pub started: bool,
    pub sensor: bool,
}

impl ContactEvent {
    /// The entity that is not `entity`, if `entity` is one of the two.
    pub fn other(&self, entity: Entity) -> Option<Entity> {
        if self.a == entity {
            Some(self.b)
        } else if self.b == entity {
            Some(self.a)
        } else {
            None
        }
    }
}

/// The world, as a resource, for whatever needs to push or look (impulses, rays).
#[derive(Resource)]
pub struct PhysicsWorld {
    pub sim: Physics2d,
    entities: HashMap<BodyId, Entity>,
}

impl PhysicsWorld {
    pub fn entity(&self, body: BodyId) -> Option<Entity> {
        self.entities.get(&body).copied()
    }
}

/// Adds the physics world and its systems.
pub struct PhysicsPlugin {
    /// `Vec2::ZERO` for a top-down game, `(0, -980)` in pixels for a platformer.
    pub gravity: Vec2,
}

impl PhysicsPlugin {
    pub fn top_down() -> Self {
        Self {
            gravity: Vec2::ZERO,
        }
    }

    pub fn platformer(gravity: Vec2) -> Self {
        Self { gravity }
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        let dt = (1.0 / TICKS_PER_SECOND) as f32;
        app.insert_resource(Time::<Fixed>::from_hz(TICKS_PER_SECOND))
            .insert_resource(PhysicsWorld {
                sim: Physics2d::new(dt, [self.gravity.x, self.gravity.y]),
                entities: HashMap::new(),
            })
            .add_message::<ContactEvent>()
            .add_observer(remove_body)
            .add_systems(
                FixedUpdate,
                (add_bodies, move_movers, step_world, follow_bodies).chain(),
            );
    }
}

fn add_bodies(
    mut commands: Commands,
    mut world: ResMut<PhysicsWorld>,
    new: Query<(Entity, &Body2d, &Transform), Without<PhysicsBody>>,
) {
    for (entity, body, at) in &new {
        let mut def = body.0;
        def.position = [at.translation.x, at.translation.y];
        def.rotation = at.rotation.to_euler(EulerRot::XYZ).2;
        def.user = entity.to_bits();
        let id = world.sim.add(def);
        world.entities.insert(id, entity);
        commands.entity(entity).insert(PhysicsBody(id));
    }
}

fn remove_body(
    on: On<Remove, PhysicsBody>,
    bodies: Query<&PhysicsBody>,
    mut world: ResMut<PhysicsWorld>,
) {
    if let Ok(PhysicsBody(id)) = bodies.get(on.entity) {
        world.sim.remove(*id);
        world.entities.remove(id);
    }
}

fn move_movers(mut world: ResMut<PhysicsWorld>, movers: Query<(&PhysicsBody, &Mover)>) {
    let dt = world.sim.dt();
    for (PhysicsBody(id), mover) in &movers {
        let step: V2 = [mover.wish.x * dt, mover.wish.y * dt];
        if step != [0.0, 0.0] {
            world.sim.move_character(*id, step);
        }
    }
}

fn step_world(mut world: ResMut<PhysicsWorld>, mut contacts: MessageWriter<ContactEvent>) {
    world.sim.step();
    for c in world.sim.contacts() {
        if let (Some(a), Some(b)) = (world.entity(c.a), world.entity(c.b)) {
            contacts.write(ContactEvent {
                a,
                b,
                started: c.started,
                sensor: c.sensor,
            });
        }
    }
}

/// Entities follow their bodies (fixed ones stay where the scene put them).
fn follow_bodies(
    world: Res<PhysicsWorld>,
    mut bodies: Query<(&PhysicsBody, &Body2d, &mut Transform), Without<ChildOf>>,
) {
    for (PhysicsBody(id), body, mut transform) in &mut bodies {
        if body.0.kind == Kind::Fixed || !world.sim.contains(*id) {
            continue;
        }
        let at = world.sim.position(*id);
        transform.translation.x = at[0];
        transform.translation.y = at[1];
        transform.rotation = Quat::from_rotation_z(world.sim.rotation(*id));
    }
}
