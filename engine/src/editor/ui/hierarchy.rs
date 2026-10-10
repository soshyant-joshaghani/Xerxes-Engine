//! The Hierarchy: the open scene's objects as a tree. Click to select (synced with the
//! Scene view and the Inspector). Prefab instances show in blue.

use dioxus::prelude::*;

use super::Editor;
use crate::editor::project::codec::{Dimension, ObjectDoc};

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
        div { class: "body",
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
    let selected = *editor.selected.read() == Some(object.id);
    let name = display_name(&object, &editor.prefabs.read());
    let id = object.id;
    let indent = depth * 14;
    let class = match (selected, object.prefab.is_some()) {
        (true, _) => "row sel",
        (false, true) => "row prefab",
        _ => "row",
    };
    rsx! {
        div {
            class,
            style: "padding-left: {indent}px;",
            onclick: move |_| {
                let mut selected = editor.selected;
                selected.set(Some(id));
            },
            span { class: "icon", if object.children.is_empty() { "" } else { "-" } }
            "{name}"
        }
        for child in object.children {
            Node { key: "{child.id}", object: child, depth: depth + 1 }
        }
    }
}
