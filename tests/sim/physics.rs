//! The simulation core's physics: bodies fall and rest, layers filter, sensors report, the
//! character mover slides, and the same script gives the same state every time.

use xerxes_sim::{BodyDef, Layers, Physics2d, Shape};

const DT: f32 = 1.0 / 60.0;

fn floor(world: &mut Physics2d) {
    world.add(BodyDef::fixed(
        Shape::Box { half: [50.0, 1.0] },
        [0.0, -1.0],
    ));
}

#[test]
fn a_ball_falls_and_rests_on_the_floor() {
    let mut world = Physics2d::new(DT, [0.0, -9.81]);
    floor(&mut world);
    let ball = world.add(BodyDef::dynamic(Shape::Circle { radius: 0.5 }, [0.0, 5.0]));
    for _ in 0..240 {
        world.step();
    }
    let at = world.position(ball);
    assert!((at[1] - 0.5).abs() < 0.05, "resting on the floor: {at:?}");
    assert!(world.velocity(ball)[1].abs() < 0.1);
}

#[test]
fn without_gravity_a_body_keeps_its_speed_and_an_impulse_moves_it() {
    let mut world = Physics2d::new(DT, [0.0, 0.0]);
    let puck = world.add(BodyDef::dynamic(Shape::Circle { radius: 0.5 }, [0.0, 0.0]));
    world.set_velocity(puck, [3.0, 0.0]);
    for _ in 0..60 {
        world.step();
    }
    assert!((world.position(puck)[0] - 3.0).abs() < 0.05);
    world.apply_impulse(puck, [0.0, 2.0]);
    world.step();
    assert!(world.velocity(puck)[1] > 0.5);
}

#[test]
fn layers_decide_who_touches_whom() {
    const RED: u32 = 0b01;
    const BLUE: u32 = 0b10;
    let mut world = Physics2d::new(DT, [0.0, -9.81]);
    floor(&mut world);
    // Blue does not collide with the floor's layer 0 only if its filter leaves it out.
    let ghost = world.add(
        BodyDef::dynamic(Shape::Circle { radius: 0.5 }, [0.0, 2.0]).layers(Layers::new(BLUE, BLUE)),
    );
    let solid = world.add(
        BodyDef::dynamic(Shape::Circle { radius: 0.5 }, [5.0, 2.0])
            .layers(Layers::new(RED, u32::MAX)),
    );
    for _ in 0..240 {
        world.step();
    }
    assert!(world.position(ghost)[1] < -5.0, "fell through the floor");
    assert!(world.position(solid)[1] > 0.0, "rests on the floor");
}

#[test]
fn a_sensor_reports_overlaps_without_pushing() {
    let mut world = Physics2d::new(DT, [0.0, 0.0]);
    let zone = world.add(
        BodyDef::fixed(Shape::Box { half: [1.0, 1.0] }, [5.0, 0.0])
            .sensor()
            .user(7),
    );
    let player = world.add(BodyDef::dynamic(Shape::Circle { radius: 0.5 }, [0.0, 0.0]).user(1));
    world.set_velocity(player, [5.0, 0.0]);
    let (mut entered, mut left) = (false, false);
    for _ in 0..120 {
        world.step();
        for c in world.contacts() {
            assert!(c.sensor);
            let pair = [world.user(c.a), world.user(c.b)];
            assert!(pair.contains(&7) && pair.contains(&1));
            if c.started {
                entered = true;
            } else {
                left = true;
            }
        }
    }
    assert!(entered && left, "entered {entered}, left {left}");
    assert!(
        world.position(player)[0] > 8.0,
        "the sensor did not stop it"
    );
    assert!(world.contains(zone));
}

#[test]
fn a_character_slides_along_a_wall() {
    let mut world = Physics2d::new(DT, [0.0, 0.0]);
    world.add(BodyDef::fixed(Shape::Box { half: [0.5, 5.0] }, [3.0, 0.0]));
    let player = world.add(BodyDef::kinematic(
        Shape::Box { half: [0.5, 0.5] },
        [0.0, 0.0],
    ));
    // Diagonal into the wall: x is stopped before the wall, y still moves.
    for _ in 0..30 {
        world.move_character(player, [0.2, 0.1]);
        world.step();
    }
    let at = world.position(player);
    assert!(at[0] < 2.1, "stopped by the wall: {at:?}");
    assert!(at[0] > 1.9, "reached the wall: {at:?}");
    assert!(at[1] > 2.5, "slid along it: {at:?}");
}

#[test]
fn a_ray_finds_the_first_body() {
    let mut world = Physics2d::new(DT, [0.0, 0.0]);
    let near = world.add(BodyDef::fixed(Shape::Box { half: [1.0, 1.0] }, [5.0, 0.0]));
    world.add(BodyDef::fixed(Shape::Box { half: [1.0, 1.0] }, [9.0, 0.0]));
    world.step();
    let (hit, distance) = world.ray([0.0, 0.0], [1.0, 0.0], 100.0).unwrap();
    assert_eq!(hit, near);
    assert!((distance - 4.0).abs() < 0.01);
    assert!(world.ray([0.0, 0.0], [0.0, 1.0], 100.0).is_none());
}

fn script() -> Physics2d {
    let mut world = Physics2d::new(DT, [0.0, -9.81]);
    floor(&mut world);
    for i in 0..10 {
        let mut def = BodyDef::dynamic(
            Shape::Box { half: [0.4, 0.4] },
            [i as f32 * 0.3, 1.0 + i as f32],
        );
        def.restitution = 0.3;
        world.add(def);
        let ball = world.add(BodyDef::dynamic(
            Shape::Circle { radius: 0.3 },
            [0.1 + i as f32 * 0.25, 0.5 + i as f32],
        ));
        world.apply_impulse(ball, [0.2 * i as f32, 0.0]);
    }
    for _ in 0..300 {
        world.step();
    }
    world
}

#[test]
fn the_same_script_gives_the_same_state() {
    let a = script();
    let b = script();
    assert_eq!(a.state_hash(), b.state_hash());
    // And it is a real number: a different script differs.
    let mut c = script();
    let first = c.bodies()[1];
    c.apply_impulse(first, [1.0, 0.0]);
    c.step();
    assert_ne!(a.state_hash(), c.state_hash());
}

#[test]
fn a_moved_character_triggers_a_fixed_sensor() {
    let mut world = Physics2d::new(DT, [0.0, 0.0]);
    let coin = world.add(BodyDef::fixed(Shape::Box { half: [7.0, 7.0] }, [100.0, 0.0]).sensor());
    let player = world.add(BodyDef::kinematic(
        Shape::Box { half: [14.0, 14.0] },
        [0.0, 0.0],
    ));
    let mut touched = false;
    for _ in 0..120 {
        world.move_character(player, [4.0, 0.0]);
        world.step();
        touched |= world
            .contacts()
            .iter()
            .any(|c| c.started && c.sensor && (c.a == coin || c.b == coin));
    }
    assert!(touched);
}

/// Darius's arena: a 800 x 500 room (20 thick walls) with two pillars and two bars.
fn arena(world: &mut Physics2d) {
    for (x, y, w, h) in [
        (0.0, 240.0, 800.0, 20.0),
        (0.0, -240.0, 800.0, 20.0),
        (-390.0, 0.0, 20.0, 500.0),
        (390.0, 0.0, 20.0, 500.0),
        (-150.0, 60.0, 40.0, 220.0),
        (150.0, -60.0, 40.0, 220.0),
        (0.0, 170.0, 160.0, 30.0),
        (280.0, 150.0, 100.0, 30.0),
    ] {
        world.add(BodyDef::fixed(
            Shape::Box {
                half: [w / 2.0, h / 2.0],
            },
            [x, y],
        ));
    }
}

/// How deep a box centred at `at` with half size `half` is inside the arena's walls.
fn overlap_with_walls(at: [f32; 2], half: f32) -> f32 {
    let walls = [
        (0.0, 240.0, 400.0, 10.0),
        (0.0, -240.0, 400.0, 10.0),
        (-390.0, 0.0, 10.0, 250.0),
        (390.0, 0.0, 10.0, 250.0),
        (-150.0, 60.0, 20.0, 110.0),
        (150.0, -60.0, 20.0, 110.0),
        (0.0, 170.0, 80.0, 15.0),
        (280.0, 150.0, 50.0, 15.0),
    ];
    let mut worst = 0.0f32;
    for (x, y, hx, hy) in walls {
        let dx = (hx + half) - (at[0] - x).abs();
        let dy = (hy + half) - (at[1] - y).abs();
        if dx > 0.0 && dy > 0.0 {
            worst = worst.max(dx.min(dy));
        }
    }
    worst
}

/// The way the game drives the player: every tick a move of `wish * dt`, then a step.
fn drive(
    world: &mut Physics2d,
    id: xerxes_sim::BodyId,
    wish: [f32; 2],
    ticks: usize,
    half: f32,
) -> f32 {
    let mut worst = 0.0f32;
    for _ in 0..ticks {
        world.move_character(id, [wish[0] * DT, wish[1] * DT]);
        world.step();
        worst = worst.max(overlap_with_walls(world.position(id), half));
    }
    worst
}

#[test]
fn a_player_pushed_into_a_wall_while_sliding_along_it_never_overlaps_it() {
    for size in [28.0f32, 56.0] {
        let half = size / 2.0;
        let mut world = Physics2d::new(DT, [0.0, 0.0]);
        arena(&mut world);
        world.step();
        let player = world.add(BodyDef::kinematic(
            Shape::Box { half: [half, half] },
            [-340.0, -190.0],
        ));
        world.step();
        // Down and right along the south wall, then up and left along the north one.
        let mut worst = drive(&mut world, player, [260.0, -260.0], 240, half);
        worst = worst.max(drive(&mut world, player, [-260.0, 260.0], 240, half));
        worst = worst.max(drive(&mut world, player, [260.0, 100.0], 240, half));
        assert!(worst < 0.05, "size {size}: overlapped a wall by {worst}");
    }
}
