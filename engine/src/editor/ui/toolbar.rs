//! The top bar: the app menus (File, Edit, Window, Help), the layout tabs, then the open scene,
//! Save, and the pipeline buttons (Play / Build / Publish). Projects are opened from the File
//! menu; the transform tools live in the Viewport.

use dioxus::prelude::*;

use super::Editor;

#[component]
pub fn Toolbar(fps: u32) -> Element {
    let editor = use_context::<Editor>();
    let scene = editor.scene_path.read().clone().unwrap_or_default();
    let dirty = *editor.dirty.read();

    rsx! {
        div { class: "bar", "data-interactive": "true",
            super::appmenu::AppMenus {}
            super::workspaces::WorkspaceTabs {}
            div { class: "grow" }
            span { class: "muted", "{scene}" }
            button {
                disabled: !dirty,
                onclick: move |_| super::save(editor),
                if dirty { "Save*" } else { "Save" }
            }
            if super::gamepreview::running(editor) {
                button { onclick: move |_| super::gamepreview::stop(editor), "Stop" }
            } else {
                button {
                    class: "play",
                    disabled: editor.project.read().is_none(),
                    onclick: move |_| super::gamepreview::run(editor, "dev"),
                    "Play"
                }
            }
            button {
                disabled: editor.project.read().is_none() || super::gamepreview::running(editor),
                onclick: move |_| super::gamepreview::run(editor, "build"),
                "Build"
            }
            button {
                disabled: editor.project.read().is_none() || super::gamepreview::running(editor),
                onclick: move |_| super::gamepreview::run(editor, "publish"),
                "Publish"
            }
            div { class: "sep" }
            span { class: "muted", "{fps} fps" }
        }
    }
}
