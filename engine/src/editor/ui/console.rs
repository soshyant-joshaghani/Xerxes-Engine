//! The Console: the editor's messages (and, with the jobs step, build output).

use dioxus::prelude::*;

use super::Editor;

#[component]
pub fn Console() -> Element {
    let editor = use_context::<Editor>();
    let lines = editor.log.read().clone();
    let errors = lines.iter().filter(|(error, _)| *error).count();
    let (total, none) = (lines.len(), lines.is_empty());
    rsx! {
        div { class: "actions",
            button {
                disabled: none,
                onclick: move |_| {
                    let mut log = editor.log;
                    log.set(Vec::new());
                },
                "Clear"
            }
            span { class: "muted", "{total} messages, {errors} errors" }
        }
        div { class: "body",
            if lines.is_empty() {
                div { class: "hint", "No messages." }
            }
            for (i, (error, line)) in lines.into_iter().enumerate().rev() {
                div { key: "{i}", class: if error { "console-line err" } else { "console-line" }, "{line}" }
            }
        }
    }
}
