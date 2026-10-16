//! A 3D physics world over Rapier, same contract as [`crate::physics`]: plain data, fixed
//! step, deterministic mode, no clock and no Bevy. Units are the game's (metres for Cyrus).

use std::sync::mpsc;

use rapier3d::prelude::*;

use crate::physics::{Kind, Layers};

/// A point or a vector: `[x, y, z]`.
pub type V3 = [f32; 3];

fn vec(v: V3) -> Vector {
    Vector::new(v[0], v[1], v[2])
}

fn arr(v: Vector) -> V3 {
    [v.x, v.y, v.z]
}

/// A body of the 3D world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyId3d(RigidBodyHandle);

impl BodyId3d {
    fn key(self) -> (u32, u32) {
        self.0.into_raw_parts()
    }
}

impl PartialOrd for BodyId3d {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BodyId3d {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key().cmp(&other.key())
    }
}

/// The shape of a 3D body's collider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape3d {
    /// A box given by its half sizes (`[hx, hy, hz]`).
    Box { half: V3 },
    /// A ball (sphere).
    Ball { radius: f32 },
}

/// Everything needed to add a 3D body with one collider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyDef3d {
    pub kind: Kind,
    pub shape: Shape3d,
    pub position: V3,
    /// Unit quaternion `[x, y, z, w]`.
    pub rotation: [f32; 4],
    pub sensor: bool,
    pub layers: Layers,
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub lock_rotation: bool,
    pub fast: bool,
    pub gravity_scale: f32,
    pub user: u64,
}

impl BodyDef3d {
    pub fn new(kind: Kind, shape: Shape3d, position: V3) -> Self {
        Self {
            kind,
            shape,
            position,
            rotation: [0.0, 0.0, 0.0, 1.0],
            sensor: false,
            layers: Layers::ALL,
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            lock_rotation: false,
            fast: false,
            gravity_scale: 1.0,
            user: 0,
        }
    }

    pub fn fixed(shape: Shape3d, position: V3) -> Self {
        Self::new(Kind::Fixed, shape, position)
    }

    pub fn dynamic(shape: Shape3d, position: V3) -> Self {
        Self::new(Kind::Dynamic, shape, position)
    }

    pub fn sensor(mut self) -> Self {
        self.sensor = true;
        self
    }

    pub fn layers(mut self, layers: Layers) -> Self {
        self.layers = layers;
        self
    }

    pub fn user(mut self, user: u64) -> Self {
        self.user = user;
        self
    }
}

/// Two bodies started or stopped touching (3D world).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contact3d {
    pub a: BodyId3d,
    pub b: BodyId3d,
    pub started: bool,
    pub sensor: bool,
}

/// The 3D physics world.
pub struct Physics3d {
    world: PhysicsWorld,
    dt: f32,
    events: Vec<Contact3d>,
}

impl Physics3d {
    /// A world stepping `dt` seconds per [`step`](Self::step). Gravity `[0.0, -9.81, 0.0]` is
    /// Earth in metres.
    pub fn new(dt: f32, gravity: V3) -> Self {
        let mut world = PhysicsWorld::new();
        world.gravity = vec(gravity);
        world.integration_parameters.dt = dt;
        Self {
            world,
            dt,
            events: Vec::new(),
        }
    }

    pub fn dt(&self) -> f32 {
        self.dt
    }

    pub fn set_gravity(&mut self, gravity: V3) {
        self.world.gravity = vec(gravity);
    }

    pub fn add(&mut self, def: BodyDef3d) -> BodyId3d {
        // Rapier 0.36 uses glam: `Rotation` is a quaternion; builder.rotation takes axis-angle.
        let quat = Rotation::from_xyzw(
            def.rotation[0],
            def.rotation[1],
            def.rotation[2],
            def.rotation[3],
        );
        let builder = match def.kind {
            Kind::Fixed => RigidBodyBuilder::fixed(),
            Kind::Dynamic => RigidBodyBuilder::dynamic(),
            Kind::Kinematic => RigidBodyBuilder::kinematic_position_based(),
        }
        .translation(vec(def.position))
        .rotation(quat.to_scaled_axis())
        .linear_damping(def.linear_damping)
        .angular_damping(def.angular_damping)
        .gravity_scale(def.gravity_scale)
        .ccd_enabled(def.fast)
        .user_data(def.user as u128);
        let builder = if def.lock_rotation {
            builder.lock_rotations()
        } else {
            builder
        };
        let shape = match def.shape {
            Shape3d::Box { half } => ColliderBuilder::cuboid(half[0], half[1], half[2]),
            Shape3d::Ball { radius } => ColliderBuilder::ball(radius),
        };
        let groups = InteractionGroups::new(
            Group::from_bits_truncate(def.layers.member),
            Group::from_bits_truncate(def.layers.filter),
            InteractionTestMode::And,
        );
        let collider = shape
            .sensor(def.sensor)
            .collision_groups(groups)
            .friction(def.friction)
            .restitution(def.restitution)
            .density(def.density)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .active_collision_types(ActiveCollisionTypes::all());
        let (body, _) = self.world.insert(builder, collider);
        BodyId3d(body)
    }

    pub fn remove(&mut self, id: BodyId3d) {
        self.world.remove_body_with_colliders(id.0, true);
    }

    pub fn contains(&self, id: BodyId3d) -> bool {
        self.world.bodies.contains(id.0)
    }

    pub fn bodies(&self) -> Vec<BodyId3d> {
        let mut ids: Vec<BodyId3d> = self.world.bodies.iter().map(|(h, _)| BodyId3d(h)).collect();
        ids.sort();
        ids
    }

    pub fn step(&mut self) {
        let (sender, receiver) = mpsc::channel();
        let (force_sender, _force_receiver) = mpsc::channel();
        let (tear_sender, _tear_receiver) = mpsc::channel();
        let collector = ChannelEventCollector::new(sender, force_sender, tear_sender);
        self.world.step_with_events(&(), &collector);
        self.events.clear();
        for event in receiver.try_iter() {
            let (h1, h2, started, sensor) = match event {
                CollisionEvent::Started(a, b, flags) => {
                    (a, b, true, flags.contains(CollisionEventFlags::SENSOR))
                }
                CollisionEvent::Stopped(a, b, flags) => {
                    (a, b, false, flags.contains(CollisionEventFlags::SENSOR))
                }
            };
            let body = |h: ColliderHandle| self.world.colliders.get(h).and_then(|c| c.parent());
            if let (Some(a), Some(b)) = (body(h1), body(h2)) {
                self.events.push(Contact3d {
                    a: BodyId3d(a),
                    b: BodyId3d(b),
                    started,
                    sensor,
                });
            }
        }
        self.events.sort_by_key(|c| (c.a, c.b, !c.started));
    }

    pub fn contacts(&self) -> &[Contact3d] {
        &self.events
    }

    pub fn position(&self, id: BodyId3d) -> V3 {
        arr(self.world.bodies[id.0].translation())
    }

    /// Unit quaternion `[x, y, z, w]`.
    pub fn rotation(&self, id: BodyId3d) -> [f32; 4] {
        let q = *self.world.bodies[id.0].rotation();
        [q.x, q.y, q.z, q.w]
    }

    pub fn set_position(&mut self, id: BodyId3d, position: V3) {
        let body = &mut self.world.bodies[id.0];
        body.set_translation(vec(position), true);
        if body.is_kinematic() {
            body.set_next_kinematic_translation(vec(position));
        }
    }

    pub fn velocity(&self, id: BodyId3d) -> V3 {
        arr(self.world.bodies[id.0].linvel())
    }

    pub fn set_velocity(&mut self, id: BodyId3d, velocity: V3) {
        self.world.bodies[id.0].set_linvel(vec(velocity), true);
    }

    pub fn apply_impulse(&mut self, id: BodyId3d, impulse: V3) {
        self.world.bodies[id.0].apply_impulse(vec(impulse), true);
    }

    pub fn user(&self, id: BodyId3d) -> u64 {
        self.world.bodies[id.0].user_data as u64
    }

    /// First body a ray from `from` along `direction` hits within `max`.
    pub fn ray(&self, from: V3, direction: V3, max: f32) -> Option<(BodyId3d, f32)> {
        let ray = Ray::new(vec(from), vec(direction));
        let queries = self.world.query_pipeline();
        let (collider, toi) = queries.cast_ray(&ray, max, true)?;
        let body = self.world.colliders.get(collider)?.parent()?;
        Some((BodyId3d(body), toi))
    }

    pub fn state_hash(&self) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut mix = |value: u32| {
            for byte in value.to_le_bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(0x0100_0000_01b3);
            }
        };
        for id in self.bodies() {
            let body = &self.world.bodies[id.0];
            let at = body.translation();
            let speed = body.linvel();
            let q = *body.rotation();
            for value in [
                at.x, at.y, at.z, q.x, q.y, q.z, q.w, speed.x, speed.y, speed.z,
            ] {
                mix(value.to_bits());
            }
        }
        hash
    }
}
