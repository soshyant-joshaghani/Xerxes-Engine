//! The Scene view (Bevy): the open scene as the editor sees it, the editor-default camera,
//! the ground grid, selection and picking. Driven by bridge commands from the UI.
//!
//! - `camera`  the editor camera (Unreal controls) and the Scene view viewport
//! - `scene`   spawning the scene view, textures, picking, icons, the selection outline

mod camera;
mod gizmo;
pub mod runtime;
mod scene;
mod shortcuts;
pub mod touch;

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::protocol::{EditorCommand, EditorSnapshot, Space, Tool};
use super::runtime::{UiBridge, UiCommand};
use camera::SceneInput;

type Bridge = UiBridge<EditorCommand, EditorSnapshot>;

pub struct StagePlugin;

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        crate::editor::touch::init();
        app.insert_resource(ClearColor(Color::srgb(0.19, 0.19, 0.2)))
            .insert_resource(GlobalAmbientLight {
                color: Color::WHITE,
                brightness: 400.0,
                ..default()
            })
            .init_resource::<FrameRate>()
            .init_resource::<camera::ActiveLayout>()
            .init_resource::<camera::ActiveWorkspace>()
            .init_resource::<camera::ViewState>()
            .init_resource::<shortcuts::Shortcuts>()
            .init_resource::<SceneInput>()
            .init_resource::<crate::editor::host::UiFocus>()
            .init_resource::<Selected>()
            .init_resource::<ActiveTool>()
            .init_resource::<ActiveSpace>()
            .init_resource::<Keyed>()
            .init_resource::<Picked>()
            .init_resource::<scene::Textures>()
            .init_resource::<scene::LastView>()
            .init_resource::<gizmo::GizmoState>()
            .init_resource::<touch::TouchPointer>()
            // Touch becomes the pointer right after Bevy reads input (before anything uses it).
            .add_systems(
                PreUpdate,
                (touch::touch_as_pointer, touch::touch_mode).after(bevy::input::InputSystems),
            )
            .add_systems(
                Update,
                (
                    apply_commands,
                    camera::fit_viewport,
                    (
                        camera::control_camera,
                        touch::touch_camera,
                        gizmo::gizmo,
                        scene::pick,
                        tool_keys,
                        shortcuts::shortcuts,
                    )
                        .chain(),
                    (
                        draw_grid,
                        scene::draw_icons,
                        scene::draw_cameras,
                        scene::draw_selection,
                        measure_frame_rate,
                    ),
                    publish_snapshot,
                )
                    .chain(),
            );
    }
}

/// An object of the shown scene: its view id, and what clicking it selects.
#[derive(Component, Debug, Clone, Copy)]
pub struct StageObject {
    pub id: u32,
    pub select: u32,
}

/// The selected object (view id), as the UI last said or a click picked.
#[derive(Resource, Debug, Default)]
pub struct Selected(pub Option<u32>);

#[derive(Resource, Debug, Default)]
pub struct ActiveTool(pub Tool);

#[derive(Resource, Debug, Default)]
pub struct ActiveSpace(pub Space);

/// The last W/E/R/X change, for the UI (see `EditorSnapshot::keyed`).
#[derive(Resource, Debug, Default)]
struct Keyed(Option<(Tool, Space, u64)>);

/// The last pick, published to the UI.
#[derive(Resource, Debug, Default)]
pub struct Picked(pub Option<(Option<u32>, u64)>);

/// Frames per second, refreshed once a second.
#[derive(Resource, Default)]
struct FrameRate {
    frames: u32,
    elapsed: f32,
    fps: u32,
}

#[allow(clippy::too_many_arguments)]
fn apply_commands(
    mut commands: Commands,
    mut messages: MessageReader<UiCommand<EditorCommand>>,
    mut selected: ResMut<Selected>,
    mut tool: ResMut<ActiveTool>,
    mut space: ResMut<ActiveSpace>,
    mut input: ResMut<SceneInput>,
    mut workspace: ResMut<camera::ActiveWorkspace>,
    mut views: ResMut<camera::ViewState>,
    objects: Query<(&GlobalTransform, &StageObject)>,
    mut cameras: Query<(
        &mut camera::EditorCamera,
        &mut Transform,
        &mut Projection,
        &camera::ViewportArea,
        Has<camera::ActiveCamera>,
    )>,
) {
    for UiCommand(command) in messages.read() {
        match command {
            EditorCommand::ShowScene(view) => {
                let flat = view.dimension == crate::editor::project::codec::Dimension::D2;
                views.flat = flat;
                for (mut cam, mut transform, mut projection, _, _) in &mut cameras {
                    cam.set_flat(flat);
                    *transform = cam.transform();
                    *projection = cam.projection();
                }
                let view = view.clone();
                commands.queue(move |world: &mut World| scene::show(world, &view));
            }
            EditorCommand::Texture { path, bytes } => {
                let (path, bytes) = (path.clone(), bytes.clone());
                commands.queue(move |world: &mut World| scene::add_texture(world, path, &bytes));
            }
            EditorCommand::Select(id) => selected.0 = *id,
            EditorCommand::SetTool(next) => tool.0 = *next,
            EditorCommand::SetSpace(next) => space.0 = *next,
            EditorCommand::Frame => {
                for (mut cam, mut transform, mut projection, _, active) in &mut cameras {
                    if active {
                        camera::frame(&mut cam, &selected, &objects);
                        *transform = cam.transform();
                        *projection = cam.projection();
                    }
                }
            }
            EditorCommand::ViewAction(id, action) => {
                for (mut cam, mut transform, mut projection, area, _) in &mut cameras {
                    if area.0 == *id {
                        camera::view_action(&mut cam, *action, &selected, &objects);
                        *transform = cam.transform();
                        *projection = cam.projection();
                    }
                }
            }
            EditorCommand::Modal(modal) => input.modal = *modal,
            EditorCommand::SetLayout(next) => workspace.0 = (**next).clone(),
        }
    }
}

/// W / E / R switch the transform tool and X the gizmo space, like Unity: anywhere in the
/// editor except while typing in a text field (or flying the camera with RMB + WASD).
fn tool_keys(
    keys: Res<ButtonInput<KeyCode>>,
    input: Res<SceneInput>,
    focus: Res<crate::editor::host::UiFocus>,
    mut tool: ResMut<ActiveTool>,
    mut space: ResMut<ActiveSpace>,
    mut keyed: ResMut<Keyed>,
) {
    // Ctrl+S, Ctrl+Z... are the keymap's, not tool keys.
    let modified = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    if input.dragging.is_some() || focus.0 || modified {
        return;
    }
    let next_tool = if keys.just_pressed(KeyCode::KeyW) {
        Some(Tool::Move)
    } else if keys.just_pressed(KeyCode::KeyE) {
        Some(Tool::Rotate)
    } else if keys.just_pressed(KeyCode::KeyR) {
        Some(Tool::Scale)
    } else {
        None
    };
    let toggle_space = keys.just_pressed(KeyCode::KeyX);
    if next_tool.is_none() && !toggle_space {
        return;
    }
    if let Some(next) = next_tool {
        tool.0 = next;
    }
    if toggle_space {
        space.0 = match space.0 {
            Space::Global => Space::Local,
            Space::Local => Space::Global,
        };
    }
    let seq = keyed.0.map_or(1, |(_, _, n)| n + 1);
    keyed.0 = Some((tool.0, space.0, seq));
}

/// The ground grid and the X (red) / Z (blue) axes.
fn draw_grid(mut gizmos: Gizmos, views: Res<camera::ViewState>) {
    if views.flat {
        // 2D: the XY plane, in pixels (a 50 px grid); X red, Y green.
        gizmos.grid(
            Isometry3d::IDENTITY,
            UVec2::splat(80),
            Vec2::splat(50.0),
            Color::srgba(0.6, 0.6, 0.6, 0.15),
        );
        gizmos.line(
            Vec3::new(-2000.0, 0.0, 0.0),
            Vec3::new(2000.0, 0.0, 0.0),
            Color::srgb(0.85, 0.3, 0.3),
        );
        gizmos.line(
            Vec3::new(0.0, -2000.0, 0.0),
            Vec3::new(0.0, 2000.0, 0.0),
            Color::srgb(0.4, 0.8, 0.35),
        );
        return;
    }
    gizmos.grid(
        Isometry3d::new(Vec3::new(0.0, 0.001, 0.0), Quat::from_rotation_x(FRAC_PI_2)),
        UVec2::splat(40),
        Vec2::splat(1.0),
        Color::srgba(0.6, 0.6, 0.6, 0.18),
    );
    gizmos.line(
        Vec3::new(-20.0, 0.002, 0.0),
        Vec3::new(20.0, 0.002, 0.0),
        Color::srgb(0.85, 0.3, 0.3),
    );
    gizmos.line(
        Vec3::new(0.0, 0.002, -20.0),
        Vec3::new(0.0, 0.002, 20.0),
        Color::srgb(0.3, 0.5, 0.95),
    );
}

fn measure_frame_rate(time: Res<Time>, mut rate: ResMut<FrameRate>) {
    rate.frames += 1;
    rate.elapsed += time.delta_secs();
    if rate.elapsed >= 1.0 {
        rate.fps = (rate.frames as f32 / rate.elapsed).round() as u32;
        rate.frames = 0;
        rate.elapsed = 0.0;
    }
}

fn publish_snapshot(
    bridge: Res<Bridge>,
    rate: Res<FrameRate>,
    picked: Res<Picked>,
    keyed: Res<Keyed>,
    gizmo: Res<gizmo::GizmoState>,
    layout: Res<camera::ActiveLayout>,
    shortcuts: Res<shortcuts::Shortcuts>,
) {
    bridge.publish(EditorSnapshot {
        fps: rate.fps,
        picked: picked.0,
        dragged: gizmo.dragged,
        keyed: keyed.0,
        action: shortcuts.fired,
        layout: layout.0,
    });
}
