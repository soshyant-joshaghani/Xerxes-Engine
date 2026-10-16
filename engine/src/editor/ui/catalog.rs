//! What the Add menu can put in a scene: the common 2D and 3D game objects, each as the
//! components a scene file would hold (so they are written and read like anything authored by
//! hand). UI elements go under a canvas; without one, a canvas is added with them.

use crate::editor::project::codec::{ComponentDoc, Field, TransformData, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddKind {
    Empty,
    // 3D
    Cube,
    Sphere,
    Plane,
    DirectionalLight,
    PointLight,
    SpotLight,
    Camera3d,
    // 2D
    Camera2d,
    Sprite,
    // UI
    Canvas,
    Panel,
    Text,
    Button,
    Toggle,
    Slider,
    Dropdown,
    Joystick,
    TouchPad,
}

impl AddKind {
    pub fn label(self) -> &'static str {
        match self {
            AddKind::Empty => "Empty",
            AddKind::Cube => "Cube",
            AddKind::Sphere => "Sphere",
            AddKind::Plane => "Plane",
            AddKind::DirectionalLight => "Directional Light",
            AddKind::PointLight => "Point Light",
            AddKind::SpotLight => "Spot Light",
            AddKind::Camera3d => "Game Camera (3D)",
            AddKind::Camera2d => "Game Camera (2D)",
            AddKind::Sprite => "Sprite",
            AddKind::Canvas => "Canvas",
            AddKind::Panel => "Panel",
            AddKind::Text => "Text",
            AddKind::Button => "Button",
            AddKind::Toggle => "Toggle",
            AddKind::Slider => "Slider",
            AddKind::Dropdown => "Drop-down",
            AddKind::Joystick => "Virtual Joystick",
            AddKind::TouchPad => "Touch Pad",
        }
    }

    /// An element that lives on a canvas (everything of the UI group but the canvas itself).
    pub fn is_ui_element(self) -> bool {
        matches!(
            self,
            AddKind::Panel
                | AddKind::Text
                | AddKind::Button
                | AddKind::Toggle
                | AddKind::Slider
                | AddKind::Dropdown
                | AddKind::Joystick
                | AddKind::TouchPad
        )
    }

    pub const THREE_D: [AddKind; 7] = [
        AddKind::Cube,
        AddKind::Sphere,
        AddKind::Plane,
        AddKind::DirectionalLight,
        AddKind::PointLight,
        AddKind::SpotLight,
        AddKind::Camera3d,
    ];
    pub const TWO_D: [AddKind; 2] = [AddKind::Camera2d, AddKind::Sprite];
    pub const UI: [AddKind; 9] = [
        AddKind::Canvas,
        AddKind::Panel,
        AddKind::Text,
        AddKind::Button,
        AddKind::Toggle,
        AddKind::Slider,
        AddKind::Dropdown,
        AddKind::Joystick,
        AddKind::TouchPad,
    ];
}

/// An object to add: its name, components and children.
#[derive(Debug, Clone)]
pub struct Spec {
    pub name: String,
    pub components: Vec<ComponentDoc>,
    pub children: Vec<Spec>,
}

fn transform(translation: [f32; 3], rotation: [f32; 3]) -> ComponentDoc {
    ComponentDoc::new(
        "Transform",
        Value::Transform(TransformData {
            translation,
            rotation,
            scale: [1.0; 3],
        }),
    )
}

fn unit(ty: &str, source: &str) -> ComponentDoc {
    ComponentDoc::new(ty, Value::Unit(source.into()))
}

fn expr(ty: &str, source: &str) -> ComponentDoc {
    ComponentDoc::new(ty, Value::Expr(source.into()))
}

fn node(fields: &[(&str, Field)]) -> ComponentDoc {
    ComponentDoc::new(
        "Node",
        Value::Struct {
            fields: fields
                .iter()
                .map(|(n, f)| (n.to_string(), f.clone()))
                .collect(),
            rest: true,
        },
    )
}

fn mesh(ty: &str, field: &str, value: f32) -> Vec<ComponentDoc> {
    vec![
        transform([0.0, 0.5, 0.0], [0.0; 3]),
        ComponentDoc::new(
            ty,
            Value::Struct {
                fields: vec![(field.to_string(), Field::Num(value))],
                rest: false,
            },
        ),
        ComponentDoc::new(
            "MaterialDef",
            Value::Struct {
                fields: vec![
                    ("color".into(), Field::Color([0.8, 0.8, 0.8, 1.0])),
                    ("texture".into(), Field::OptStr(None)),
                ],
                rest: false,
            },
        ),
    ]
}

fn anchored(size: (f32, f32)) -> ComponentDoc {
    expr(
        "RectTransform",
        &format!(
            "RectTransform::anchored(Anchor::Middle, Vec2::new(0.0, 0.0), Vec2::new({:?}, {:?}))",
            size.0, size.1
        ),
    )
}

fn text_spec(name: &str, value: &str, size: f32, color: &str) -> Spec {
    Spec {
        name: name.into(),
        components: vec![
            expr("Text", &format!("Text::new({value:?})")),
            expr("TextFont", &format!("TextFont::from_font_size({size:?})")),
            expr("TextColor", &format!("TextColor({color})")),
        ],
        children: Vec::new(),
    }
}

/// The object a kind adds.
pub fn spec(kind: AddKind) -> Spec {
    let leaf = |name: &str, components: Vec<ComponentDoc>| Spec {
        name: name.into(),
        components,
        children: Vec::new(),
    };
    match kind {
        AddKind::Empty => leaf(
            "New Object",
            vec![ComponentDoc::new(
                "Transform",
                Value::Transform(TransformData::default()),
            )],
        ),
        AddKind::Cube => leaf("Cube", mesh("MeshDef::Cube", "size", 1.0)),
        AddKind::Sphere => leaf("Sphere", mesh("MeshDef::Sphere", "radius", 0.5)),
        AddKind::Plane => leaf("Plane", {
            let mut components = mesh("MeshDef::Plane", "size", 10.0);
            components[0] = transform([0.0; 3], [0.0; 3]);
            components
        }),
        AddKind::DirectionalLight => leaf(
            "Directional Light",
            vec![
                transform([0.0, 4.0, 0.0], [-50.0, -30.0, 0.0]),
                ComponentDoc::new(
                    "DirectionalLight",
                    Value::Struct {
                        fields: vec![("shadows_enabled".into(), Field::Bool(true))],
                        rest: true,
                    },
                ),
            ],
        ),
        AddKind::PointLight => leaf(
            "Point Light",
            vec![
                transform([0.0, 3.0, 0.0], [0.0; 3]),
                unit("PointLight", "PointLight::default()"),
            ],
        ),
        AddKind::SpotLight => leaf(
            "Spot Light",
            vec![
                transform([0.0, 3.0, 0.0], [-90.0, 0.0, 0.0]),
                unit("SpotLight", "SpotLight::default()"),
            ],
        ),
        AddKind::Camera3d => leaf(
            "Camera",
            vec![
                transform([0.0, 3.0, 6.0], [-20.0, 0.0, 0.0]),
                unit("GameCamera3D", "GameCamera3D"),
            ],
        ),
        AddKind::Camera2d => leaf(
            "Camera",
            vec![transform([0.0; 3], [0.0; 3]), unit("GameCamera2D", "GameCamera2D")],
        ),
        AddKind::Sprite => leaf(
            "Sprite",
            vec![
                transform([0.0; 3], [0.0; 3]),
                ComponentDoc::new(
                    "SpriteDef",
                    Value::Struct {
                        fields: vec![
                            ("image".into(), Field::OptStr(None)),
                            ("color".into(), Field::Color([1.0; 4])),
                            (
                                "size".into(),
                                Field::Expr("Some(Vec2::new(64.0, 64.0))".into()),
                            ),
                        ],
                        rest: false,
                    },
                ),
            ],
        ),
        AddKind::Canvas => leaf("Canvas", vec![expr("Canvas", "Canvas::overlay()")]),
        AddKind::Panel => leaf(
            "Panel",
            vec![
                node(&[]),
                expr(
                    "BackgroundColor",
                    "BackgroundColor(Color::srgba(0.1, 0.11, 0.14, 0.85))",
                ),
                anchored((240.0, 140.0)),
            ],
        ),
        AddKind::Text => {
            let mut spec = text_spec("Text", "New Text", 20.0, "Color::WHITE");
            spec.components.push(anchored((0.0, 0.0)));
            spec
        }
        AddKind::Button => Spec {
            name: "Button".into(),
            components: vec![
                unit("Button", "Button"),
                node(&[
                    (
                        "justify_content",
                        Field::Path("JustifyContent::Center".into()),
                    ),
                    ("align_items", Field::Path("AlignItems::Center".into())),
                    (
                        "border_radius",
                        Field::Expr("BorderRadius::all(px(6))".into()),
                    ),
                ]),
                expr(
                    "BackgroundColor",
                    "BackgroundColor(Color::srgb(0.22, 0.74, 0.97))",
                ),
                anchored((160.0, 40.0)),
            ],
            children: vec![text_spec(
                "Label",
                "Button",
                16.0,
                "Color::srgb(0.04, 0.07, 0.12)",
            )],
        },
        AddKind::Toggle => leaf(
            "Toggle",
            vec![expr("Toggle", "Toggle::new(true)"), anchored((24.0, 24.0))],
        ),
        AddKind::Slider => leaf(
            "Slider",
            vec![
                expr("Slider", "Slider::new(0.0, 1.0, 0.5)"),
                anchored((200.0, 14.0)),
            ],
        ),
        AddKind::Joystick => leaf(
            "Move Stick",
            vec![
                expr("VirtualJoystick", "VirtualJoystick::new(80.0)"),
                expr(
                    "RectTransform",
                    "RectTransform::anchored(Anchor::BottomLeft, Vec2::new(0.0, 0.0), Vec2::new(420.0, 420.0))",
                ),
            ],
        ),
        AddKind::TouchPad => leaf(
            "Touch Pad",
            vec![
                expr("TouchPad", "TouchPad::new()"),
                expr(
                    "RectTransform",
                    "RectTransform::anchored(Anchor::BottomRight, Vec2::new(0.0, 0.0), Vec2::new(420.0, 420.0))",
                ),
            ],
        ),
        AddKind::Dropdown => leaf(
            "Dropdown",
            vec![
                expr("Dropdown", "Dropdown::new([\"Option A\", \"Option B\"])"),
                anchored((200.0, 32.0)),
            ],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::project::codec::component_source;

    #[test]
    fn every_kind_writes_valid_component_source() {
        let kinds = [AddKind::Empty]
            .into_iter()
            .chain(AddKind::THREE_D)
            .chain(AddKind::TWO_D)
            .chain(AddKind::UI);
        for kind in kinds {
            let spec = spec(kind);
            assert!(!spec.name.is_empty());
            for component in spec
                .components
                .iter()
                .chain(spec.children.iter().flat_map(|c| &c.components))
            {
                let source = component_source(component);
                assert!(
                    syn::parse_str::<syn::Expr>(&source).is_ok(),
                    "{:?}: `{source}` is not an expression",
                    kind
                );
            }
        }
    }

    /// What the Add menu writes is read back by the scene reader and shown by the Scene view:
    /// UI kinds as UI, meshes as meshes, lights and cameras as such.
    #[test]
    fn added_objects_round_trip_through_a_scene_file() {
        use crate::editor::bridge::view;
        use crate::editor::project::codec::{ObjectDoc, object_source, parse_scene};
        use std::collections::HashMap;

        fn object(spec: &Spec) -> ObjectDoc {
            ObjectDoc {
                name: spec.name.clone(),
                components: spec.components.clone(),
                children: spec.children.iter().map(object).collect(),
                ..Default::default()
            }
        }
        let kinds = [AddKind::Empty]
            .into_iter()
            .chain(AddKind::THREE_D)
            .chain(AddKind::TWO_D)
            .chain(AddKind::UI);
        let objects: Vec<String> = kinds.map(|k| object_source(&object(&spec(k)))).collect();
        let src = format!(
            "pub fn scene() -> SceneAsset {{ SceneAsset {{ dimension: Dimension::D3, objects: vec![{}] }} }}",
            objects.join(", ")
        );
        let doc = parse_scene(&src).expect("the scene parses");
        let view = view::build(&doc, &HashMap::new());
        let find = |name: &str| view.objects.iter().find(|o| o.name == name).unwrap();
        assert!(find("Cube").mesh.is_some());
        assert!(find("Directional Light").light.is_some());
        assert!(find("Camera").camera.is_some());
        assert!(find("Sprite").mesh.is_some());
        assert!(find("Canvas").ui.as_ref().unwrap().canvas.is_some());
        assert!(find("Panel").ui.as_ref().unwrap().rect.is_some());
        assert_eq!(
            find("Text").ui.as_ref().unwrap().text.as_deref(),
            Some("New Text")
        );
        assert!(find("Button").ui.as_ref().unwrap().button);
        assert!(find("Toggle").ui.as_ref().unwrap().widget.is_some());
        assert!(find("Slider").ui.as_ref().unwrap().widget.is_some());
        assert!(find("Dropdown").ui.as_ref().unwrap().widget.is_some());
        assert!(find("Move Stick").ui.as_ref().unwrap().widget.is_some());
        assert!(find("Touch Pad").ui.as_ref().unwrap().widget.is_some());
    }
}
