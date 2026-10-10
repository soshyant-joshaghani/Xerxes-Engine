//! The physics module in Bevy: a `Body2d` entity becomes a body and follows it, a `Mover`
//! walks and is stopped by a wall, a sensor sends contact messages, despawning removes the body.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use xerxes_engine::modules::physics::{
    Body2d, ContactEvent, Mover, PhysicsPlugin, PhysicsWorld, Shape, rect,
};

fn app(gravity: Vec2) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_micros(
            16_667,
        )))
        .add_plugins(PhysicsPlugin { gravity });
    // The first updates only set the clock going.
    app.update();
    app
}

fn run(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

#[test]
fn a_dynamic_entity_falls_onto_a_fixed_one() {
    let mut app = app(Vec2::new(0.0, -980.0));
    app.world_mut().spawn((
        Body2d::fixed(rect(Vec2::new(2000.0, 20.0))),
        Transform::from_xyz(0.0, -10.0, 0.0),
    ));
    let ball = app
        .world_mut()
        .spawn((
            Body2d::dynamic(Shape::Circle { radius: 10.0 }),
            Transform::from_xyz(0.0, 200.0, 0.0),
        ))
        .id();
    run(&mut app, 240);
    let y = app.world().get::<Transform>(ball).unwrap().translation.y;
    assert!((y - 10.0).abs() < 1.0, "resting on the floor, y = {y}");
}

#[test]
fn a_mover_is_stopped_by_a_wall_and_a_sensor_reports() {
    let mut app = app(Vec2::ZERO);
    app.world_mut().spawn((
        Body2d::fixed(rect(Vec2::new(20.0, 400.0))),
        Transform::from_xyz(200.0, 0.0, 0.0),
    ));
    let coin = app
        .world_mut()
        .spawn((
            Body2d::fixed(rect(Vec2::splat(14.0))).sensor(),
            Transform::from_xyz(100.0, 0.0, 0.0),
        ))
        .id();
    let player = app
        .world_mut()
        .spawn((
            Body2d::kinematic(rect(Vec2::splat(28.0))),
            Mover {
                wish: Vec2::new(260.0, 0.0),
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();
    let mut touched = false;
    for _ in 0..180 {
        app.update();
        let messages = app.world().resource::<Messages<ContactEvent>>();
        let mut cursor = messages.get_cursor();
        for c in cursor.read(messages) {
            if c.started && c.sensor && c.other(player) == Some(coin) {
                touched = true;
            }
        }
    }
    let x = app.world().get::<Transform>(player).unwrap().translation.x;
    assert!(touched, "the player touched the coin");
    assert!(
        x > 150.0 && x < 180.0,
        "stopped by the wall at 190 - 14: x = {x}"
    );
}

#[test]
fn despawning_removes_the_body() {
    let mut app = app(Vec2::ZERO);
    let entity = app
        .world_mut()
        .spawn((
            Body2d::dynamic(Shape::Circle { radius: 5.0 }),
            Transform::default(),
        ))
        .id();
    run(&mut app, 3);
    assert_eq!(app.world().resource::<PhysicsWorld>().sim.bodies().len(), 1);
    app.world_mut().entity_mut(entity).despawn();
    run(&mut app, 2);
    assert_eq!(app.world().resource::<PhysicsWorld>().sim.bodies().len(), 0);
}
