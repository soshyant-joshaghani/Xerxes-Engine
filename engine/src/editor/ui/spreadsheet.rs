//! The Spreadsheet: every object of the open scene as a row (name, what it is, position,
//! rotation, scale, how many components), like Blender's. Click a row to select the object
//! (the Outliner and the Properties follow); the filter narrows the rows by name or kind.
//! It only looks: change values in the Properties.

use dioxus::prelude::*;

use super::Editor;
use crate::editor::project::table::{self, Row, number};

/// A cell with a number.
#[component]
fn Num(value: f32) -> Element {
    rsx! { span { class: "c c-num", "{number(value)}" } }
}

#[component]
fn SheetRow(row: Row, selected: bool) -> Element {
    let editor = use_context::<Editor>();
    let id = row.id;
    let indent = 6 + row.depth * 12;
    rsx! {
        div {
            class: if selected { "srow sel" } else { "srow" },
            onclick: move |_| {
                let mut chosen = editor.selected;
                chosen.set(Some(id));
            },
            span { class: "c c-id", "{row.id}" }
            span { class: "c c-name", style: "padding-left: {indent}px;", "{row.name}" }
            span { class: "c c-kind", "{row.kind}" }
            Num { value: row.position[0] }
            Num { value: row.position[1] }
            Num { value: row.position[2] }
            Num { value: row.rotation[0] }
            Num { value: row.rotation[1] }
            Num { value: row.rotation[2] }
            Num { value: row.scale[0] }
            Num { value: row.scale[1] }
            Num { value: row.scale[2] }
            span { class: "c c-count", "{row.components}" }
        }
    }
}

#[component]
pub fn SpreadsheetPanel() -> Element {
    let editor = use_context::<Editor>();
    let mut filter = use_signal(String::new);
    let scene = editor.scene.read().clone();
    let Some(scene) = scene else {
        return rsx! {
            div { class: "hint", "Open a scene to see its objects here." }
        };
    };
    let prefabs = editor.prefabs.read().clone();
    let all = table::rows(&scene, &prefabs);
    let total = all.len();
    let shown = table::filter(all, &filter.read());
    let selected = *editor.selected.read();
    rsx! {
        div { class: "actions",
            input {
                class: "num sheet-filter",
                value: "{filter}",
                placeholder: "Filter by name or kind",
                oninput: move |e| filter.set(e.value()),
            }
            span { class: "muted", "{shown.len()} of {total} objects" }
        }
        div { class: "sheet",
            div { class: "srow head",
                span { class: "c c-id", "Id" }
                span { class: "c c-name", "Name" }
                span { class: "c c-kind", "Kind" }
                for label in ["Pos X", "Pos Y", "Pos Z", "Rot X", "Rot Y", "Rot Z", "Scl X", "Scl Y", "Scl Z"] {
                    span { key: "{label}", class: "c c-num", "{label}" }
                }
                span { class: "c c-count", "Comp" }
            }
            if shown.is_empty() {
                div { class: "hint", "No object matches." }
            }
            for row in shown {
                SheetRow { key: "{row.id}", selected: selected == Some(row.id), row }
            }
        }
    }
}
