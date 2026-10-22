//! Help, About Xerxes Engine: a panel in the middle of the window.

use dioxus::prelude::*;

use super::Editor;

const WIDTH: f32 = 440.0;
const HEIGHT: f32 = 300.0;

#[component]
pub fn About(width: f32, height: f32) -> Element {
    let editor = use_context::<Editor>();
    let left = ((width - WIDTH) / 2.0).max(8.0);
    let top = ((height - HEIGHT) / 2.0).max(8.0);
    let close = move || {
        let mut about = editor.about;
        about.set(false);
    };
    let project = editor
        .project
        .read()
        .as_ref()
        .map(|p| p.title.clone())
        .unwrap_or_else(|| "none".into());
    rsx! {
        div { class: "backdrop", "data-interactive": "true", onclick: move |_| close() }
        div {
            class: "about",
            style: "left: {left}px; top: {top}px; width: {WIDTH}px; height: {HEIGHT}px;",
            "data-interactive": "true",
            div { class: "about-name", "Xerxes Engine" }
            div { class: "muted", "Version {env!(\"CARGO_PKG_VERSION\")}" }
            div { class: "about-text",
                "A game engine and editor on Bevy, with a Rust multiplayer server and an API that grow with it, so a game does not have to build its backend. Games are Bevy; the editor is Dioxus over a Bevy viewport, from one source on the web, Windows and Android."
            }
            div { class: "about-rows",
                div { class: "about-row", span { class: "muted", "Runtime" } span { "Bevy 0.18" } }
                div { class: "about-row", span { class: "muted", "Editor UI" } span { "Dioxus 0.7" } }
                div { class: "about-row", span { class: "muted", "Physics" } span { "Rapier 0.36 (xerxes_sim)" } }
                div { class: "about-row", span { class: "muted", "Open project" } span { "{project}" } }
            }
            div { class: "grow" }
            div { class: "about-buttons",
                button { onclick: move |_| close(), "Close" }
            }
        }
    }
}
