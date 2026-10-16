//! The Canvas: where a UI is drawn.
//!
//! | mode | Unity name | what it does |
//! |------|------------|--------------|
//! | [`CanvasMode::ScreenSpaceOverlay`] | Screen Space - Overlay | drawn over the window by the default UI camera |
//! | [`CanvasMode::ScreenSpaceCamera`] | Screen Space - Camera | drawn by the scene's camera (it sticks to it) |
//! | [`CanvasMode::WorldSpace`] | World Space | a UI camera renders into an image, shown on a quad at the canvas's `Transform` |

use bevy::camera::{ClearColorConfig, RenderTarget, visibility::RenderLayers};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

/// Where a canvas is drawn.
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasMode {
    ScreenSpaceOverlay,
    ScreenSpaceCamera,
    /// A surface of `size` pixels, `pixels_per_unit` pixels to one world unit.
    WorldSpace {
        size: UVec2,
        pixels_per_unit: f32,
    },
}

/// The root of a UI. Put the elements (`Node`, `Text`, `Button`, widgets...) under it.
/// `Node` is required and defaults to the full window (or the whole surface in world space).
#[derive(Component, Debug, Clone, PartialEq)]
#[require(Node = canvas_node())]
pub struct Canvas {
    pub mode: CanvasMode,
    /// Higher draws above lower (screen-space canvases).
    pub sort_order: i32,
    /// For `ScreenSpaceCamera`: the object (by name) of the camera it sticks to. `None` is the
    /// camera marked [`CanvasCamera`], else the highest camera that draws to the window.
    pub camera: Option<&'static str>,
    /// The size the UI is designed for; `None` is the project's (its window size, in Project
    /// Settings). The editor's Scene view draws a screen-space canvas this big.
    pub reference_resolution: Option<UVec2>,
}

/// UI settings that belong to the whole project, not to a canvas. The reference resolution is
/// the project's window size (`WindowSettings`), so one place says how big the UI is designed.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct UiSettings {
    pub reference_resolution: UVec2,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            reference_resolution: UVec2::new(1280, 720),
        }
    }
}

fn canvas_node() -> Node {
    Node {
        width: percent(100),
        height: percent(100),
        position_type: PositionType::Absolute,
        ..default()
    }
}

impl Canvas {
    pub fn overlay() -> Self {
        Self {
            mode: CanvasMode::ScreenSpaceOverlay,
            sort_order: 0,
            camera: None,
            reference_resolution: None,
        }
    }

    pub fn camera() -> Self {
        Self {
            mode: CanvasMode::ScreenSpaceCamera,
            ..Self::overlay()
        }
    }

    pub fn world(width: u32, height: u32, pixels_per_unit: f32) -> Self {
        Self {
            mode: CanvasMode::WorldSpace {
                size: UVec2::new(width, height),
                pixels_per_unit,
            },
            sort_order: 0,
            camera: None,
            reference_resolution: Some(UVec2::new(width, height)),
        }
    }

    /// Sticks a `ScreenSpaceCamera` canvas to the camera object with this name.
    pub fn camera_named(mut self, name: &'static str) -> Self {
        self.camera = Some(name);
        self
    }

    pub fn sorted(mut self, sort_order: i32) -> Self {
        self.sort_order = sort_order;
        self
    }

    pub fn reference(mut self, width: u32, height: u32) -> Self {
        self.reference_resolution = Some(UVec2::new(width, height));
        self
    }
}

/// Put on the camera a `ScreenSpaceCamera` canvas should stick to (the highest camera that
/// draws to the window is used when none has it).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct CanvasCamera;

/// Added to a world-space canvas's root: its UI camera and the image it renders into.
#[derive(Component, Debug, Clone)]
pub struct CanvasSurface {
    pub camera: Entity,
    pub image: Handle<Image>,
    /// The surface in pixels, and how many of them are one world unit.
    pub size: UVec2,
    pub pixels_per_unit: f32,
}

/// `ScreenSpaceCamera` canvases are laid out and drawn by the scene's camera.
pub(super) fn attach_camera(
    mut commands: Commands,
    canvases: Query<(Entity, &Canvas), (Without<UiTargetCamera>, Without<ChildOf>)>,
    cameras: Query<(
        Entity,
        &Camera,
        &RenderTarget,
        Has<CanvasCamera>,
        Option<&Name>,
    )>,
) {
    for (entity, canvas) in &canvases {
        if canvas.mode != CanvasMode::ScreenSpaceCamera {
            continue;
        }
        let best = cameras
            .iter()
            .filter(|(_, camera, target, _, name)| {
                camera.is_active
                    && matches!(target, RenderTarget::Window(_))
                    && canvas
                        .camera
                        .is_none_or(|wanted| name.is_some_and(|n| n.as_str() == wanted))
            })
            .max_by_key(|(_, camera, _, marked, _)| (*marked, camera.order))
            .map(|(e, ..)| e);
        if let Some(camera) = best {
            commands.entity(entity).insert(UiTargetCamera(camera));
        }
    }
}

/// Screen-space canvases sort by `sort_order` (a global z index).
pub(super) fn order_overlay(
    mut commands: Commands,
    added: Query<(Entity, &Canvas), Added<Canvas>>,
) {
    for (entity, canvas) in &added {
        if !matches!(canvas.mode, CanvasMode::WorldSpace { .. }) {
            commands
                .entity(entity)
                .insert(GlobalZIndex(canvas.sort_order));
        }
    }
}

/// World-space canvases: a UI camera that renders into an image, and a quad (a 3D mesh, and a
/// sprite for 2D cameras: each camera kind draws its own) that shows it.
pub(super) fn build_surface(
    mut commands: Commands,
    added: Query<(Entity, &Canvas, Option<&Children>), (Added<Canvas>, Without<CanvasSurface>)>,
    elements: Query<(), With<Node>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, canvas, children) in &added {
        let CanvasMode::WorldSpace {
            size,
            pixels_per_unit,
        } = canvas.mode
        else {
            continue;
        };
        // A world-space canvas with no UI under it is a layer for world objects (sprites,
        // meshes): it only places and groups them, with nothing to render.
        let has_ui = children.is_some_and(|c| c.iter().any(|child| elements.contains(child)));
        if !has_ui {
            commands.entity(entity).insert(Node {
                display: Display::None,
                ..default()
            });
            continue;
        }
        let size = size.max(UVec2::ONE);
        let units = size.as_vec2() / pixels_per_unit.max(0.0001);
        let image = images.add(Image::new_target_texture(
            size.x,
            size.y,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        let camera = commands
            .spawn((
                Name::new("Canvas Camera"),
                Camera2d,
                Camera {
                    order: -100 + canvas.sort_order as isize,
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    ..default()
                },
                RenderTarget::from(image.clone()),
                // It draws UI only: no world entity is on its layer.
                RenderLayers::none(),
                ChildOf(entity),
            ))
            .id();
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(image.clone()),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            Name::new("Canvas Surface"),
            Mesh3d(meshes.add(Rectangle::new(units.x, units.y))),
            MeshMaterial3d(material),
            ChildOf(entity),
        ));
        commands.spawn((
            Name::new("Canvas Surface 2D"),
            Sprite {
                image: image.clone(),
                custom_size: Some(units),
                ..default()
            },
            ChildOf(entity),
        ));
        commands.entity(entity).insert((
            UiTargetCamera(camera),
            CanvasSurface {
                camera,
                image,
                size,
                pixels_per_unit,
            },
        ));
    }
}
