//! Editors for the components the Properties panel knows by name, like Unity's inspectors:
//! the Canvas (render mode, camera, sort order, size), anchoring (`RectTransform`, with the
//! nine presets and stretch), the sprite (image, color, size) and the camera (2D or 3D,
//! active, perspective or orthographic). Each reads the component as the scene wrote it and
//! writes it back as source, so the file stays plain data.

use dioxus::prelude::*;

use super::{Editor, edit};
use crate::editor::bridge::ui_view::{
    ViewCanvas, ViewCanvasMode, ViewRect, ViewVal, ViewXform, canvas_of, canvas_source,
    color_hex, color_source, component_color, const_color, parse_hex, parse_radius, parse_ui_rect,
    parse_val, radius_source, rect_of, text_of, text_size_of, ui_rect_source, val_source,
    xform_component, xform_of,
};
use crate::editor::project::codec::{ComponentDoc, Field, Value};

fn base(ty: &str) -> &str {
    ty.split("::").next().unwrap_or(ty)
}

/// A number input that applies when the text parses.
#[component]
fn Num(value: f32, on_value: EventHandler<f32>) -> Element {
    rsx! {
        input {
            class: "num",
            value: "{super::inspector::num(value)}",
            oninput: move |e| {
                if let Ok(v) = e.value().trim().parse::<f32>() {
                    if v.is_finite() {
                        on_value.call(v);
                    }
                }
            },
        }
    }
}

#[component]
fn Row(label: String, children: Element) -> Element {
    rsx! {
        div { class: "field",
            span { class: "label", "{label}" }
            div { class: "value", {children} }
        }
    }
}

/// Which component editor a component gets, if any.
pub fn has_editor(ty: &str) -> bool {
    matches!(
        base(ty),
        "Canvas"
            | "RectTransform"
            | "SpriteDef"
            | "Camera2d"
            | "Camera3d"
            | "GameCamera2D"
            | "GameCamera3D"
            | "Node"
            | "BackgroundColor"
            | "BorderColor"
            | "TextColor"
            | "Text"
            | "TextFont"
    )
}

#[component]
pub fn ComponentEditor(id: u32, component: ComponentDoc) -> Element {
    match base(&component.ty) {
        "Canvas" => rsx! { CanvasEditor { id, component } },
        "RectTransform" => rsx! { RectEditor { id, component } },
        "SpriteDef" => rsx! { SpriteEditor { id, component } },
        "Node" => rsx! { NodeEditor { id, component } },
        "BackgroundColor" | "BorderColor" | "TextColor" => rsx! { ColorEditor { id, component } },
        "Text" => rsx! { TextEditor { id, component } },
        "TextFont" => rsx! { FontEditor { id, component } },
        "Camera2d" | "Camera3d" | "GameCamera2D" | "GameCamera3D" => {
            rsx! { CameraEditor { id, component } }
        }
        _ => rsx! {},
    }
}

// ---- Canvas -----------------------------------------------------------------------------

#[component]
fn CanvasEditor(id: u32, component: ComponentDoc) -> Element {
    let editor = use_context::<Editor>();
    let Some(canvas) = canvas_of(&component) else {
        return rsx! { div { class: "field", span { class: "code", "unreadable Canvas" } } };
    };
    let project = *editor.reference.read();
    let cameras: Vec<String> = editor
        .scene
        .read()
        .as_ref()
        .map(|doc| {
            doc.all()
                .into_iter()
                .filter(|o| {
                    o.components
                        .iter()
                        .any(|c| {
                            matches!(
                                base(&c.ty),
                                "Camera2d" | "Camera3d" | "GameCamera2D" | "GameCamera3D"
                            )
                        })
                })
                .map(|o| o.name.clone())
                .collect()
        })
        .unwrap_or_default();
    let write = move |next: ViewCanvas| {
        edit::set_component(
            editor,
            id,
            "Canvas",
            ComponentDoc::new("Canvas", Value::Expr(canvas_source(&next))),
        );
    };
    let world = matches!(canvas.mode, ViewCanvasMode::World { .. });
    let is_camera = canvas.mode == ViewCanvasMode::Camera;
    let overridden = canvas.reference.is_some();
    let reference = canvas.reference.unwrap_or(project);
    let (size, ppu) = match canvas.mode {
        ViewCanvasMode::World {
            size,
            pixels_per_unit,
        } => (size, pixels_per_unit),
        _ => (project, 1.0),
    };
    let modes: [(&str, ViewCanvasMode); 3] = [
        ("Screen Space - Overlay", ViewCanvasMode::Overlay),
        ("Screen Space - Camera", ViewCanvasMode::Camera),
        (
            "World Space",
            ViewCanvasMode::World {
                size,
                pixels_per_unit: ppu,
            },
        ),
    ];
    let shown_camera = canvas.camera.clone();
    rsx! {
        Row { label: "Render Mode".to_string(),
            div { class: "vstack",
                for (label , mode) in modes {
                    button {
                        key: "{label}",
                        class: if std::mem::discriminant(&canvas.mode) == std::mem::discriminant(&mode) { "on" } else { "" },
                        onclick: {
                            let canvas = canvas.clone();
                            move |_| {
                                let mut next = canvas.clone();
                                next.mode = mode;
                                write(next);
                            }
                        },
                        "{label}"
                    }
                }
            }
        }
        Row { label: "Sort Order".to_string(),
            Num {
                value: canvas.sort_order as f32,
                on_value: {
                    let canvas = canvas.clone();
                    move |n: f32| {
                        let mut next = canvas.clone();
                        next.sort_order = n as i32;
                        write(next);
                    }
                },
            }
        }
        if is_camera {
            Row { label: "Render Camera".to_string(),
                div { class: "vstack",
                    button {
                        class: if shown_camera.is_none() { "on" } else { "" },
                        onclick: {
                            let canvas = canvas.clone();
                            move |_| {
                                let mut next = canvas.clone();
                                next.camera = None;
                                write(next);
                            }
                        },
                        "Active camera"
                    }
                    for name in cameras {
                        button {
                            key: "{name}",
                            class: if shown_camera.as_deref() == Some(name.as_str()) { "on" } else { "" },
                            onclick: {
                                let canvas = canvas.clone();
                                let name = name.clone();
                                move |_| {
                                    let mut next = canvas.clone();
                                    next.camera = Some(name.clone());
                                    write(next);
                                }
                            },
                            "{name}"
                        }
                    }
                }
            }
        }
        if world {
            Row { label: "Size".to_string(),
                span { class: "axis", "W" }
                Num {
                    value: size[0] as f32,
                    on_value: {
                        let canvas = canvas.clone();
                        move |n: f32| {
                            let mut next = canvas.clone();
                            next.mode = ViewCanvasMode::World { size: [n.max(1.0) as u32, size[1]], pixels_per_unit: ppu };
                            write(next);
                        }
                    },
                }
                span { class: "axis", "H" }
                Num {
                    value: size[1] as f32,
                    on_value: {
                        let canvas = canvas.clone();
                        move |n: f32| {
                            let mut next = canvas.clone();
                            next.mode = ViewCanvasMode::World { size: [size[0], n.max(1.0) as u32], pixels_per_unit: ppu };
                            write(next);
                        }
                    },
                }
            }
            Row { label: "Pixels Per Unit".to_string(),
                Num {
                    value: ppu,
                    on_value: {
                        let canvas = canvas.clone();
                        move |n: f32| {
                            let mut next = canvas.clone();
                            next.mode = ViewCanvasMode::World { size, pixels_per_unit: n.max(0.01) };
                            write(next);
                        }
                    },
                }
            }
        } else {
            Row { label: "Reference Size".to_string(),
                button {
                    class: if overridden { "" } else { "on" },
                    title: "The project's window size (Project Settings)",
                    onclick: {
                        let canvas = canvas.clone();
                        move |_| {
                            let mut next = canvas.clone();
                            next.reference = None;
                            write(next);
                        }
                    },
                    "Project {project[0]} x {project[1]}"
                }
                button {
                    class: if overridden { "on" } else { "" },
                    onclick: {
                        let canvas = canvas.clone();
                        move |_| {
                            let mut next = canvas.clone();
                            next.reference = Some(project);
                            write(next);
                        }
                    },
                    "Custom"
                }
            }
            if overridden {
                Row { label: "".to_string(),
                    span { class: "axis", "W" }
                    Num {
                        value: reference[0] as f32,
                        on_value: {
                            let canvas = canvas.clone();
                            move |n: f32| {
                                let mut next = canvas.clone();
                                next.reference = Some([n.max(1.0) as u32, reference[1]]);
                                write(next);
                            }
                        },
                    }
                    span { class: "axis", "H" }
                    Num {
                        value: reference[1] as f32,
                        on_value: {
                            let canvas = canvas.clone();
                            move |n: f32| {
                                let mut next = canvas.clone();
                                next.reference = Some([reference[0], n.max(1.0) as u32]);
                                write(next);
                            }
                        },
                    }
                }
            }
        }
    }
}

// ---- RectTransform (anchoring) -------------------------------------------------------------

fn rect_component(rect: ViewRect) -> ComponentDoc {
    let v = Field::Vec2;
    ComponentDoc::new(
        "RectTransform",
        Value::Struct {
            fields: vec![
                ("anchor_min".into(), v(rect.anchor_min)),
                ("anchor_max".into(), v(rect.anchor_max)),
                ("pivot".into(), v(rect.pivot)),
                ("position".into(), v(rect.position)),
                ("size".into(), v(rect.size)),
            ],
            rest: false,
        },
    )
}

#[component]
fn RectEditor(id: u32, component: ComponentDoc) -> Element {
    let editor = use_context::<Editor>();
    let Some(rect) = rect_of(&component) else {
        return rsx! { div { class: "field", span { class: "code", "unreadable RectTransform" } } };
    };
    let write = move |next: ViewRect| {
        edit::set_component(editor, id, "Anchors", rect_component(next));
    };
    // The nine presets, top row first (y up: 1 is the top).
    let presets: [[(f32, f32); 3]; 3] = [
        [(0.0, 1.0), (0.5, 1.0), (1.0, 1.0)],
        [(0.0, 0.5), (0.5, 0.5), (1.0, 0.5)],
        [(0.0, 0.0), (0.5, 0.0), (1.0, 0.0)],
    ];
    let stretched = rect.anchor_min != rect.anchor_max;
    let point = |p: (f32, f32)| [p.0, p.1];
    let field = move |label: &'static str,
                      get: fn(&ViewRect) -> [f32; 2],
                      set: fn(&mut ViewRect, [f32; 2])| {
        let v = get(&rect);
        rsx! {
            Row { label: label.to_string(),
                span { class: "axis", "X" }
                Num {
                    value: v[0],
                    on_value: move |n: f32| {
                        let mut next = rect;
                        let mut value = get(&next);
                        value[0] = n;
                        set(&mut next, value);
                        write(next);
                    },
                }
                span { class: "axis", "Y" }
                Num {
                    value: v[1],
                    on_value: move |n: f32| {
                        let mut next = rect;
                        let mut value = get(&next);
                        value[1] = n;
                        set(&mut next, value);
                        write(next);
                    },
                }
            }
        }
    };
    rsx! {
        Row { label: "Anchor Preset".to_string(),
            div { class: "vstack",
                for (row , cells) in presets.into_iter().enumerate() {
                    div { key: "{row}", class: "hstack",
                        for (col , cell) in cells.into_iter().enumerate() {
                            button {
                                key: "{col}",
                                class: if !stretched && rect.anchor_min == point(cell) { "on" } else { "" },
                                title: "Anchor to this point of the parent",
                                onclick: move |_| {
                                    let mut next = rect;
                                    next.anchor_min = point(cell);
                                    next.anchor_max = point(cell);
                                    next.pivot = point(cell);
                                    write(next);
                                },
                                "o"
                            }
                        }
                    }
                }
                button {
                    class: if rect.anchor_min == [0.0, 0.0] && rect.anchor_max == [1.0, 1.0] { "on" } else { "" },
                    title: "Fill the parent",
                    onclick: move |_| {
                        write(ViewRect {
                            anchor_min: [0.0, 0.0],
                            anchor_max: [1.0, 1.0],
                            pivot: [0.5, 0.5],
                            position: [0.0, 0.0],
                            size: [0.0, 0.0],
                        });
                    },
                    "Stretch"
                }
            }
        }
        {field("Position", |r| r.position, |r, v| r.position = v)}
        {field(if stretched { "Size Delta" } else { "Size (0: content)" }, |r| r.size, |r, v| r.size = v)}
        {field("Anchor Min", |r| r.anchor_min, |r, v| r.anchor_min = v)}
        {field("Anchor Max", |r| r.anchor_max, |r, v| r.anchor_max = v)}
        {field("Pivot", |r| r.pivot, |r, v| r.pivot = v)}
    }
}

// ---- Sprite ------------------------------------------------------------------------------

#[component]
fn SpriteEditor(id: u32, component: ComponentDoc) -> Element {
    let editor = use_context::<Editor>();
    let Value::Struct { fields, .. } = &component.value else {
        return rsx! { div { class: "field", span { class: "code", "unreadable SpriteDef" } } };
    };
    let consts: Vec<(String, String)> = editor
        .scene
        .read()
        .as_ref()
        .map(|d| d.consts.clone())
        .unwrap_or_default();
    let get = |name: &str| {
        fields
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, f)| f.clone())
    };
    let image = match get("image") {
        Some(Field::OptStr(s)) => s,
        _ => None,
    };
    // A constant (`color: PLAYER`) shows by name and can be made a literal to edit.
    let (color, constant) = match get("color") {
        Some(Field::Color(c)) => (c, None),
        Some(Field::Path(name)) => (const_color(&name, &consts).unwrap_or([1.0; 4]), Some(name)),
        _ => ([1.0; 4], None),
    };
    let size = match get("size") {
        Some(Field::Expr(src)) => crate::editor::view::parse_some_vec2(&src),
        _ => None,
    };
    let set = move |name: &'static str, field: Field| {
        edit::set_field(editor, id, "SpriteDef", None, name, field)
    };
    let css = format!(
        "background: rgba({}, {}, {}, {});",
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        color[3]
    );
    rsx! {
        Row { label: "Sprite".to_string(),
            input {
                class: "num",
                value: "{image.clone().unwrap_or_default()}",
                placeholder: "none: a plain color",
                oninput: move |e| {
                    let v = e.value();
                    set("image", Field::OptStr(if v.trim().is_empty() { None } else { Some(v) }));
                },
            }
        }
        Row { label: if image.is_some() { "Color (tint)".to_string() } else { "Color".to_string() },
            span { class: "swatch", style: "{css}" }
            if let Some(name) = constant {
                span { class: "code", "{name}" }
                button {
                    onclick: move |_| set("color", Field::Color(color)),
                    "Edit"
                }
            } else {
                for (i , channel) in ["R", "G", "B", "A"].into_iter().enumerate() {
                    span { key: "{channel}", class: "axis", "{channel}" }
                    Num {
                        value: color[i],
                        on_value: move |n: f32| {
                            let mut next = color;
                            next[i] = n.clamp(0.0, 1.0);
                            set("color", Field::Color(next));
                        },
                    }
                }
            }
        }
        Row { label: "Size".to_string(),
            if let Some(s) = size {
                span { class: "axis", "W" }
                Num {
                    value: s[0],
                    on_value: move |n: f32| {
                        set("size", Field::Expr(format!("Some(Vec2::new({:?}, {:?}))", n.max(1.0), s[1])));
                    },
                }
                span { class: "axis", "H" }
                Num {
                    value: s[1],
                    on_value: move |n: f32| {
                        set("size", Field::Expr(format!("Some(Vec2::new({:?}, {:?}))", s[0], n.max(1.0))));
                    },
                }
            } else {
                span { class: "muted", "the image's size" }
                button {
                    onclick: move |_| set("size", Field::Expr("Some(Vec2::new(64.0, 64.0))".into())),
                    "Set"
                }
            }
        }
    }
}

// ---- Camera ------------------------------------------------------------------------------

#[component]
fn CameraEditor(id: u32, component: ComponentDoc) -> Element {
    let editor = use_context::<Editor>();
    let is_2d = matches!(base(&component.ty), "Camera2d" | "GameCamera2D");
    let object = editor
        .scene
        .read()
        .as_ref()
        .and_then(|doc| doc.find(id).cloned());
    let active = object.as_ref().is_none_or(|o| {
        !o.components.iter().any(|c| {
            base(&c.ty) == "Camera"
                && matches!(&c.value, Value::Struct { fields, .. }
                    if fields.iter().any(|(n, f)| n == "is_active" && *f == Field::Bool(false)))
        })
    });
    let orthographic = is_2d
        || object.as_ref().is_some_and(|o| {
            o.components.iter().any(|c| {
                base(&c.ty) == "Projection"
                    && matches!(&c.value, Value::Expr(s) if s.contains("Orthographic"))
            })
        });
    rsx! {
        Row { label: "Type".to_string(),
            span { class: "muted", if is_2d { "2D camera" } else { "3D camera" } }
        }
        Row { label: "Active".to_string(),
            button {
                class: if active { "on" } else { "" },
                title: "An inactive camera does not render",
                onclick: move |_| edit::set_camera_active(editor, id, !active),
                if active { "true" } else { "false" }
            }
        }
        Row { label: "Projection".to_string(),
            button {
                class: if !orthographic { "on" } else { "" },
                onclick: move |_| edit::set_camera_projection(editor, id, false),
                "Perspective"
            }
            button {
                class: if orthographic { "on" } else { "" },
                onclick: move |_| edit::set_camera_projection(editor, id, true),
                "Orthographic"
            }
        }
    }
}

// ---- Node (flexbox layout) ----------------------------------------------------------------

/// One-of choice: a button per option, the current one lit. An option is (label, source).
#[component]
fn Choice(
    label: String,
    options: Vec<(String, String)>,
    current: String,
    on_pick: EventHandler<String>,
) -> Element {
    rsx! {
        Row { label,
            div { class: "hstack",
                for (name , source) in options {
                    button {
                        key: "{name}",
                        class: if source.rsplit("::").next() == Some(current.as_str()) { "on" } else { "" },
                        onclick: {
                            let source = source.clone();
                            move |_| on_pick.call(source.clone())
                        },
                        "{name}"
                    }
                }
            }
        }
    }
}

/// A `Val`: auto, pixels or percent, with its number.
#[component]
fn ValRow(label: String, current: ViewVal, on_val: EventHandler<ViewVal>) -> Element {
    let amount = match current {
        ViewVal::Px(v) | ViewVal::Percent(v) | ViewVal::Vw(v) | ViewVal::Vh(v) => v,
        ViewVal::Auto => 0.0,
    };
    let kind = match current {
        ViewVal::Auto => "auto",
        ViewVal::Px(_) => "px",
        ViewVal::Percent(_) => "%",
        ViewVal::Vw(_) => "vw",
        ViewVal::Vh(_) => "vh",
    };
    let make = move |kind: &str, v: f32| match kind {
        "px" => ViewVal::Px(v),
        "%" => ViewVal::Percent(v),
        "vw" => ViewVal::Vw(v),
        "vh" => ViewVal::Vh(v),
        _ => ViewVal::Auto,
    };
    rsx! {
        Row { label,
            for k in ["auto", "px", "%"] {
                button {
                    key: "{k}",
                    class: if kind == k { "on" } else { "" },
                    onclick: move |_| on_val.call(make(k, amount)),
                    "{k}"
                }
            }
            if current != ViewVal::Auto {
                Num { value: amount, on_value: move |n: f32| on_val.call(make(kind, n)) }
            }
        }
    }
}

/// Four pixel sides: left, right, top, bottom.
#[component]
fn SidesRow(label: String, current: [f32; 4], on_sides: EventHandler<[f32; 4]>) -> Element {
    rsx! {
        Row { label,
            for (i , axis) in ["L", "R", "T", "B"].into_iter().enumerate() {
                span { key: "{axis}", class: "axis", "{axis}" }
                Num {
                    value: current[i],
                    on_value: move |n: f32| {
                        let mut next = current;
                        next[i] = n.max(0.0);
                        on_sides.call(next);
                    },
                }
            }
        }
    }
}

fn options(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(l, s)| (l.to_string(), s.to_string()))
        .collect()
}

/// A flexbox `Node`: how the element is sized and placed by its parent, and how it lays out its
/// own children. The values are the `Node`'s fields as written in the scene, so what is edited
/// here is what the game runs.
#[component]
fn NodeEditor(id: u32, component: ComponentDoc) -> Element {
    let editor = use_context::<Editor>();
    let Value::Struct { fields, .. } = &component.value else {
        return rsx! {
            div { class: "field", span { class: "code", "Node::default()" } }
            div { class: "field muted", "no fields set: defaults" }
        };
    };
    let source = |name: &str| -> Option<String> {
        fields
            .iter()
            .find(|(n, _)| n == name)
            .and_then(|(_, f)| match f {
                Field::Expr(s) | Field::Path(s) => Some(s.clone()),
                Field::Num(n) => Some(format!("{n:?}")),
                Field::Int(i) => Some(i.to_string()),
                _ => None,
            })
    };
    let val_of = |name: &str| {
        source(name)
            .and_then(|s| parse_val(&s))
            .unwrap_or(ViewVal::Auto)
    };
    let sides_of = |name: &str| {
        source(name)
            .and_then(|s| parse_ui_rect(&s))
            .unwrap_or([0.0; 4])
    };
    let variant = |name: &str, default: &str| {
        source(name)
            .and_then(|s| s.rsplit("::").next().map(str::to_string))
            .unwrap_or_else(|| default.to_string())
    };
    let set = move |name: &'static str, source: String| {
        edit::set_field(editor, id, "Node", None, name, Field::Expr(source));
    };
    let absolute = variant("position_type", "Relative") == "Absolute";
    let radius = source("border_radius")
        .and_then(|s| parse_radius(&s))
        .unwrap_or(0.0);
    let xform = editor
        .scene
        .read()
        .as_ref()
        .and_then(|d| d.find(id).cloned())
        .and_then(|o| o.component("UiTransform").and_then(xform_of))
        .unwrap_or_default();
    let write_xform = move |next: ViewXform| {
        edit::set_component(editor, id, "Rotation / Scale", xform_component(next));
    };
    rsx! {
        Choice {
            label: "Position".to_string(),
            options: options(&[("In flow", "PositionType::Relative"), ("Absolute", "PositionType::Absolute")]),
            current: variant("position_type", "Relative"),
            on_pick: move |s: String| set("position_type", s),
        }
        ValRow { label: "Width".to_string(), current: val_of("width"), on_val: move |v| set("width", val_source(v)) }
        ValRow { label: "Height".to_string(), current: val_of("height"), on_val: move |v| set("height", val_source(v)) }
        ValRow {
            label: (if absolute { "Left" } else { "Offset X (left)" }).to_string(),
            current: val_of("left"),
            on_val: move |v| set("left", val_source(v)),
        }
        ValRow {
            label: (if absolute { "Top" } else { "Offset Y (top)" }).to_string(),
            current: val_of("top"),
            on_val: move |v| set("top", val_source(v)),
        }
        if absolute {
            ValRow { label: "Right".to_string(), current: val_of("right"), on_val: move |v| set("right", val_source(v)) }
            ValRow { label: "Bottom".to_string(), current: val_of("bottom"), on_val: move |v| set("bottom", val_source(v)) }
        }
        Row { label: "Rotation".to_string(),
            Num {
                value: xform.rotation,
                on_value: move |n: f32| write_xform(ViewXform { rotation: n, ..xform }),
            }
            span { class: "muted", "deg" }
        }
        Row { label: "Scale".to_string(),
            span { class: "axis", "X" }
            Num {
                value: xform.scale[0],
                on_value: move |n: f32| write_xform(ViewXform { scale: [n, xform.scale[1]], ..xform }),
            }
            span { class: "axis", "Y" }
            Num {
                value: xform.scale[1],
                on_value: move |n: f32| write_xform(ViewXform { scale: [xform.scale[0], n], ..xform }),
            }
        }
        div { class: "section-head", "Layout of children" }
        Choice {
            label: "Children".to_string(),
            options: options(&[
                ("Row", "FlexDirection::Row"),
                ("Column", "FlexDirection::Column"),
                ("Row rev", "FlexDirection::RowReverse"),
                ("Col rev", "FlexDirection::ColumnReverse"),
            ]),
            current: variant("flex_direction", "Row"),
            on_pick: move |s: String| set("flex_direction", s),
        }
        Choice {
            label: "Justify".to_string(),
            options: options(&[
                ("Start", "JustifyContent::FlexStart"),
                ("Center", "JustifyContent::Center"),
                ("End", "JustifyContent::FlexEnd"),
                ("Between", "JustifyContent::SpaceBetween"),
                ("Around", "JustifyContent::SpaceAround"),
                ("Evenly", "JustifyContent::SpaceEvenly"),
            ]),
            current: variant("justify_content", "Default"),
            on_pick: move |s: String| set("justify_content", s),
        }
        Choice {
            label: "Align".to_string(),
            options: options(&[
                ("Start", "AlignItems::FlexStart"),
                ("Center", "AlignItems::Center"),
                ("End", "AlignItems::FlexEnd"),
                ("Stretch", "AlignItems::Stretch"),
            ]),
            current: variant("align_items", "Default"),
            on_pick: move |s: String| set("align_items", s),
        }
        ValRow { label: "Row Gap".to_string(), current: val_of("row_gap"), on_val: move |v| set("row_gap", val_source(v)) }
        ValRow { label: "Column Gap".to_string(), current: val_of("column_gap"), on_val: move |v| set("column_gap", val_source(v)) }
        SidesRow {
            label: "Padding".to_string(),
            current: sides_of("padding"),
            on_sides: move |r: [f32; 4]| set("padding", ui_rect_source(r)),
        }
        SidesRow {
            label: "Margin".to_string(),
            current: sides_of("margin"),
            on_sides: move |r: [f32; 4]| set("margin", ui_rect_source(r)),
        }
        SidesRow {
            label: "Border".to_string(),
            current: sides_of("border"),
            on_sides: move |r: [f32; 4]| set("border", ui_rect_source(r)),
        }
        Row { label: "Corner Radius".to_string(),
            Num { value: radius, on_value: move |n: f32| set("border_radius", radius_source(n.max(0.0))) }
        }
    }
}

// ---- Colors, text and font ----------------------------------------------------------------

/// Which element's color the picker window edits: the object and the color component's type
/// (`BackgroundColor`, `BorderColor` or `TextColor`).
#[derive(Debug, Clone, PartialEq)]
pub struct ColorTarget {
    pub id: u32,
    pub kind: String,
}

/// Written back the way a scene writes each of them.
fn color_component(kind: &str, c: [f32; 4]) -> (&'static str, ComponentDoc) {
    let src = color_source(c);
    match kind {
        "BorderColor" => (
            "Border Color",
            ComponentDoc::new("BorderColor::all", Value::Expr(format!("BorderColor::all({src})"))),
        ),
        "TextColor" => (
            "Text Color",
            ComponentDoc::new("TextColor", Value::Expr(format!("TextColor({src})"))),
        ),
        _ => (
            "Background Color",
            ComponentDoc::new(
                "BackgroundColor",
                Value::Expr(format!("BackgroundColor({src})")),
            ),
        ),
    }
}

fn write_color(editor: Editor, id: u32, kind: &str, c: [f32; 4]) {
    let (label, component) = color_component(kind, c);
    edit::set_component(editor, id, label, component);
}

/// `BackgroundColor`, `BorderColor` or `TextColor`: a swatch that opens the picker window and
/// a hex field.
#[component]
fn ColorEditor(id: u32, component: ComponentDoc) -> Element {
    let editor = use_context::<Editor>();
    let consts: Vec<(String, String)> = editor
        .scene
        .read()
        .as_ref()
        .map(|d| d.consts.clone())
        .unwrap_or_default();
    let kind = base(&component.ty).to_string();
    let Some((color, constant)) = component_color(&component, &consts) else {
        return rsx! { div { class: "field", span { class: "code", "unreadable {kind}" } } };
    };
    let css = format!(
        "background: rgba({}, {}, {}, {});",
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        color[3]
    );
    let open_kind = kind.clone();
    let write_kind = kind.clone();
    let literal_kind = kind.clone();
    rsx! {
        Row { label: "Color".to_string(),
            // A button, not a span: it keeps its size and its fill in Blitz too.
            button {
                class: "swatch pick",
                style: "{css}",
                title: "Open the color picker",
                onclick: move |_| {
                    let target = ColorTarget { id, kind: open_kind.clone() };
                    let mut window = editor.color_window;
                    let same = window.peek().as_ref() == Some(&target);
                    window.set(if same { None } else { Some(target) });
                },
            }
            if let Some(name) = constant {
                span { class: "code", "{name}" }
                button {
                    title: "Make it a color of this element alone",
                    onclick: move |_| write_color(editor, id, &literal_kind, color),
                    "Edit"
                }
            } else {
                input {
                    class: "num",
                    value: "{color_hex(color)}",
                    oninput: move |e| {
                        if let Some(c) = parse_hex(&e.value()) {
                            write_color(editor, id, &write_kind, c);
                        }
                    },
                }
            }
        }
    }
}

/// The color picker as a window over the editor (Unity's): it floats by the right edge so the
/// Scene view stays visible, follows the element's color while it is edited and closes with
/// its button or when another object is selected.
#[component]
pub fn ColorWindow(target: ColorTarget, width: f32) -> Element {
    const WIDTH: f32 = 264.0;
    let editor = use_context::<Editor>();
    let consts: Vec<(String, String)> = editor
        .scene
        .read()
        .as_ref()
        .map(|d| d.consts.clone())
        .unwrap_or_default();
    let object = editor
        .scene
        .read()
        .as_ref()
        .and_then(|d| d.find(target.id).cloned());
    let current = object
        .as_ref()
        .and_then(|o| {
            o.components
                .iter()
                .find(|c| base(&c.ty) == target.kind)
                .and_then(|c| component_color(c, &consts))
        })
        .map(|(c, _)| c);
    let Some(color) = current else {
        return rsx! {};
    };
    let left = (width - WIDTH - 14.0).max(8.0);
    let title = match target.kind.as_str() {
        "BorderColor" => "Border Color",
        "TextColor" => "Text Color",
        _ => "Background Color",
    };
    let name = object.map(|o| o.name).unwrap_or_default();
    let kind = target.kind.clone();
    let id = target.id;
    rsx! {
        div {
            class: "cp-window",
            style: "left: {left}px; top: 64px; width: {WIDTH}px;",
            "data-interactive": "true",
            div { class: "cp-title",
                span { class: "grow", "{title}: {name}" }
                button {
                    onclick: move |_| {
                        let mut window = editor.color_window;
                        window.set(None);
                    },
                    "Close"
                }
            }
            super::colorpicker::ColorPicker {
                color,
                on_change: move |c: [f32; 4]| write_color(editor, id, &kind, c),
            }
        }
    }
}

/// `Text::new("..")`: the string.
#[component]
fn TextEditor(id: u32, component: ComponentDoc) -> Element {
    let editor = use_context::<Editor>();
    let Some(text) = text_of(&component) else {
        return rsx! { div { class: "field", span { class: "code", "unreadable Text" } } };
    };
    rsx! {
        Row { label: "Text".to_string(),
            input {
                class: "num",
                value: "{text}",
                oninput: move |e| {
                    edit::set_component(
                        editor,
                        id,
                        "Text",
                        ComponentDoc::new("Text", Value::Expr(format!("Text::new({:?})", e.value()))),
                    );
                },
            }
        }
    }
}

/// `TextFont`: the size. (Which font is a project asset, not built yet.)
#[component]
fn FontEditor(id: u32, component: ComponentDoc) -> Element {
    let editor = use_context::<Editor>();
    let Some(size) = text_size_of(&component) else {
        return rsx! { div { class: "field", span { class: "code", "unreadable TextFont" } } };
    };
    let is_struct = matches!(component.value, Value::Struct { .. });
    rsx! {
        Row { label: "Size".to_string(),
            Num {
                value: size,
                on_value: move |n: f32| {
                    let n = n.max(1.0);
                    if is_struct {
                        edit::set_field(editor, id, "TextFont", None, "font_size", Field::Num(n));
                    } else {
                        edit::set_component(
                            editor,
                            id,
                            "Font size",
                            ComponentDoc::new(
                                "TextFont",
                                Value::Expr(format!("TextFont::from_font_size({n:?})")),
                            ),
                        );
                    }
                },
            }
        }
        Row { label: "Font".to_string(),
            span { class: "muted", "the engine's default (font assets are not built yet)" }
        }
    }
}
