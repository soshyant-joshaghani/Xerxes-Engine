//! Physics in Bevy over `xerxes_sim`: [`PhysicsPlugin`] / [`Body2d`] (2D) and
//! [`Physics3dPlugin`] / [`Body3d`] (3D). The rules and numbers live in the sim crate so a
//! client, the multiplayer server and a replay agree; this module only connects them to
//! entities.
//!
//! - 2D: [`Body2d`], [`Mover`], [`ContactEvent`], [`PhysicsPlugin`].
//! - 3D: [`Body3d`], [`ContactEvent3d`], [`Physics3dPlugin`] (see [`d3`]).
//!
//! The plugins are optional: a game adds one from a logic module.

mod d3;

pub use d3::{Body3d, ContactEvent3d, Physics3dPlugin, PhysicsBody3d, PhysicsWorld3d, Shape3d};

use std::collections::HashMap;

use bevy::math::Affine3A;
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

    /// A fixed box that fits the object's sprite (its `SpriteDef` size), so resizing the
    /// sprite resizes the collider. The shape is read when the body is created.
    pub fn fixed_fit() -> Self {
        Self::fixed(rect(Vec2::ZERO))
    }

    pub fn dynamic_fit() -> Self {
        Self::dynamic(rect(Vec2::ZERO))
    }

    pub fn kinematic_fit() -> Self {
        Self::kinematic(rect(Vec2::ZERO))
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

/// The transform an entity's parents give it (identity for a root). Walks the local transforms,
/// because the global ones are only computed later in the frame.
fn parent_world(
    entity: Entity,
    parents: &Query<&ChildOf>,
    locals: &Query<&Transform, Without<PhysicsBody>>,
) -> Affine3A {
    let mut world = Affine3A::IDENTITY;
    let mut current = entity;
    // Bounded: a hierarchy is shallow, and a cycle must not hang the step.
    for _ in 0..32 {
        let Ok(parent) = parents.get(current).map(ChildOf::parent) else {
            break;
        };
        if let Ok(local) = locals.get(parent) {
            world = local.compute_affine() * world;
        }
        current = parent;
    }
    world
}

fn add_bodies(
    mut commands: Commands,
    mut world: ResMut<PhysicsWorld>,
    new: Query<(Entity, &Body2d, &Transform, Option<&Sprite>), Without<PhysicsBody>>,
    parents: Query<&ChildOf>,
    locals: Query<&Transform, Without<PhysicsBody>>,
) {
    for (entity, body, at, sprite) in &new {
        let mut def = body.0;
        // A box of no size fits the sprite.
        if matches!(def.shape, Shape::Box { half } if half == [0.0, 0.0]) {
            let size = sprite
                .and_then(|s| s.custom_size)
                .unwrap_or(Vec2::splat(1.0));
            def.shape = Shape::Box {
                half: [size.x / 2.0, size.y / 2.0],
            };
        }
        let at = Transform::from_matrix(Mat4::from(
            parent_world(entity, &parents, &locals) * at.compute_affine(),
        ));
        def.position = [at.translation.x, at.translation.y];
        def.rotation = at.rotation.to_euler(EulerRot::XYZ).2;
        // The collider follows the object's scale (its sprite does too), so a player scaled up
        // in the editor is as big to the walls as it looks.
        let (sx, sy) = (at.scale.x.abs(), at.scale.y.abs());
        def.shape = match def.shape {
            Shape::Box { half } => Shape::Box {
                half: [half[0] * sx, half[1] * sy],
            },
            Shape::Circle { radius } => Shape::Circle {
                radius: radius * sx.max(sy),
            },
        };
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
    mut bodies: Query<(Entity, &PhysicsBody, &Body2d, &mut Transform)>,
    parents: Query<&ChildOf>,
    locals: Query<&Transform, Without<PhysicsBody>>,
) {
    for (entity, PhysicsBody(id), body, mut transform) in &mut bodies {
        if body.0.kind == Kind::Fixed || !world.sim.contains(*id) {
            continue;
        }
        let at = world.sim.position(*id);
        let rotation = world.sim.rotation(*id);
        // The body is in world space; the entity's transform is local to its parents.
        let to_local = parent_world(entity, &parents, &locals).inverse();
        let local = to_local.transform_point3(Vec3::new(at[0], at[1], transform.translation.z));
        transform.translation.x = local.x;
        transform.translation.y = local.y;
        transform.rotation = Quat::from_rotation_z(rotation);
    }
}
