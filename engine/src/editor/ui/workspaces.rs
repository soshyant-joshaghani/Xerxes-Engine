//! The saved layouts in the top bar: Blender's workspace tabs. Click a tab to switch to that
//! layout, click the showing one (or right click any) for its menu: rename, duplicate, reset,
//! delete. Drag a tab to reorder. `+` adds a layout (a copy of this one or a shipped one).
//!
//! Tabs have a fixed width so a menu (and a drag) can be placed against a tab without
//! measuring anything (Blitz cannot measure). Menus are drawn at the root, over every area.

use dioxus::html::input_data::MouseButton;
use dioxus::html::{InteractionLocation, PointerInteraction};
use dioxus::prelude::*;

use super::{Editor, Menu};
use crate::editor::layout::{self, PRESET_NAMES};
use crate::editor::theme::{TAB_GAP, TAB_WIDTH, tabs_left, toolbar};

/// A press on a tab: a click, or (once the pointer moves) a drag that reorders the tabs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabDrag {
    /// The tab being dragged (follows it as it moves).
    pub from: usize,
    /// Where the press started (client x).
    pub start: f32,
    pub moved: bool,
}

/// Moving less than this is a click.
const SLOP: f32 = 4.0;

/// The tabs and the `+` button, inside the top bar.
#[component]
pub fn WorkspaceTabs() -> Element {
    let editor = use_context::<Editor>();
    let layouts = editor.layouts.read().clone();
    let renaming = *editor.renaming.read();
    rsx! {
        div { class: "wtabs",
            for (index, workspace) in layouts.workspaces.iter().enumerate() {
                WorkspaceTab {
                    key: "{index}",
                    index,
                    name: workspace.name.clone(),
                    active: index == layouts.active,
                    renaming: renaming == Some(index),
                }
            }
            button {
                class: "dim",
                onclick: move |_| {
                    end_rename(editor);
                    let mut menu = editor.menu;
                    let open = *menu.peek() == Some(Menu::NewWorkspace);
                    menu.set(if open { None } else { Some(Menu::NewWorkspace) });
                },
                "+"
            }
        }
    }
}

#[component]
fn WorkspaceTab(index: usize, name: String, active: bool, renaming: bool) -> Element {
    let editor = use_context::<Editor>();
    if renaming {
        return rsx! {
            input {
                class: "num wtab-input",
                value: "{name}",
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
                    let mut layouts = editor.layouts;
                    layouts.write().rename(index, &e.value());
                },
                onkeydown: move |e| {
                    if matches!(e.key(), Key::Enter | Key::Escape) {
                        end_rename(editor);
                    }
                },
            }
        };
    }
    rsx! {
        div {
            class: if active { "wtab on" } else { "wtab" },
            onmousedown: move |e| {
                end_rename(editor);
                let (mut menu, mut drag) = (editor.menu, editor.tab_drag);
                if e.trigger_button() == Some(MouseButton::Secondary) {
                    menu.set(Some(Menu::Workspace(index)));
                } else {
                    menu.set(None);
                    drag.set(Some(TabDrag {
                        from: index,
                        start: e.client_coordinates().x as f32,
                        moved: false,
                    }));
                }
            },
            "{name}"
        }
    }
}

/// Ends a rename: an empty name becomes `Layout`.
pub fn end_rename(editor: Editor) {
    let (mut layouts, mut renaming) = (editor.layouts, editor.renaming);
    let Some(index) = *renaming.peek() else {
        return;
    };
    if layouts
        .peek()
        .workspaces
        .get(index)
        .is_some_and(|w| w.name.trim().is_empty())
    {
        layouts.write().rename(index, "Layout");
    }
    renaming.set(None);
}

/// Covers the window while a tab is pressed: moving reorders the tabs, letting go without
/// moving is a click (switch to the tab, or open the menu of the one that is showing, which
/// is how touch reaches it).
#[component]
pub fn TabDragLayer(drag: TabDrag, x: f32) -> Element {
    let editor = use_context::<Editor>();
    let on_move = move |e: Event<MouseData>| {
        let at = e.client_coordinates().x as f32;
        if !drag.moved && (at - drag.start).abs() < SLOP {
            return;
        }
        let mut layouts = editor.layouts;
        let count = layouts.peek().workspaces.len();
        let slot = ((at - x - tabs_left()) / (TAB_WIDTH + TAB_GAP)).floor();
        let to = (slot.max(0.0) as usize).min(count - 1);
        if to != drag.from {
            layouts.write().move_tab(drag.from, to);
        }
        let mut tab_drag = editor.tab_drag;
        tab_drag.set(Some(TabDrag {
            from: to,
            moved: true,
            ..drag
        }));
    };
    let on_up = move |_: Event<MouseData>| {
        let (mut tab_drag, mut menu, mut layouts) = (editor.tab_drag, editor.menu, editor.layouts);
        tab_drag.set(None);
        if drag.moved {
            return;
        }
        let index = drag.from;
        if layouts.peek().active == index {
            // Clicking the layout that is showing opens its menu.
            let open = *menu.peek() == Some(Menu::Workspace(index));
            menu.set(if open {
                None
            } else {
                Some(Menu::Workspace(index))
            });
        } else {
            layouts.write().select(index);
        }
    };
    rsx! {
        div {
            class: "capture",
            "data-interactive": "true",
            onmousemove: on_move,
            onmouseup: on_up,
        }
    }
}

/// Ends a rename when something else is clicked: covers the areas, not the bar.
#[component]
pub fn RenameBackdrop() -> Element {
    let editor = use_context::<Editor>();
    rsx! {
        div {
            class: "backdrop",
            style: "top: {toolbar()}px;",
            "data-interactive": "true",
            onmousedown: move |_| end_rename(editor),
        }
    }
}

/// The menu of a tab, or of the `+` button: under it, over the areas.
#[component]
pub fn WorkspaceMenu(menu: Menu) -> Element {
    let editor = use_context::<Editor>();
    let layouts = editor.layouts.read().clone();
    let (index, left) = match menu {
        Menu::Workspace(i) => (Some(i), tabs_left() + i as f32 * (TAB_WIDTH + TAB_GAP)),
        _ => (
            None,
            tabs_left() + layouts.workspaces.len() as f32 * (TAB_WIDTH + TAB_GAP),
        ),
    };
    let top = toolbar() - 4.0;
    let close = move || {
        let mut menu = editor.menu;
        menu.set(None);
    };
    rsx! {
        div { class: "backdrop", "data-interactive": "true", onclick: move |_| close() }
        div { class: "menu", style: "left: {left}px; top: {top}px;", "data-interactive": "true",
            if let Some(index) = index {
                div {
                    class: "row",
                    onclick: move |_| {
                        let mut renaming = editor.renaming;
                        renaming.set(Some(index));
                        close();
                    },
                    "Rename"
                }
                div {
                    class: "row",
                    onclick: move |_| {
                        let mut layouts = editor.layouts;
                        layouts.write().duplicate(index);
                        close();
                    },
                    "Duplicate"
                }
                div {
                    class: "row",
                    onclick: move |_| {
                        let mut layouts = editor.layouts;
                        layouts.write().reset_workspace(index);
                        close();
                    },
                    "Reset Layout"
                }
                div {
                    class: if layouts.workspaces.len() > 1 { "row" } else { "row off" },
                    onclick: move |_| {
                        let mut layouts = editor.layouts;
                        let result = layouts.write().remove(index);
                        if let Err(err) = result {
                            editor.error(format!("layout: {err}"));
                        }
                        close();
                    },
                    "Delete"
                }
            } else {
                div {
                    class: "row",
                    onclick: move |_| {
                        let mut layouts = editor.layouts;
                        let active = layouts.peek().active;
                        layouts.write().duplicate(active);
                        close();
                    },
                    "Duplicate Current"
                }
                div { class: "rule" }
                for name in PRESET_NAMES {
                    div {
                        key: "{name}",
                        class: "row",
                        onclick: move |_| {
                            let mut layouts = editor.layouts;
                            if let Some(workspace) = layout::preset(name) {
                                layouts.write().add(workspace);
                            }
                            close();
                        },
                        "{name}"
                    }
                }
                div { class: "rule" }
                div {
                    class: "row",
                    onclick: move |_| {
                        let mut layouts = editor.layouts;
                        layouts.set(layout::Layouts::default());
                        close();
                    },
                    "Reset All Layouts"
                }
            }
        }
    }
}
