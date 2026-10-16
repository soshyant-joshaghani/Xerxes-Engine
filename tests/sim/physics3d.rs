//! 3D simulation core: a ball falls onto a floor; gravity can be changed.

use xerxes_sim::{BodyDef3d, Physics3d, Shape3d};

const DT: f32 = 1.0 / 60.0;

fn floor(world: &mut Physics3d) {
    world.add(BodyDef3d::fixed(
        Shape3d::Box {
            half: [50.0, 0.5, 50.0],
        },
        [0.0, -0.5, 0.0],
    ));
}

#[test]
fn a_ball_falls_and_rests_on_the_floor() {
    let mut world = Physics3d::new(DT, [0.0, -9.81, 0.0]);
    floor(&mut world);
    let ball = world.add(BodyDef3d::dynamic(
        Shape3d::Ball { radius: 0.5 },
        [0.0, 5.0, 0.0],
    ));
    for _ in 0..240 {
        world.step();
    }
    let at = world.position(ball);
    assert!(
        (at[1] - 0.5).abs() < 0.1,
        "resting on the floor: {at:?}"
    );
    assert!(world.velocity(ball)[1].abs() < 0.2);
}

#[test]
fn set_gravity_stops_a_fall() {
    let mut world = Physics3d::new(DT, [0.0, -9.81, 0.0]);
    floor(&mut world);
    let ball = world.add(BodyDef3d::dynamic(
        Shape3d::Ball { radius: 0.5 },
        [0.0, 3.0, 0.0],
    ));
    for _ in 0..30 {
        world.step();
    }
    let falling = world.position(ball)[1];
    world.set_gravity([0.0, 0.0, 0.0]);
    world.set_velocity(ball, [0.0, 0.0, 0.0]);
    for _ in 0..60 {
        world.step();
    }
    assert!(
        (world.position(ball)[1] - falling).abs() < 0.05,
        "held in place without gravity"
    );
}

#[test]
fn the_same_script_gives_the_same_hash() {
    let run = || {
        let mut world = Physics3d::new(DT, [0.0, -9.81, 0.0]);
        floor(&mut world);
        world.add(BodyDef3d::dynamic(
            Shape3d::Box {
                half: [0.5, 0.5, 0.5],
            },
            [0.0, 4.0, 0.0],
        ));
        for _ in 0..120 {
            world.step();
        }
        world.state_hash()
    };
    assert_eq!(run(), run());
}
