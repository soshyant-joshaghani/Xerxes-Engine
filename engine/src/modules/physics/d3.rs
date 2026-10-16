//! 3D physics in Bevy: [`Physics3d`] stepped on a fixed clock, entities as bodies.
//! Pairidaeza (and later Atash) add [`Physics3dPlugin`].

use std::collections::HashMap;

use bevy::prelude::*;
use xerxes_sim::{BodyDef3d, BodyId3d, Kind, Physics3d, V3};

pub use xerxes_sim::Shape3d;

use super::TICKS_PER_SECOND;

/// An entity that is a 3D physics body. Position and rotation come from `Transform`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Body3d(pub BodyDef3d);

impl Body3d {
    pub fn fixed(shape: Shape3d) -> Self {
        Self(BodyDef3d::fixed(shape, [0.0, 0.0, 0.0]))
    }

    pub fn dynamic(shape: Shape3d) -> Self {
        Self(BodyDef3d::dynamic(shape, [0.0, 0.0, 0.0]))
    }

    pub fn box_half(half: Vec3) -> Shape3d {
        Shape3d::Box {
            half: [half.x, half.y, half.z],
        }
    }

    pub fn ball(radius: f32) -> Shape3d {
        Shape3d::Ball { radius }
    }
}

/// The body an entity has in the 3D world.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicsBody3d(pub BodyId3d);

/// Two entities started or stopped touching in the 3D world.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContactEvent3d {
    pub a: Entity,
    pub b: Entity,
    pub started: bool,
    pub sensor: bool,
}

/// The 3D world, as a resource.
#[derive(Resource)]
pub struct PhysicsWorld3d {
    pub sim: Physics3d,
    entities: HashMap<BodyId3d, Entity>,
}

impl PhysicsWorld3d {
    pub fn entity(&self, body: BodyId3d) -> Option<Entity> {
        self.entities.get(&body).copied()
    }
}

/// Adds the 3D physics world and its systems.
pub struct Physics3dPlugin {
    pub gravity: Vec3,
}

impl Physics3dPlugin {
    pub fn earth() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }

    pub fn with_gravity(gravity: Vec3) -> Self {
        Self { gravity }
    }
}

impl Plugin for Physics3dPlugin {
    fn build(&self, app: &mut App) {
        let dt = (1.0 / TICKS_PER_SECOND) as f32;
        let g: V3 = [self.gravity.x, self.gravity.y, self.gravity.z];
        app.insert_resource(Time::<Fixed>::from_hz(TICKS_PER_SECOND))
            .insert_resource(PhysicsWorld3d {
                sim: Physics3d::new(dt, g),
                entities: HashMap::new(),
            })
            .add_message::<ContactEvent3d>()
            .add_observer(remove_body)
            .add_systems(
                FixedUpdate,
                (add_bodies, step_world, follow_bodies).chain(),
            );
    }
}

fn add_bodies(
    mut commands: Commands,
    mut world: ResMut<PhysicsWorld3d>,
    new: Query<(Entity, &Body3d, &Transform), Without<PhysicsBody3d>>,
) {
    for (entity, body, at) in &new {
        let mut def = body.0;
        def.position = [at.translation.x, at.translation.y, at.translation.z];
        let q = at.rotation;
        def.rotation = [q.x, q.y, q.z, q.w];
        let (sx, sy, sz) = (
            at.scale.x.abs(),
            at.scale.y.abs(),
            at.scale.z.abs(),
        );
        def.shape = match def.shape {
            Shape3d::Box { half } => Shape3d::Box {
                half: [half[0] * sx, half[1] * sy, half[2] * sz],
            },
            Shape3d::Ball { radius } => Shape3d::Ball {
                radius: radius * sx.max(sy).max(sz),
            },
        };
        def.user = entity.to_bits();
        let id = world.sim.add(def);
        world.entities.insert(id, entity);
        commands.entity(entity).insert(PhysicsBody3d(id));
    }
}

fn remove_body(
    on: On<Remove, PhysicsBody3d>,
    bodies: Query<&PhysicsBody3d>,
    mut world: ResMut<PhysicsWorld3d>,
) {
    if let Ok(PhysicsBody3d(id)) = bodies.get(on.entity) {
        world.sim.remove(*id);
        world.entities.remove(id);
    }
}

fn step_world(mut world: ResMut<PhysicsWorld3d>, mut contacts: MessageWriter<ContactEvent3d>) {
    world.sim.step();
    for c in world.sim.contacts() {
        if let (Some(a), Some(b)) = (world.entity(c.a), world.entity(c.b)) {
            contacts.write(ContactEvent3d {
                a,
                b,
                started: c.started,
                sensor: c.sensor,
            });
        }
    }
}

fn follow_bodies(
    world: Res<PhysicsWorld3d>,
    mut bodies: Query<(&PhysicsBody3d, &Body3d, &mut Transform)>,
) {
    for (PhysicsBody3d(id), body, mut transform) in &mut bodies {
        if body.0.kind == Kind::Fixed || !world.sim.contains(*id) {
            continue;
        }
        let at = world.sim.position(*id);
        let q = world.sim.rotation(*id);
        transform.translation = Vec3::new(at[0], at[1], at[2]);
        transform.rotation = Quat::from_xyzw(q[0], q[1], q[2], q[3]);
    }
}
