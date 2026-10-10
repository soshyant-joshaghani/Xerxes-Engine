//! The top bar: the app menus (File, Edit, Window, Help), the layout tabs, then the project
//! picker, open scene, Save, and the pipeline buttons (Play / Build / Publish come with the
//! jobs step). The transform tools live in the Viewport's header.

use dioxus::prelude::*;

use super::Editor;

#[component]
pub fn Toolbar(fps: u32) -> Element {
    let editor = use_context::<Editor>();
    let project = editor
        .project
        .read()
        .as_ref()
        .map(|p| p.title.clone())
        .unwrap_or_else(|| "No project".into());
    let scene = editor.scene_path.read().clone().unwrap_or_default();
    let dirty = *editor.dirty.read();

    rsx! {
        div { class: "bar", "data-interactive": "true",
            super::appmenu::AppMenus {}
            super::workspaces::WorkspaceTabs {}
            div { class: "grow" }
            button {
                onclick: move |_| {
                    let (mut screen, mut menu) = (editor.screen, editor.menu);
                    menu.set(None);
                    screen.set(super::Screen::Manager);
                },
                "Projects"
            }
            button {
                onclick: move |_| {
                    let mut menu = editor.menu;
                    let open = *menu.peek() == Some(super::Menu::Projects);
                    menu.set(if open { None } else { Some(super::Menu::Projects) });
                },
                "{project}"
            }
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
