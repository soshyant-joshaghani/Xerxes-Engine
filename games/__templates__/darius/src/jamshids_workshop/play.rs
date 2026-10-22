//! Jamshid's Workshop in Bevy: pointer drag on toys, spawn buttons, gravity slider, reset.

use bevy::input::ButtonState;
use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use bevy::window::WindowEvent;
use xerxes_engine::modules::flow::{CurrentLevel, Flow, LoadLevel, Play};
use xerxes_engine::modules::physics::{Body2d, PhysicsBody, PhysicsWorld, Shape, rect};
use xerxes_engine::modules::ui::{Slider, WidgetChanged};

/// Full Earth-like gravity in world pixels / s²; the slider scales it (LowGravity default).
pub const BASE_GRAVITY: f32 = 980.0;
/// Drop height for a freshly spawned toy.
const SPAWN_Y: f32 = 160.0;
/// Horizontal spawn jitter (±), world pixels.
const SPAWN_JITTER_X: f32 = 28.0;

const CRATE: Color = Color::srgb(0.72, 0.48, 0.28);
const BALL: Color = Color::srgb(0.30, 0.66, 0.92);

/// Marks the workbench level (the Game canvas). Systems only run while this is present.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Workbench;

/// A draggable dynamic body on the table.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Toy;

/// A round toy: drawn as a [`Mesh2d`] circle (sprites are always rectangles).
#[derive(Component, Debug, Clone, Copy)]
pub struct Ball {
    pub radius: f32,
}

/// The scene's camera, to turn the pointer into a world position.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LabCamera;

/// The gravity slider in the HUD.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GravitySlider;

/// Label next to the slider (`Gravity 0.35x`).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GravityLabel;

/// What a HUD spawn / reset button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkshopAction {
    Spawn(SpawnKind),
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnKind {
    Crate,
    Ball,
}

/// A HUD button.
#[derive(Component, Debug, Clone, Copy)]
pub struct WorkshopButton(pub WorkshopAction);

/// Active while the workbench scene is up.
#[derive(Resource)]
struct Lab {
    /// Entity being dragged, if any.
    held: Option<Entity>,
    /// Drop the menu click that opened the level.
    fresh: bool,
    /// LCG seed for spawn jitter (no `SystemTime` — it panics on the web).
    rng: u32,
}

pub struct WorkshopPlugin;

impl Plugin for WorkshopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(Flow::Level), teardown).add_systems(
            Update,
            (
                open_lab.run_if(not(resource_exists::<Lab>)),
                (
                    dress_balls,
                    apply_gravity.in_set(Play),
                    pointer_drag,
                    press_buttons,
                    refresh_gravity_label,
                )
                    .chain()
                    .run_if(resource_exists::<Lab>),
            )
                .chain()
                .run_if(in_state(Flow::Level)),
        );
    }
}

/// Sprites are boxes; balls need a circle mesh. Runs once per ball (scene starters keep a
/// `SpriteDef` for the editor; this replaces it when the level plays).
fn dress_balls(
    mut commands: Commands,
    balls: Query<(Entity, &Ball, Option<&Sprite>), (With<Toy>, Without<Mesh2d>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, ball, sprite) in &balls {
        let color = sprite.map(|s| s.color).unwrap_or(BALL);
        let mut entity = commands.entity(entity);
        entity.remove::<Sprite>().insert((
            Mesh2d(meshes.add(Circle::new(ball.radius))),
            MeshMaterial2d(materials.add(color)),
        ));
    }
}

fn teardown(mut commands: Commands, mut world: Option<ResMut<PhysicsWorld>>) {
    commands.remove_resource::<Lab>();
    // Other Darius games are top-down: leave the shared world with no gravity.
    if let Some(world) = world.as_mut() {
        world.sim.set_gravity([0.0, 0.0]);
    }
}

fn open_lab(
    mut commands: Commands,
    added: Query<(), Added<Workbench>>,
    slider: Query<&Slider, With<GravitySlider>>,
    mut world: Option<ResMut<PhysicsWorld>>,
) {
    if added.is_empty() {
        return;
    }
    let scale = slider.iter().next().map(|s| s.value).unwrap_or(0.35);
    if let Some(world) = world.as_mut() {
        world.sim.set_gravity([0.0, -BASE_GRAVITY * scale]);
    }
    commands.insert_resource(Lab {
        held: None,
        fresh: true,
        rng: 0xA5A5_C0DE,
    });
}

/// Next drop point: small random X so toys do not stack.
fn next_spawn_at(lab: &mut Lab) -> Vec2 {
    lab.rng = lab.rng.wrapping_mul(1664525).wrapping_add(1013904223);
    let u = (lab.rng >> 8) as f32 / (u32::MAX >> 8) as f32;
    let x = (u * 2.0 - 1.0) * SPAWN_JITTER_X;
    Vec2::new(x, SPAWN_Y)
}

fn apply_gravity(
    mut changed: MessageReader<WidgetChanged>,
    sliders: Query<&Slider, With<GravitySlider>>,
    mut world: ResMut<PhysicsWorld>,
) {
    for WidgetChanged(entity) in changed.read() {
        let Ok(slider) = sliders.get(*entity) else {
            continue;
        };
        world.sim.set_gravity([0.0, -BASE_GRAVITY * slider.value]);
    }
}

fn refresh_gravity_label(
    slider: Query<&Slider, (With<GravitySlider>, Changed<Slider>)>,
    mut label: Query<&mut Text, With<GravityLabel>>,
) {
    let Ok(slider) = slider.single() else {
        return;
    };
    for mut text in &mut label {
        text.0 = format!("Gravity {:.2}x", slider.value);
    }
}

/// Pick and drag a toy. While held, the body follows the pointer with zero velocity.
fn pointer_drag(
    mut lab: ResMut<Lab>,
    mut events: MessageReader<WindowEvent>,
    camera: Query<(&Camera, &GlobalTransform), With<LabCamera>>,
    toys: Query<(Entity, &GlobalTransform, Option<&Sprite>, Option<&Ball>), With<Toy>>,
    bodies: Query<&PhysicsBody, With<Toy>>,
    ui: Query<&Interaction, With<Button>>,
    mut world: ResMut<PhysicsWorld>,
    mut pointer: Local<Option<Vec2>>,
) {
    if lab.fresh {
        lab.fresh = false;
        events.clear();
        return;
    }
    let Ok((camera, camera_at)) = camera.single() else {
        return;
    };
    let over_ui = ui.iter().any(|i| *i != Interaction::None);
    let to_world = |screen: Vec2| camera.viewport_to_world_2d(camera_at, screen).ok();

    for event in events.read() {
        match event {
            WindowEvent::CursorMoved(moved) => {
                *pointer = Some(moved.position);
            }
            WindowEvent::MouseButtonInput(input) if input.button == MouseButton::Left => {
                if input.state == ButtonState::Released {
                    lab.held = None;
                } else if input.state == ButtonState::Pressed && !over_ui {
                    if let Some(screen) = *pointer {
                        if let Some(at) = to_world(screen) {
                            lab.held = toy_at(&toys, at);
                        }
                    }
                }
            }
            // Bevy's window event (not winit's `Touch`).
            WindowEvent::TouchInput(touch) => {
                *pointer = Some(touch.position);
                match touch.phase {
                    TouchPhase::Started => {
                        if !over_ui {
                            if let Some(at) = to_world(touch.position) {
                                lab.held = toy_at(&toys, at);
                            }
                        }
                    }
                    TouchPhase::Moved => {}
                    TouchPhase::Ended | TouchPhase::Canceled => {
                        lab.held = None;
                    }
                }
            }
            _ => {}
        }
    }

    let Some(held) = lab.held else {
        return;
    };
    let Some(screen) = *pointer else {
        return;
    };
    let Some(at) = to_world(screen) else {
        return;
    };
    let Ok(PhysicsBody(id)) = bodies.get(held) else {
        lab.held = None;
        return;
    };
    if !world.sim.contains(*id) {
        lab.held = None;
        return;
    }
    world.sim.set_position(*id, [at.x, at.y]);
    world.sim.set_velocity(*id, [0.0, 0.0]);
}

fn toy_at(
    toys: &Query<(Entity, &GlobalTransform, Option<&Sprite>, Option<&Ball>), With<Toy>>,
    at: Vec2,
) -> Option<Entity> {
    toys.iter().find_map(|(entity, transform, sprite, ball)| {
        let pos = transform.translation().truncate();
        let scale = transform.compute_transform().scale.truncate().abs();
        if let Some(ball) = ball {
            let radius = ball.radius * scale.x.max(scale.y);
            return (at.distance(pos) <= radius).then_some(entity);
        }
        let half = sprite
            .and_then(|s| s.custom_size)
            .unwrap_or(Vec2::splat(24.0))
            * 0.5
            * scale;
        let d = (at - pos).abs();
        (d.x <= half.x && d.y <= half.y).then_some(entity)
    })
}

fn press_buttons(
    mut commands: Commands,
    interactions: Query<(&Interaction, &WorkshopButton), Changed<Interaction>>,
    workbench: Query<Entity, With<Workbench>>,
    mut lab: ResMut<Lab>,
    level: Res<CurrentLevel>,
    mut load: MessageWriter<LoadLevel>,
) {
    let Ok(parent) = workbench.single() else {
        return;
    };
    for (interaction, button) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button.0 {
            WorkshopAction::Spawn(kind) => {
                let at = next_spawn_at(&mut lab);
                spawn_toy(&mut commands, parent, kind, at);
            }
            WorkshopAction::Reset => {
                load.write(LoadLevel(level.0));
            }
        }
    }
}

fn spawn_toy(commands: &mut Commands, parent: Entity, kind: SpawnKind, at: Vec2) {
    commands.entity(parent).with_children(|bench| {
        match kind {
            SpawnKind::Crate => {
                bench.spawn((
                    Toy,
                    Sprite::from_color(CRATE, Vec2::new(40.0, 40.0)),
                    Transform::from_xyz(at.x, at.y, 0.5),
                    Body2d::dynamic(rect(Vec2::new(40.0, 40.0))),
                ));
            }
            SpawnKind::Ball => {
                // `dress_balls` adds the circle mesh; no sprite (would look square).
                bench.spawn((
                    Toy,
                    Ball { radius: 18.0 },
                    Transform::from_xyz(at.x, at.y, 0.5),
                    Body2d::dynamic(Shape::Circle { radius: 18.0 }),
                ));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use xerxes_engine::modules::assets::spawn_scene;
    use xerxes_engine::modules::ui::Slider;

    #[test]
    fn the_scene_is_a_workbench_with_toys_and_a_gravity_slider() {
        let mut world = World::new();
        spawn_scene(
            &mut world,
            &crate::assets::scenes::games::outside::jamshids_workshop_scene::scene(),
        );
        assert_eq!(world.query::<&Workbench>().iter(&world).count(), 1);
        assert_eq!(world.query::<&LabCamera>().iter(&world).count(), 1);
        assert!(
            world.query::<&Toy>().iter(&world).count() >= 2,
            "starter crate and ball"
        );
        assert_eq!(world.query::<&Ball>().iter(&world).count(), 1);
        let slider = world
            .query::<&Slider>()
            .iter(&world)
            .find(|_| true)
            .expect("gravity slider");
        assert!((slider.value - 0.35).abs() < 0.001);
        assert_eq!(
            world.query::<&WorkshopButton>().iter(&world).count(),
            3,
            "crate, ball, reset"
        );
        assert_eq!(world.query::<&GravityLabel>().iter(&world).count(), 1);
    }

    #[test]
    fn gravity_scale_maps_to_world_gravity() {
        assert!((BASE_GRAVITY * 0.35 - 343.0).abs() < 0.01);
    }
}
