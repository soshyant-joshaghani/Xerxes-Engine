//! The Hierarchy: the open scene's objects as a tree. Click to select (synced with the
//! Scene view and the Inspector). Prefab instances show in blue.

use dioxus::prelude::*;

use dioxus::html::ModifiersInteraction;

use super::Editor;
use crate::editor::project::codec::{Dimension, Field, ObjectDoc, Value};

fn base(ty: &str) -> &str {
    ty.split("::").next().unwrap_or(ty)
}

fn has_component(object: &ObjectDoc, bases: &[&str]) -> bool {
    object
        .components
        .iter()
        .any(|c| bases.contains(&base(&c.ty)))
}

#[component]
pub fn Hierarchy() -> Element {
    let editor = use_context::<Editor>();
    let scene = editor.scene.read().clone();
    let title = editor.scene_path.read().clone().map(|p| {
        p.rsplit('/')
            .next()
            .unwrap_or(&p)
            .trim_end_matches(".scene.rs")
            .to_string()
    });
    rsx! {
        div { class: "actions",
            button { disabled: editor.scene.read().is_none(), onclick: move |_| super::edit::add_empty(editor, *editor.selected.peek()), "+ Empty" }
            button { disabled: editor.selected.read().is_none(), onclick: move |_| if let Some(id) = *editor.selected.peek() { super::edit::duplicate(editor, id) }, "Duplicate" }
            button { disabled: editor.selected.read().is_none(), onclick: move |_| if let Some(id) = *editor.selected.peek() { super::edit::delete(editor, id) }, "Delete" }
        }
        div {
            class: "body",
            onmouseup: move |_| {
                let mut dragging = editor.dragging;
                if dragging.peek().is_some() {
                    dragging.set(None);
                }
            },
            match (scene, title) {
                (Some(scene), Some(title)) => rsx! {
                    div { class: "row", style: "padding-left: 6px;",
                        span { class: "grow", style: "font-weight: 600;", "{title}" }
                        // Scenes are 2D or 3D (not games): this is the scene's `dimension`.
                        for (dimension, label) in [(Dimension::D2, "2D"), (Dimension::D3, "3D")] {
                            button {
                                key: "{label}",
                                class: if scene.dimension == dimension { "dim on" } else { "dim" },
                                onclick: move |_| super::edit::set_dimension(editor, dimension),
                                "{label}"
                            }
                        }
                    }
                    for object in scene.objects {
                        Node { key: "{object.id}", object, depth: 1 }
                    }
                    // While a row is being moved: a place to drop it at the scene's root.
                    if editor.dragging.read().is_some() {
                        div {
                            class: "row drop",
                            onmouseup: move |_| {
                                let source = *editor.dragging.peek();
                                let mut dragging = editor.dragging;
                                dragging.set(None);
                                if let Some(source) = source {
                                    super::edit::reparent(editor, source, None);
                                }
                            },
                            "Drop here to move it to the scene root"
                        }
                    }
                },
                _ => rsx! { div { class: "hint", "Open a scene from the Project browser." } },
            }
        }
    }
}

/// The name shown for an object: its own, or its prefab's file name.
pub fn display_name(
    object: &ObjectDoc,
    prefabs: &std::collections::HashMap<String, ObjectDoc>,
) -> String {
    if !object.name.is_empty() {
        return object.name.clone();
    }
    object
        .prefab
        .as_ref()
        .and_then(|p| prefabs.get(p).map(|root| root.name.clone()))
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "(prefab)".into())
}

#[component]
fn Node(object: ObjectDoc, depth: u32) -> Element {
    let editor = use_context::<Editor>();
    let stage = use_context::<ReadSignal<crate::editor::protocol::EditorSnapshot>>();
    let primary = *editor.selected.read() == Some(object.id);
    let also = editor.also.read().contains(&object.id);
    let selected = primary || also;
    let name = display_name(&object, &editor.prefabs.read());
    let id = object.id;
    let indent = depth * 14;
    let is_camera = has_component(
        &object,
        &["Camera2d", "Camera3d", "GameCamera2D", "GameCamera3D"],
    );
    let camera_active = is_camera
        && !object.components.iter().any(|c| {
            base(&c.ty) == "Camera"
                && matches!(&c.value, Value::Struct { fields, .. }
                    if fields.iter().any(|(n, f)| n == "is_active" && *f == Field::Bool(false)))
        });
    let is_light = has_component(&object, &["DirectionalLight", "PointLight", "SpotLight"]);
    let hidden = object
        .components
        .iter()
        .any(|c| matches!(&c.value, Value::Unit(p) if p == "Visibility::Hidden"));
    let drawn = is_light
        || has_component(
            &object,
            &[
                "MeshDef",
                "SpriteDef",
                "Canvas",
                "Node",
                "Text",
                "Button",
                "Toggle",
                "Slider",
                "Dropdown",
            ],
        );
    let class = match (primary, also, object.prefab.is_some()) {
        (true, _, _) => "row sel",
        (_, true, _) => "row sel also",
        (_, _, true) => "row prefab",
        _ => "row",
    };
    let _ = selected;
    let has_children = !object.children.is_empty();
    let folded = editor.collapsed.read().contains(&id);
    rsx! {
        div {
            class,
            style: "padding-left: {indent}px;",
            onclick: move |event| {
                // Ctrl toggles, Shift selects a range (the stage reads the keys on every platform).
                let m = event.modifiers();
                let held = stage.peek().held;
                let ctrl = m.ctrl() || m.meta() || held.ctrl;
                let shift = m.shift() || held.shift;
                if ctrl {
                    super::edit::toggle_selection(editor, id);
                } else if shift {
                    super::edit::select_range(editor, id);
                } else {
                    super::edit::select_only(editor, Some(id));
                }
            },
            // Pressing a row starts moving it; letting go over another row makes it a child.
            onmousedown: move |_| {
                let mut dragging = editor.dragging;
                dragging.set(Some(id));
            },
            onmouseup: move |_| {
                let source = *editor.dragging.peek();
                let mut dragging = editor.dragging;
                dragging.set(None);
                if let Some(source) = source.filter(|s| *s != id) {
                    super::edit::reparent(editor, source, Some(id));
                }
            },
            span {
                class: "icon",
                onmousedown: move |event| event.stop_propagation(),
                onclick: move |event| {
                    event.stop_propagation();
                    if has_children {
                        let mut collapsed = editor.collapsed;
                        let mut set = collapsed.peek().clone();
                        if !set.remove(&id) {
                            set.insert(id);
                        }
                        collapsed.set(set);
                    }
                },
                if !has_children { "" } else if folded { "+" } else { "-" }
            }
            span { class: "grow", "{name}" }
            // A camera: the rectangle is lit while it is the active one in the game.
            if is_camera {
                span {
                    class: if camera_active { "tog cam on" } else { "tog cam" },
                    title: "Active camera",
                    onclick: move |event| {
                        event.stop_propagation();
                        super::edit::set_camera_active(editor, id, !camera_active);
                    },
                }
            }
            // A light: the bulb is lit while the light is on (hidden lights do not light).
            if is_light {
                span {
                    class: if hidden { "tog bulb" } else { "tog bulb on" },
                    title: "Light on / off",
                    onclick: move |event| {
                        event.stop_propagation();
                        super::edit::set_hidden(editor, id, !hidden);
                    },
                }
            }
            // Anything drawn: shown or hidden.
            if drawn && !is_light {
                span {
                    class: if hidden { "tog eye" } else { "tog eye on" },
                    title: "Visible",
                    onclick: move |event| {
                        event.stop_propagation();
                        super::edit::set_hidden(editor, id, !hidden);
                    },
                }
            }
        }
        if !folded {
            for child in object.children {
                Node { key: "{child.id}", object: child, depth: depth + 1 }
            }
        }
    }
}
