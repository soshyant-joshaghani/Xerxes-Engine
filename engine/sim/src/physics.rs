//! A 2D physics world over Rapier, with the settings a game needs written as plain data.
//!
//! Units are the game's: world pixels, or metres, whichever the game picks, as long as it
//! sticks to one. Time is a fixed step ([`Physics2d::new`]); call [`Physics2d::step`] once per
//! tick. Rapier runs in its deterministic mode (see this crate's Cargo.toml), and the world
//! never reads the clock or a random number, so the same bodies and the same calls give the
//! same [`Physics2d::state_hash`] on every platform.

use std::sync::mpsc;

use rapier2d::control::{CharacterLength, KinematicCharacterController};
use rapier2d::prelude::*;

/// A point or a vector: `[x, y]`.
pub type V2 = [f32; 2];

fn vec(v: V2) -> Vector {
    Vector::new(v[0], v[1])
}

fn arr(v: Vector) -> V2 {
    [v.x, v.y]
}

/// A body of the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyId(RigidBodyHandle);

impl BodyId {
    fn key(self) -> (u32, u32) {
        self.0.into_raw_parts()
    }
}

impl PartialOrd for BodyId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Ids sort in the order the bodies were added (while none was removed in between).
impl Ord for BodyId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key().cmp(&other.key())
    }
}

/// How a body moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Never moves (walls, floors).
    Fixed,
    /// Moves under forces, gravity and contacts.
    Dynamic,
    /// Moved by the game ([`Physics2d::move_character`], [`Physics2d::set_velocity`]); pushes
    /// dynamic bodies, is never pushed.
    Kinematic,
}

/// The shape of a body's collider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// A box given by its half sizes.
    Box {
        half: V2,
    },
    Circle {
        radius: f32,
    },
}

/// Collision layers: a body is on the layers in `member` and touches bodies whose `member`
/// shares a bit with its `filter` (and the other way round).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layers {
    pub member: u32,
    pub filter: u32,
}

impl Layers {
    /// Layer 0, touches everything.
    pub const ALL: Layers = Layers {
        member: 1,
        filter: u32::MAX,
    };

    pub const fn new(member: u32, filter: u32) -> Self {
        Self { member, filter }
    }
}

impl Default for Layers {
    fn default() -> Self {
        Self::ALL
    }
}

/// Everything needed to add a body with one collider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyDef {
    pub kind: Kind,
    pub shape: Shape,
    pub position: V2,
    /// Radians, counter-clockwise.
    pub rotation: f32,
    /// A sensor reports overlaps ([`Contact`]) but does not push anything.
    pub sensor: bool,
    pub layers: Layers,
    pub friction: f32,
    /// Bounciness: 0 stops dead, 1 keeps all its speed.
    pub restitution: f32,
    pub density: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    /// A dynamic body that cannot spin (a top-down character, a puck).
    pub lock_rotation: bool,
    /// Continuous collision detection against other moving bodies: for small fast bodies.
    pub fast: bool,
    /// Gravity multiplier (0 for top-down games' bodies, 1 normal, less for low gravity).
    pub gravity_scale: f32,
    /// Yours: returned by [`Physics2d::user`], e.g. an entity or object id.
    pub user: u64,
}

impl BodyDef {
    pub fn new(kind: Kind, shape: Shape, position: V2) -> Self {
        Self {
            kind,
            shape,
            position,
            rotation: 0.0,
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

    pub fn fixed(shape: Shape, position: V2) -> Self {
        Self::new(Kind::Fixed, shape, position)
    }

    pub fn dynamic(shape: Shape, position: V2) -> Self {
        Self::new(Kind::Dynamic, shape, position)
    }

    pub fn kinematic(shape: Shape, position: V2) -> Self {
        Self::new(Kind::Kinematic, shape, position)
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

/// Two bodies started or stopped touching (or, with a sensor, overlapping).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contact {
    pub a: BodyId,
    pub b: BodyId,
    pub started: bool,
    /// One of the two is a sensor.
    pub sensor: bool,
}

/// What [`Physics2d::move_character`] did.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Moved {
    /// Where the body is now.
    pub position: V2,
    /// The part of the wish that was not blocked.
    pub moved: V2,
    pub grounded: bool,
}

/// The physics world.
pub struct Physics2d {
    world: PhysicsWorld,
    dt: f32,
    events: Vec<Contact>,
}

impl Physics2d {
    /// A world stepping `dt` seconds per [`step`](Self::step), with `gravity` (`[0.0, -9.81]`
    /// is Earth in metres; top-down games use `[0.0, 0.0]`).
    pub fn new(dt: f32, gravity: V2) -> Self {
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

    pub fn set_gravity(&mut self, gravity: V2) {
        self.world.gravity = vec(gravity);
    }

    /// Adds a body and returns its id.
    pub fn add(&mut self, def: BodyDef) -> BodyId {
        let builder = match def.kind {
            Kind::Fixed => RigidBodyBuilder::fixed(),
            Kind::Dynamic => RigidBodyBuilder::dynamic(),
            Kind::Kinematic => RigidBodyBuilder::kinematic_position_based(),
        }
        .translation(vec(def.position))
        .rotation(def.rotation)
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
            Shape::Box { half } => ColliderBuilder::cuboid(half[0], half[1]),
            Shape::Circle { radius } => ColliderBuilder::ball(radius),
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
        BodyId(body)
    }

    pub fn remove(&mut self, id: BodyId) {
        self.world.remove_body_with_colliders(id.0, true);
    }

    pub fn contains(&self, id: BodyId) -> bool {
        self.world.bodies.contains(id.0)
    }

    /// Every body, in the order they were added (stable, so two worlds agree).
    pub fn bodies(&self) -> Vec<BodyId> {
        let mut ids: Vec<BodyId> = self.world.bodies.iter().map(|(h, _)| BodyId(h)).collect();
        ids.sort();
        ids
    }

    /// Steps the world one tick. The contacts that started or stopped are in
    /// [`contacts`](Self::contacts) until the next step.
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
            // A removed body's colliders are gone: its last contacts are not reported.
            if let (Some(a), Some(b)) = (body(h1), body(h2)) {
                self.events.push(Contact {
                    a: BodyId(a),
                    b: BodyId(b),
                    started,
                    sensor,
                });
            }
        }
        // Same order everywhere, whatever order Rapier produced them in.
        self.events.sort_by_key(|c| (c.a, c.b, !c.started));
    }

    /// The contacts of the last step.
    pub fn contacts(&self) -> &[Contact] {
        &self.events
    }

    pub fn position(&self, id: BodyId) -> V2 {
        arr(self.world.bodies[id.0].translation())
    }

    pub fn rotation(&self, id: BodyId) -> f32 {
        self.world.bodies[id.0].rotation().angle()
    }

    /// Puts a body somewhere (a respawn). Its speed is kept; clear it with
    /// [`set_velocity`](Self::set_velocity).
    pub fn set_position(&mut self, id: BodyId, position: V2) {
        let body = &mut self.world.bodies[id.0];
        body.set_translation(vec(position), true);
        if body.is_kinematic() {
            body.set_next_kinematic_translation(vec(position));
        }
    }

    pub fn velocity(&self, id: BodyId) -> V2 {
        arr(self.world.bodies[id.0].linvel())
    }

    pub fn set_velocity(&mut self, id: BodyId, velocity: V2) {
        self.world.bodies[id.0].set_linvel(vec(velocity), true);
    }

    /// A sudden push (a hit, a dash).
    pub fn apply_impulse(&mut self, id: BodyId, impulse: V2) {
        self.world.bodies[id.0].apply_impulse(vec(impulse), true);
    }

    /// The `user` value the body was added with.
    pub fn user(&self, id: BodyId) -> u64 {
        self.world.bodies[id.0].user_data as u64
    }

    /// Moves a kinematic body by `wish`, sliding along whatever blocks it (walls, other
    /// bodies on layers it touches). The move is applied; it takes effect in the world at the
    /// next [`step`](Self::step), and `position` is already the new place.
    pub fn move_character(&mut self, id: BodyId, wish: V2) -> Moved {
        let body = &self.world.bodies[id.0];
        let start = *body.position();
        let Some(&collider) = body.colliders().first() else {
            return Moved {
                position: arr(start.translation),
                moved: [0.0, 0.0],
                grounded: false,
            };
        };
        let filter = QueryFilter::default()
            .exclude_rigid_body(id.0)
            .exclude_sensors();
        let queries = self.world.query_pipeline_with_filter(filter);
        let controller = KinematicCharacterController {
            offset: CharacterLength::Absolute(0.01),
            snap_to_ground: None,
            ..KinematicCharacterController::default()
        };
        let shape = self.world.colliders[collider].shape();
        let result = controller.move_shape(self.dt, &queries, shape, &start, vec(wish), |_| {});
        let to = start.translation + result.translation;
        let body = &mut self.world.bodies[id.0];
        body.set_next_kinematic_translation(to);
        body.set_translation(to, true);
        Moved {
            position: arr(to),
            moved: arr(result.translation),
            grounded: result.grounded,
        }
    }

    /// The first body a ray from `from` along `direction` hits within `max` distance.
    pub fn ray(&self, from: V2, direction: V2, max: f32) -> Option<(BodyId, f32)> {
        let ray = Ray::new(vec(from), vec(direction));
        let queries = self.world.query_pipeline();
        let (collider, toi) = queries.cast_ray(&ray, max, true)?;
        let body = self.world.colliders.get(collider)?.parent()?;
        Some((BodyId(body), toi))
    }

    /// A number that is the same for the same world state: positions, rotations and
    /// velocities of every body, in a fixed order. Replays and the server compare these.
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
            for value in [
                at.x,
                at.y,
                body.rotation().angle(),
                speed.x,
                speed.y,
                body.angvel(),
            ] {
                mix(value.to_bits());
            }
        }
        hash
    }
}
