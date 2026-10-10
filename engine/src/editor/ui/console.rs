//! The Console: the editor's messages (and, with the jobs step, build output).

use dioxus::prelude::*;

use super::Editor;

#[component]
pub fn Console() -> Element {
    let editor = use_context::<Editor>();
    let lines = editor.log.read().clone();
    rsx! {
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
