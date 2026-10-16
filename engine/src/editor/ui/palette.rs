//! The command palette (F3, like Blender's search): type to filter every command, Enter runs
//! the first one that can run, a click runs the one clicked.

use dioxus::prelude::*;

use super::Editor;
use super::commands::{self, Command};
use crate::editor::theme::toolbar;

const WIDTH: f32 = 460.0;
const SHOWN: usize = 12;

#[component]
pub fn Palette(window_width: f32) -> Element {
    let editor = use_context::<Editor>();
    let port = use_context::<super::Port>();
    let query = editor.palette.read().clone().unwrap_or_default();
    let words: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect();
    let all = commands::entries(editor);
    let matches: Vec<_> = all
        .into_iter()
        .filter(|e| {
            let label = e.label.to_lowercase();
            words.iter().all(|w| label.contains(w))
        })
        .take(SHOWN)
        .collect();
    let first: Option<Command> = matches.iter().find(|e| e.enabled).map(|e| e.command);
    let left = ((window_width - WIDTH) / 2.0).max(8.0);
    let top = toolbar() + 8.0;
    let close = move || {
        let mut palette = editor.palette;
        palette.set(None);
    };
    let enter_port = port.clone();
    rsx! {
        div {
            class: "backdrop",
            "data-interactive": "true",
            onmousedown: move |_| close(),
        }
        div {
            class: "palette",
            "data-interactive": "true",
            style: "left: {left}px; top: {top}px; width: {WIDTH}px;",
            input {
                class: "num palette-input",
                value: "{query}",
                placeholder: "Type a command",
                onmounted: move |e: Event<MountedData>| async move {
                    // The web page focuses the field (autofocus only works at page load);
                    // natively a click focuses it (Blitz has no mounted events).
                    #[cfg(target_arch = "wasm32")]
                    {
                        let _ = e.set_focus(true).await;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = e;
                },
                oninput: move |e| {
                    let mut palette = editor.palette;
                    palette.set(Some(e.value()));
                },
                onkeydown: move |e| match e.key() {
                    Key::Escape => close(),
                    Key::Enter => {
                        if let Some(command) = first {
                            close();
                            commands::run(editor, &enter_port, command);
                        }
                    }
                    _ => {}
                },
            }
            if matches.is_empty() {
                div { class: "hint", "No command matches." }
            }
            for entry in matches {
                {
                    let port = port.clone();
                    let command = entry.command;
                    let on = entry.enabled;
                    rsx! {
                        div {
                            key: "{entry.label}",
                            class: if on { "row" } else { "row off" },
                            onclick: move |_| {
                                if on {
                                    close();
                                    commands::run(editor, &port, command);
                                }
                            },
                            span { class: "grow", "{entry.label}" }
                            if let Some(key) = &entry.shortcut {
                                span { class: "muted", "{key}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
