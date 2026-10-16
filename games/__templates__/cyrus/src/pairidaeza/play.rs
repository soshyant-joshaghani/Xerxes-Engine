//! Pairidaeza physics lab: Grab (ray-drag toys) or Camera (orbit / pan / zoom).
//! Camera controls mirror the editor Scene viewport (`editor/stage/camera.rs` + `touch.rs`).

use bevy::input::ButtonState;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input::touch::{TouchPhase, Touches};
use bevy::prelude::*;
use bevy::window::WindowEvent;
use xerxes_engine::modules::flow::{CurrentLevel, Flow, LoadLevel};
use xerxes_engine::modules::physics::{Body3d, PhysicsBody3d, PhysicsWorld3d};
use xerxes_engine::modules::ui::{Slider, WidgetChanged};

/// Full Earth gravity (m/s²); the slider scales it.
pub const BASE_GRAVITY: f32 = 9.81;
const SPAWN_Y: f32 = 2.5;
/// Horizontal spawn jitter (±), metres.
const SPAWN_JITTER_X: f32 = 0.55;
const CUBE: Color = Color::srgb(0.72, 0.48, 0.28);
const BALL: Color = Color::srgb(0.30, 0.66, 0.92);

/// Same as `EditorCamera` mouse look in `control_camera`.
const LOOK: f32 = 0.004;
/// Same as `touch_camera` one-finger orbit.
const LOOK_TOUCH: f32 = 0.006;
const ACCENT: Color = Color::srgb(0.22, 0.741, 0.973);
const ACCENT_TEXT: Color = Color::srgb(0.043, 0.071, 0.125);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);

/// Marks the Pairidaeza plot (systems only run while present).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Plot;

/// A draggable dynamic body.
#[derive(Component, Debug, Clone, Copy)]
pub struct Toy {
    pub kind: ToyKind,
    pub half_extent: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToyKind {
    Cube,
    Ball,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LabCamera;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GravitySlider;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GravityLabel;

/// Label on the Grab / Camera toggle.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ModeLabel;

/// Hint line under the title.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HintText;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    Grab,
    Camera,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkshopAction {
    Spawn(SpawnKind),
    Reset,
    ToggleMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnKind {
    Cube,
    Ball,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WorkshopButton(pub WorkshopAction);

/// Orbit rig — same fields / math as `EditorCamera` (3D path).
#[derive(Debug, Clone, Copy)]
struct Orbit {
    pivot: Vec3,
    distance: f32,
    yaw: f32,
    pitch: f32,
}

impl Orbit {
    /// Scene default: eye (6, 7, 9) → pivot (0, 0.5, 0).
    /// Pitch sign matches the editor: negative = above the pivot looking down.
    fn scene_default() -> Self {
        let eye = Vec3::new(6.0, 7.0, 9.0);
        let pivot = Vec3::new(0.0, 0.5, 0.0);
        let offset = eye - pivot;
        let distance = offset.length().max(0.5);
        Self {
            pivot,
            distance,
            yaw: offset.x.atan2(offset.z),
            pitch: (-offset.y / distance).asin().clamp(-1.54, 1.54),
        }
    }

    fn rotation(self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }

    fn position(self) -> Vec3 {
        self.pivot - self.rotation() * Vec3::NEG_Z * self.distance
    }

    fn transform(self) -> Transform {
        Transform::from_translation(self.position()).with_rotation(self.rotation())
    }
}

#[derive(Resource)]
struct Lab {
    mode: ToolMode,
    orbit: Orbit,
    /// Mouse button currently driving the camera (`SceneInput.dragging`).
    mouse_drag: Option<MouseButton>,
    /// Touch drag started off the HUD (`TouchPointer.on_scene`).
    touch_on_scene: bool,
    held: Option<Entity>,
    /// Y of the plane the held toy slides on.
    drag_y: f32,
    fresh: bool,
    /// LCG seed for spawn jitter.
    rng: u32,
}

pub struct PairidaezaPlugin;

impl Plugin for PairidaezaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(Flow::Level), teardown)
            .add_systems(
                Update,
                (
                    open_lab.run_if(not(resource_exists::<Lab>)),
                    (
                        apply_gravity,
                        press_buttons,
                        refresh_hud,
                        pointer_input,
                        // Same split as the editor: mouse in control_camera, fingers in touch_camera.
                        control_camera,
                        touch_camera,
                    )
                        .chain()
                        .run_if(resource_exists::<Lab>),
                )
                    .chain()
                    .run_if(in_state(Flow::Level)),
            );
    }
}

fn teardown(mut commands: Commands, mut world: Option<ResMut<PhysicsWorld3d>>) {
    commands.remove_resource::<Lab>();
    if let Some(world) = world.as_mut() {
        world.sim.set_gravity([0.0, -BASE_GRAVITY, 0.0]);
    }
}

fn open_lab(
    mut commands: Commands,
    added: Query<(), Added<Plot>>,
    slider: Query<&Slider, With<GravitySlider>>,
    mut world: Option<ResMut<PhysicsWorld3d>>,
) {
    if added.is_empty() {
        return;
    }
    let scale = slider.iter().next().map(|s| s.value).unwrap_or(1.0);
    if let Some(world) = world.as_mut() {
        world
            .sim
            .set_gravity([0.0, -BASE_GRAVITY * scale, 0.0]);
    }
    commands.insert_resource(Lab {
        mode: ToolMode::Grab,
        orbit: Orbit::scene_default(),
        mouse_drag: None,
        touch_on_scene: false,
        held: None,
        drag_y: 0.5,
        fresh: true,
        rng: 0xC0FF_EE42,
    });
}

/// Next drop point: small random X so toys do not stack.
fn next_spawn_at(lab: &mut Lab) -> Vec3 {
    lab.rng = lab.rng.wrapping_mul(1664525).wrapping_add(1013904223);
    let u = (lab.rng >> 8) as f32 / (u32::MAX >> 8) as f32;
    let x = (u * 2.0 - 1.0) * SPAWN_JITTER_X;
    Vec3::new(x, SPAWN_Y, 0.0)
}

fn apply_gravity(
    mut changed: MessageReader<WidgetChanged>,
    sliders: Query<&Slider, With<GravitySlider>>,
    mut world: ResMut<PhysicsWorld3d>,
) {
    for WidgetChanged(entity) in changed.read() {
        let Ok(slider) = sliders.get(*entity) else {
            continue;
        };
        world
            .sim
            .set_gravity([0.0, -BASE_GRAVITY * slider.value, 0.0]);
    }
}

fn refresh_hud(
    lab: Res<Lab>,
    mut mode_label: Query<&mut Text, (With<ModeLabel>, Without<HintText>, Without<GravityLabel>)>,
    mut mode_color: Query<&mut TextColor, With<ModeLabel>>,
    mut hint: Query<&mut Text, (With<HintText>, Without<ModeLabel>, Without<GravityLabel>)>,
    mut mode_bg: Query<(&WorkshopButton, &mut BackgroundColor)>,
    slider: Query<&Slider, (With<GravitySlider>, Changed<Slider>)>,
    mut gravity_label: Query<
        &mut Text,
        (With<GravityLabel>, Without<ModeLabel>, Without<HintText>),
    >,
) {
    if lab.is_changed() {
        let (name, tip, fill, ink) = match lab.mode {
            ToolMode::Grab => (
                "Grab",
                "Grab: drag toys  |  Camera: orbit / pan / zoom  |  Spawn  |  Gravity",
                GHOST,
                TEXT,
            ),
            ToolMode::Camera => (
                "Camera",
                "Camera: drag orbit, 2-finger pan/pinch, wheel zoom  |  toggle to Grab",
                ACCENT,
                ACCENT_TEXT,
            ),
        };
        for mut text in &mut mode_label {
            text.0 = name.into();
        }
        for mut color in &mut mode_color {
            color.0 = ink;
        }
        for mut text in &mut hint {
            text.0 = tip.into();
        }
        for (button, mut bg) in &mut mode_bg {
            if matches!(button.0, WorkshopAction::ToggleMode) {
                bg.0 = fill;
            }
        }
    }
    if let Ok(slider) = slider.single() {
        for mut text in &mut gravity_label {
            text.0 = format!("Gravity {:.2}x", slider.value);
        }
    }
}

fn press_buttons(
    mut commands: Commands,
    interactions: Query<(&Interaction, &WorkshopButton), Changed<Interaction>>,
    plot: Query<Entity, With<Plot>>,
    mut lab: ResMut<Lab>,
    level: Res<CurrentLevel>,
    mut load: MessageWriter<LoadLevel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(parent) = plot.single() else {
        return;
    };
    for (interaction, button) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button.0 {
            WorkshopAction::Spawn(kind) => {
                let at = next_spawn_at(&mut lab);
                spawn_toy(&mut commands, parent, kind, at, &mut meshes, &mut materials);
            }
            WorkshopAction::Reset => {
                load.write(LoadLevel(level.0));
            }
            WorkshopAction::ToggleMode => {
                lab.mode = match lab.mode {
                    ToolMode::Grab => ToolMode::Camera,
                    ToolMode::Camera => ToolMode::Grab,
                };
                lab.held = None;
                lab.mouse_drag = None;
                lab.touch_on_scene = false;
            }
        }
    }
}

fn over_ui(ui: &Query<&Interaction, With<Button>>) -> bool {
    ui.iter().any(|i| *i != Interaction::None)
}

/// Grab mode only: pick / drag toys (unchanged WindowEvent path).
fn pointer_input(
    mut lab: ResMut<Lab>,
    mut events: MessageReader<WindowEvent>,
    camera: Query<(&Camera, &GlobalTransform), With<LabCamera>>,
    toys: Query<(Entity, &GlobalTransform, &Toy)>,
    bodies: Query<&PhysicsBody3d, With<Toy>>,
    ui: Query<&Interaction, With<Button>>,
    mut world: ResMut<PhysicsWorld3d>,
    mut pointer: Local<Option<Vec2>>,
) {
    if lab.fresh {
        lab.fresh = false;
        events.clear();
        return;
    }
    if lab.mode != ToolMode::Grab {
        events.clear();
        return;
    }
    let Ok((camera, camera_at)) = camera.single() else {
        return;
    };
    let blocked = over_ui(&ui);

    for event in events.read() {
        match event {
            WindowEvent::CursorMoved(moved) => {
                *pointer = Some(moved.position);
            }
            WindowEvent::MouseButtonInput(input) if input.button == MouseButton::Left => {
                if input.state == ButtonState::Released {
                    lab.held = None;
                } else if input.state == ButtonState::Pressed && !blocked {
                    if let Some(screen) = *pointer {
                        if let Ok(ray) = camera.viewport_to_world(camera_at, screen) {
                            pick(&mut lab, &toys, &world, ray);
                        }
                    }
                }
            }
            WindowEvent::TouchInput(touch) => {
                *pointer = Some(touch.position);
                match touch.phase {
                    TouchPhase::Started if !blocked => {
                        if let Ok(ray) = camera.viewport_to_world(camera_at, touch.position) {
                            pick(&mut lab, &toys, &world, ray);
                        }
                    }
                    TouchPhase::Ended | TouchPhase::Canceled => lab.held = None,
                    _ => {}
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
    let Ok(ray) = camera.viewport_to_world(camera_at, screen) else {
        return;
    };
    let Some(at) = hit_plane(ray, lab.drag_y) else {
        return;
    };
    let Ok(PhysicsBody3d(id)) = bodies.get(held) else {
        lab.held = None;
        return;
    };
    if !world.sim.contains(*id) {
        lab.held = None;
        return;
    }
    world.sim.set_position(*id, [at.x, at.y, at.z]);
    world.sim.set_velocity(*id, [0.0, 0.0, 0.0]);
}

/// Mouse path — trimmed `editor/stage/camera.rs` `control_camera` for Camera mode.
/// In Camera mode LMB orbits (no Alt; Grab owns plain LMB when toggled the other way).
fn control_camera(
    mut lab: ResMut<Lab>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    ui: Query<&Interaction, With<Button>>,
    mut camera: Query<&mut Transform, With<LabCamera>>,
) {
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };

    if lab.mode != ToolMode::Camera {
        *transform = lab.orbit.transform();
        return;
    }

    let blocked = over_ui(&ui);
    if let Some(button) = lab.mouse_drag {
        if !buttons.pressed(button) {
            lab.mouse_drag = None;
        }
    }
    if lab.mouse_drag.is_none() && !blocked {
        for button in [MouseButton::Left, MouseButton::Middle, MouseButton::Right] {
            if buttons.just_pressed(button) {
                lab.mouse_drag = Some(button);
            }
        }
    }

    let delta = motion.delta;
    let wheel = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / 40.0,
    };
    let pan = lab.orbit.distance.max(1.0) * 0.0015;
    let drag = lab.mouse_drag;
    let cam = &mut lab.orbit;

    match drag {
        Some(MouseButton::Left) => {
            cam.yaw -= delta.x * LOOK;
            cam.pitch = (cam.pitch - delta.y * LOOK).clamp(-1.54, 1.54);
        }
        Some(MouseButton::Middle) | Some(MouseButton::Right) => {
            let right = cam.rotation() * Vec3::X;
            let up = cam.rotation() * Vec3::Y;
            cam.pivot += (-right * delta.x + up * delta.y) * pan;
        }
        Some(_) => {}
        None => {
            if !blocked && wheel != 0.0 {
                let step = cam.distance.max(2.0) * 0.12 * wheel;
                if cam.distance - step > 0.5 {
                    cam.distance -= step;
                } else {
                    let forward = cam.rotation() * Vec3::NEG_Z;
                    cam.pivot += forward * step;
                }
            }
        }
    }

    *transform = lab.orbit.transform();
}

/// Touch path — same as `editor/stage/touch.rs` `touch_camera` (Android / tablets).
fn touch_camera(
    mut lab: ResMut<Lab>,
    touches: Res<Touches>,
    ui: Query<&Interaction, With<Button>>,
    mut camera: Query<&mut Transform, With<LabCamera>>,
) {
    if lab.mode != ToolMode::Camera {
        return;
    }
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };

    // First finger down: remember whether it started off the HUD (editor `on_scene`).
    if touches.iter_just_pressed().next().is_some() && touches.iter().count() == 1 {
        lab.touch_on_scene = !over_ui(&ui);
    }
    if !lab.touch_on_scene {
        return;
    }

    let fingers: Vec<_> = touches.iter().collect();
    let cam = &mut lab.orbit;
    match fingers.as_slice() {
        [one] => {
            let delta = one.delta();
            cam.yaw -= delta.x * LOOK_TOUCH;
            cam.pitch = (cam.pitch - delta.y * LOOK_TOUCH).clamp(-1.54, 1.54);
        }
        [a, b, ..] => {
            let delta = (a.delta() + b.delta()) * 0.5;
            let now = a.position().distance(b.position());
            let before = (a.position() - a.delta()).distance(b.position() - b.delta());
            let pinch = if now > 1.0 && before > 1.0 {
                before / now
            } else {
                1.0
            };
            let pan = cam.distance.max(1.0) * 0.0015;
            let right = cam.rotation() * Vec3::X;
            let up = cam.rotation() * Vec3::Y;
            cam.pivot += (-right * delta.x + up * delta.y) * pan;
            cam.distance = (cam.distance * pinch).clamp(0.5, 200.0);
        }
        _ => return,
    }
    *transform = lab.orbit.transform();
}

fn pick(
    lab: &mut Lab,
    toys: &Query<(Entity, &GlobalTransform, &Toy)>,
    world: &PhysicsWorld3d,
    ray: Ray3d,
) {
    let origin = ray.origin;
    let dir = *ray.direction;
    if let Some((body, toi)) = world.sim.ray(
        [origin.x, origin.y, origin.z],
        [dir.x, dir.y, dir.z],
        80.0,
    ) {
        if let Some(entity) = world.entity(body) {
            if toys.get(entity).is_ok() {
                let hit = origin + dir * toi;
                lab.held = Some(entity);
                lab.drag_y = hit.y.max(0.2);
                return;
            }
        }
    }
    let mut best: Option<(Entity, f32, f32)> = None;
    for (entity, transform, toy) in toys {
        let pos = transform.translation();
        let to = pos - origin;
        let t = to.dot(dir);
        if t < 0.0 {
            continue;
        }
        let closest = origin + dir * t;
        let dist = closest.distance(pos);
        if dist <= toy.half_extent * 1.4 {
            if best.is_none_or(|(_, _, d)| dist < d) {
                best = Some((entity, pos.y, dist));
            }
        }
    }
    if let Some((entity, y, _)) = best {
        lab.held = Some(entity);
        lab.drag_y = y.max(0.2);
    }
}

fn hit_plane(ray: Ray3d, y: f32) -> Option<Vec3> {
    let dir = *ray.direction;
    if dir.y.abs() < 1e-5 {
        return None;
    }
    let t = (y - ray.origin.y) / dir.y;
    (t > 0.0).then_some(ray.origin + dir * t)
}

fn spawn_toy(
    commands: &mut Commands,
    parent: Entity,
    kind: SpawnKind,
    at: Vec3,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let (mesh, color, body, toy) = match kind {
        SpawnKind::Cube => (
            meshes.add(Cuboid::from_length(0.8)),
            CUBE,
            Body3d::dynamic(Body3d::box_half(Vec3::splat(0.4))),
            Toy {
                kind: ToyKind::Cube,
                half_extent: 0.4,
            },
        ),
        SpawnKind::Ball => (
            meshes.add(Sphere::new(0.4)),
            BALL,
            Body3d::dynamic(Body3d::ball(0.4)),
            Toy {
                kind: ToyKind::Ball,
                half_extent: 0.4,
            },
        ),
    };
    commands.entity(parent).with_children(|plot| {
        plot.spawn((
            toy,
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })),
            Transform::from_translation(at),
            body,
        ));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use xerxes_engine::modules::assets::spawn_scene;
    use xerxes_engine::modules::ui::Slider;

    #[test]
    fn orbit_from_scene_eye_stays_above_the_ground() {
        let orbit = Orbit::scene_default();
        let eye = orbit.position();
        assert!(
            eye.y > 1.0,
            "camera should sit above the plot, got y={}",
            eye.y
        );
        assert!(
            (eye - Vec3::new(6.0, 7.0, 9.0)).length() < 0.05,
            "should match the scene camera eye, got {eye:?}"
        );
    }

    #[test]
    fn the_scene_is_a_plot_with_toys_and_a_mode_toggle() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        spawn_scene(
            &mut world,
            &crate::assets::scenes::games::outside::pairidaeza_scene::scene(),
        );
        assert_eq!(world.query::<&Plot>().iter(&world).count(), 1);
        assert_eq!(world.query::<&LabCamera>().iter(&world).count(), 1);
        assert!(world.query::<&Toy>().iter(&world).count() >= 2);
        let slider = world
            .query::<&Slider>()
            .iter(&world)
            .next()
            .expect("gravity slider");
        assert!((slider.value - 1.0).abs() < 0.001);
        assert_eq!(
            world.query::<&WorkshopButton>().iter(&world).count(),
            4,
            "mode, cube, ball, reset"
        );
        assert_eq!(world.query::<&ModeLabel>().iter(&world).count(), 1);
    }
}
