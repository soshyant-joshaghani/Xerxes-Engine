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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMesh {
    Cube(f32),
    Sphere(f32),
    Plane(f32),
    /// A 2D sprite, as a flat quad of this size.
    Quad([f32; 2]),
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
        add(&mut view, object, None, None, prefabs, &mut synthetic);
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
            "MeshDef::Sphere" => item.mesh = Some(ViewMesh::Sphere(num("radius").unwrap_or(0.5))),
            "MeshDef::Plane" => item.mesh = Some(ViewMesh::Plane(num("size").unwrap_or(1.0))),
            "MaterialDef" | "SpriteDef" => {
                for (field, value) in fields {
                    match (field.as_str(), value) {
                        ("color", Field::Color(c)) => item.color = *c,
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
            "Camera3d" => item.camera = Some(ViewCamera::Perspective),
            "Camera2d" => item.camera = Some(ViewCamera::Orthographic),
            _ => {}
        }
    }
    view.objects.push(item);
    for (child, inside_prefab) in children {
        let child_select = if inside_prefab || select != id {
            Some(select)
        } else {
            None
        };
        add(view, child, Some(id), child_select, prefabs, synthetic);
    }
}

/// `Some(Vec2::new(x, y))` as written, for sprite sizes.
fn parse_some_vec2(src: &str) -> Option<[f32; 2]> {
    let inner = src.replace(' ', "");
    let inner = inner.strip_prefix("Some(Vec2::new(")?.strip_suffix("))")?;
    let (x, y) = inner.split_once(',')?;
    Some([x.parse().ok()?, y.parse().ok()?])
}
