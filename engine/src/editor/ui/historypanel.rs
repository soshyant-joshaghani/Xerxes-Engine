//! The History editor: every edit of the open scene, newest first. Click a step to go to the
//! scene as it was right after it; "Original" is the scene as opened.

use dioxus::prelude::*;

use super::{Editor, edit};

#[component]
pub fn HistoryPanel() -> Element {
    let editor = use_context::<Editor>();
    let history = editor.history.read().clone();
    let (labels, applied) = history.labels();
    let labels: Vec<String> = labels.into_iter().map(str::to_string).collect();
    rsx! {
        div { class: "body",
            if labels.is_empty() {
                div { class: "hint", "No edits yet. Edits to the scene are listed here; click one to go back to it." }
            }
            for (index, label) in labels.iter().enumerate().rev() {
                div {
                    key: "{index}",
                    class: if applied == index + 1 { "row sel" } else if applied > index { "row" } else { "row muted" },
                    onclick: {
                        let steps = index + 1;
                        move |_| edit::jump(editor, steps)
                    },
                    "{label}"
                }
            }
            if !labels.is_empty() {
                div {
                    class: if applied == 0 { "row sel" } else { "row" },
                    onclick: move |_| edit::jump(editor, 0),
                    "Original"
                }
            }
        }
    }
}
