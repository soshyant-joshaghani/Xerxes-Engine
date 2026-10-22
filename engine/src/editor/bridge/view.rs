//! What the Scene view draws: a flat list of objects built from the parsed scene, with
//! prefabs merged in (instance components replace the prefab's of the same type, like the
//! game does). Built in the UI, sent over the bridge, spawned by the stage.

use std::collections::HashMap;

use crate::editor::project::codec::{Field, ObjectDoc, SceneDoc, TransformData, Value};

/// Ids from this value up are objects inside a prefab instance (not their own selection).
pub const PREFAB_CHILD_IDS: u32 = 1_000_000;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SceneView {
    /// 2D scenes show flat (orthographic, XY), 3D scenes in perspective.
    pub dimension: crate::editor::project::codec::Dimension,
    pub objects: Vec<ViewObject>,
    /// The project's window size: the reference resolution of canvases that name none.
    /// `[0, 0]` until set (1280 x 720 then).
    pub ui_reference: [u32; 2],
}

impl SceneView {
    pub fn with_reference(mut self, reference: [u32; 2]) -> Self {
        self.ui_reference = reference;
        self
    }

    /// The reference resolution canvases use by default.
    pub fn reference(&self) -> [u32; 2] {
        if self.ui_reference[0] == 0 || self.ui_reference[1] == 0 {
            [1280, 720]
        } else {
            self.ui_reference
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewObject {
    pub id: u32,
    /// What a click on it selects (itself, or the prefab instance it belongs to).
    pub select: u32,
    pub parent: Option<u32>,
    pub name: String,
    pub transform: TransformData,
    pub mesh: Option<ViewMesh>,
    /// sRGBA and an optional texture path (`art/crate.png`).
    pub color: [f32; 4],
    pub texture: Option<String>,
    pub light: Option<ViewLight>,
    pub camera: Option<ViewCamera>,
    /// Set for UI elements and canvases (see `ui_view`).
    pub ui: Option<super::ui_view::ViewUi>,
    /// `Visibility::Hidden` was authored: not drawn (nor are its children).
    pub hidden: bool,
    /// How an orthographic game camera frames the world (drawn as its frame in the Scene
    /// view, so the level can be placed against what the player will see).
    pub ortho: Option<ViewOrtho>,
}

/// An orthographic projection's scaling mode, with its `scale`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewOrtho {
    pub mode: OrthoMode,
    pub scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrthoMode {
    /// One pixel to one unit (`WindowSize`).
    WindowSize,
    /// At least this much of the world fits.
    AutoMin {
        width: f32,
        height: f32,
    },
    /// At most this much of the world shows.
    AutoMax {
        width: f32,
        height: f32,
    },
    Fixed {
        width: f32,
        height: f32,
    },
    FixedVertical(f32),
    FixedHorizontal(f32),
}

impl ViewOrtho {
    /// The part of the world the camera shows on a screen of `screen` pixels (width, height).
    pub fn visible(&self, screen: [u32; 2]) -> [f32; 2] {
        let aspect = screen[0].max(1) as f32 / screen[1].max(1) as f32;
        let [w, h] = match self.mode {
            OrthoMode::WindowSize => [screen[0] as f32, screen[1] as f32],
            OrthoMode::AutoMin { width, height } => {
                if width / height < aspect {
                    [height * aspect, height]
                } else {
                    [width, width / aspect]
                }
            }
            OrthoMode::AutoMax { width, height } => {
                if width / height > aspect {
                    [height * aspect, height]
                } else {
                    [width, width / aspect]
                }
            }
            OrthoMode::Fixed { width, height } => [width, height],
            OrthoMode::FixedVertical(height) => [height * aspect, height],
            OrthoMode::FixedHorizontal(width) => [width, width / aspect],
        };
        [w * self.scale, h * self.scale]
    }
}

/// Reads a `Projection::Orthographic(OrthographicProjection { scaling_mode: .., scale: .. })`
/// as written.
fn parse_ortho(src: &str) -> Option<ViewOrtho> {
    let s: String = src.split_whitespace().collect();
    if !s.contains("Orthographic") {
        return None;
    }
    let num = |key: &str| -> Option<f32> {
        let at = s.find(&format!("{key}:"))? + key.len() + 1;
        let rest = &s[at..];
        let end = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .unwrap_or(rest.len());
        rest[..end].parse().ok()
    };
    let mode = if s.contains("AutoMin") {
        OrthoMode::AutoMin {
            width: num("min_width")?,
            height: num("min_height")?,
        }
    } else if s.contains("AutoMax") {
        OrthoMode::AutoMax {
            width: num("max_width")?,
            height: num("max_height")?,
        }
    } else if s.contains("FixedVertical") {
        OrthoMode::FixedVertical(num("viewport_height")?)
    } else if s.contains("FixedHorizontal") {
        OrthoMode::FixedHorizontal(num("viewport_width")?)
    } else if s.contains("Fixed{") {
        OrthoMode::Fixed {
            width: num("width")?,
            height: num("height")?,
        }
    } else {
        OrthoMode::WindowSize
    };
    // `scale` also appears inside other names; the field is `scale:` after the mode.
    let scale = num("scale").unwrap_or(1.0);
    Some(ViewOrtho { mode, scale })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMesh {
    Cube(f32),
    /// A box of its own width, height and depth.
    Cuboid([f32; 3]),
    Sphere(f32),
    Plane(f32),
    /// A 2D sprite, as a flat quad of this size.
    Quad([f32; 2]),
    /// A model from a bundle (`ModelDef`). The Scene view draws a box one unit tall standing on
    /// the object's origin, the way a glTF model stands on its own: the object can be seen, picked
    /// and moved before the editor can load the model itself.
    Model,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewLight {
    Directional { shadows: bool },
    Point,
    Spot,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewCamera {
    Perspective,
    Orthographic,
}

/// Builds the view. `prefabs` maps a prefab path as written in scenes
/// (`crate::assets::prefabs::player_prefab::prefab`) to its parsed root.
pub fn build(doc: &SceneDoc, prefabs: &HashMap<String, ObjectDoc>) -> SceneView {
    let mut view = SceneView {
        dimension: doc.dimension,
        ..Default::default()
    };
    let mut synthetic = PREFAB_CHILD_IDS;
    for object in &doc.objects {
        add(
            &mut view,
            object,
            None,
            None,
            prefabs,
            &doc.consts,
            &mut synthetic,
        );
    }
    view
}

/// The component type an override replaces (`MeshDef::Cube` and `MeshDef::Sphere` are both
/// the `MeshDef` component).
fn base(ty: &str) -> &str {
    ty.split("::").next().unwrap_or(ty)
}

/// The object's effective components and children: its prefab's (recursively), then its own.
fn resolve<'a>(
    object: &'a ObjectDoc,
    prefabs: &'a HashMap<String, ObjectDoc>,
    depth: usize,
) -> (
    String,
    Vec<&'a crate::editor::project::codec::ComponentDoc>,
    Vec<(&'a ObjectDoc, bool)>,
) {
    let mut name = object.name.clone();
    let mut components = Vec::new();
    let mut children = Vec::new();
    if let Some(prefab) = object
        .prefab
        .as_ref()
        .and_then(|p| prefabs.get(p))
        .filter(|_| depth < 8)
    {
        let (prefab_name, prefab_components, prefab_children) = resolve(prefab, prefabs, depth + 1);
        if name.is_empty() {
            name = prefab_name;
        }
        components = prefab_components;
        children = prefab_children
            .into_iter()
            .map(|(c, _)| (c, true))
            .collect();
    }
    for own in &object.components {
        components.retain(|c: &&crate::editor::project::codec::ComponentDoc| {
            base(&c.ty) != base(&own.ty)
        });
        components.push(own);
    }
    children.extend(object.children.iter().map(|c| (c, false)));
    if name.is_empty() {
        name = object
            .prefab
            .as_deref()
            .map(|p| p.rsplit("::").nth(1).unwrap_or(p).to_string())
            .unwrap_or_default();
    }
    (name, components, children)
}

fn add(
    view: &mut SceneView,
    object: &ObjectDoc,
    parent: Option<u32>,
    select: Option<u32>,
    prefabs: &HashMap<String, ObjectDoc>,
    consts: &[crate::editor::project::codec::ConstDoc],
    synthetic: &mut u32,
) {
    let (name, components, children) = resolve(object, prefabs, 0);
    let id = if select.is_some() {
        *synthetic += 1;
        *synthetic
    } else {
        object.id
    };
    let select = select.unwrap_or(id);
    let mut item = ViewObject {
        id,
        select,
        parent,
        name,
        transform: TransformData::default(),
        mesh: None,
        color: [1.0; 4],
        texture: None,
        light: None,
        camera: None,
        ui: super::ui_view::extract(&components, consts),
        hidden: components
            .iter()
            .any(|c| matches!(&c.value, Value::Unit(p) if p == "Visibility::Hidden")),
        ortho: components.iter().find_map(|c| match (&c.ty, &c.value) {
            (ty, Value::Expr(src)) if base(ty) == "Projection" => parse_ortho(src),
            _ => None,
        }),
    };
    for component in components {
        let fields = match &component.value {
            Value::Struct { fields, .. } => fields.as_slice(),
            _ => &[],
        };
        let num = |name: &str| {
            fields.iter().find(|(n, _)| n == name).and_then(|(_, f)| {
                if let Field::Num(n) = f {
                    Some(*n)
                } else {
                    None
                }
            })
        };
        match component.ty.as_str() {
            "Transform" => {
                if let Value::Transform(t) = component.value {
                    item.transform = t;
                }
            }
            "MeshDef::Cube" => item.mesh = Some(ViewMesh::Cube(num("size").unwrap_or(1.0))),
            "MeshDef::Cuboid" => {
                let size = fields
                    .iter()
                    .find(|(n, _)| n == "size")
                    .and_then(|(_, f)| match f {
                        Field::Vec3(v) => Some(*v),
                        _ => None,
                    });
                item.mesh = Some(ViewMesh::Cuboid(size.unwrap_or([1.0; 3])));
            }
            "MeshDef::Sphere" => item.mesh = Some(ViewMesh::Sphere(num("radius").unwrap_or(0.5))),
            "MeshDef::Plane" => item.mesh = Some(ViewMesh::Plane(num("size").unwrap_or(1.0))),
            // A model's own mesh is inside its file; the Scene view stands a box where it is.
            "ModelDef" => item.mesh = Some(ViewMesh::Model),
            "MaterialDef" | "SpriteDef" => {
                for (field, value) in fields {
                    match (field.as_str(), value) {
                        ("color", Field::Color(c)) => item.color = *c,
                        // A file constant (`color: WALL`).
                        ("color", Field::Path(name)) => {
                            if let Some(c) = super::ui_view::const_color(name, consts) {
                                item.color = c;
                            }
                        }
                        ("texture" | "image", Field::OptStr(path)) => item.texture = path.clone(),
                        ("size", Field::Expr(_)) | ("size", Field::OptStr(_)) => {}
                        _ => {}
                    }
                }
                if component.ty == "SpriteDef" {
                    let size = fields.iter().find_map(|(n, f)| (n == "size").then_some(f));
                    let size = match size {
                        Some(Field::Expr(src)) => parse_some_vec2(src),
                        _ => None,
                    };
                    item.mesh = Some(ViewMesh::Quad(size.unwrap_or([1.0, 1.0])));
                }
            }
            "DirectionalLight" => {
                let shadows = fields
                    .iter()
                    .any(|(n, f)| n == "shadows_enabled" && *f == Field::Bool(true));
                item.light = Some(ViewLight::Directional { shadows });
            }
            "PointLight" => item.light = Some(ViewLight::Point),
            "SpotLight" => item.light = Some(ViewLight::Spot),
            "Camera3d" | "GameCamera3D" => item.camera = Some(ViewCamera::Perspective),
            "Camera2d" | "GameCamera2D" => item.camera = Some(ViewCamera::Orthographic),
            _ => {}
        }
    }
    // A 2D camera with no projection of its own shows one pixel to one unit.
    if item.camera == Some(ViewCamera::Orthographic) && item.ortho.is_none() {
        item.ortho = Some(ViewOrtho {
            mode: OrthoMode::WindowSize,
            scale: 1.0,
        });
    }
    view.objects.push(item);
    for (child, inside_prefab) in children {
        let child_select = if inside_prefab || select != id {
            Some(select)
        } else {
            None
        };
        add(
            view,
            child,
            Some(id),
            child_select,
            prefabs,
            consts,
            synthetic,
        );
    }
}

/// `Some(Vec2::new(x, y))` as written, for sprite sizes.
pub fn parse_some_vec2(src: &str) -> Option<[f32; 2]> {
    let inner = src.replace(' ', "");
    let inner = inner.strip_prefix("Some(Vec2::new(")?.strip_suffix("))")?;
    let (x, y) = inner.split_once(',')?;
    Some([x.parse().ok()?, y.parse().ok()?])
}
