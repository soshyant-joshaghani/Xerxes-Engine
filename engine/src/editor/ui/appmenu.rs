//! The app menus at the left of the top bar: File, Edit, Window, Help. Like Blender's, they
//! hold what applies to the whole editor; what applies to one area is in that area's header.
//!
//! The buttons have a fixed width so a menu is placed under its button without measuring.

use dioxus::prelude::*;

use super::commands::{Command, CommandItem};
use super::{Editor, Menu, Screen};
use crate::editor::theme::{MENU_GAP, MENU_WIDTH, toolbar};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMenu {
    File,
    Edit,
    Window,
    Help,
}

impl AppMenu {
    const ALL: [AppMenu; 4] = [AppMenu::File, AppMenu::Edit, AppMenu::Window, AppMenu::Help];

    fn title(self) -> &'static str {
        match self {
            AppMenu::File => "File",
            AppMenu::Edit => "Edit",
            AppMenu::Window => "Window",
            AppMenu::Help => "Help",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|m| *m == self).unwrap_or(0)
    }
}

/// The four buttons, inside the top bar.
#[component]
pub fn AppMenus() -> Element {
    let editor = use_context::<Editor>();
    let open = match *editor.menu.read() {
        Some(Menu::App(menu)) => Some(menu),
        _ => None,
    };
    rsx! {
        div { class: "appmenus",
            for menu in AppMenu::ALL {
                button {
                    key: "{menu.title()}",
                    class: if open == Some(menu) { "on" } else { "" },
                    onclick: move |_| {
                        super::workspaces::end_rename(editor);
                        let mut state = editor.menu;
                        state.set(if open == Some(menu) { None } else { Some(Menu::App(menu)) });
                    },
                    "{menu.title()}"
                }
            }
        }
    }
}

/// One line of a menu. A disabled line is greyed and does nothing; `soon` marks what is
/// planned.
#[component]
fn Item(label: &'static str, enabled: bool, soon: bool, onpress: EventHandler<()>) -> Element {
    let editor = use_context::<Editor>();
    rsx! {
        div {
            class: if enabled { "row" } else { "row off" },
            onclick: move |_| {
                if enabled {
                    onpress.call(());
                    let mut menu = editor.menu;
                    menu.set(None);
                }
            },
            span { class: "grow", "{label}" }
            if soon {
                span { class: "muted", "soon" }
            }
        }
    }
}

/// The open menu, under its button.
#[component]
pub fn AppMenuView(menu: AppMenu) -> Element {
    let editor = use_context::<Editor>();
    let left = 8.0 + menu.index() as f32 * (MENU_WIDTH + MENU_GAP);
    let top = toolbar() - 4.0;
    let layouts = editor.layouts.read().clone();
    let active = layouts.active;
    rsx! {
        div {
            class: "backdrop",
            "data-interactive": "true",
            onclick: move |_| {
                let mut menu = editor.menu;
                menu.set(None);
            },
        }
        div { class: "menu", style: "left: {left}px; top: {top}px;", "data-interactive": "true",
            match menu {
                AppMenu::File => rsx! {
                    Item {
                        label: "Project Manager...",
                        enabled: true,
                        soon: false,
                        onpress: move |_| {
                            let mut screen = editor.screen;
                            screen.set(Screen::Manager);
                        },
                    }
                    div { class: "rule" }
                    CommandItem { command: Command::Save }
                    Item {
                        label: "Edit Scene as Text",
                        enabled: editor.scene_path.read().is_some(),
                        soon: false,
                        onpress: move |_| {
                            if let Some(path) = editor.scene_path.peek().clone() {
                                super::texteditor::open(editor, path);
                            }
                        },
                    }
                },
                AppMenu::Edit => rsx! {
                    CommandItem { command: Command::Undo }
                    CommandItem { command: Command::Redo }
                    div { class: "rule" }
                    CommandItem { command: Command::AddEmpty }
                    CommandItem { command: Command::Duplicate }
                    CommandItem { command: Command::Delete }
                    div { class: "rule" }
                    CommandItem { command: Command::Palette }
                },
                AppMenu::Window => rsx! {
                    Item {
                        label: "Rename Layout",
                        enabled: true,
                        soon: false,
                        onpress: move |_| {
                            let mut renaming = editor.renaming;
                            renaming.set(Some(active));
                        },
                    }
                    Item {
                        label: "Duplicate Layout",
                        enabled: true,
                        soon: false,
                        onpress: move |_| {
                            let mut layouts = editor.layouts;
                            layouts.write().duplicate(active);
                        },
                    }
                    Item {
                        label: "Reset Layout",
                        enabled: true,
                        soon: false,
                        onpress: move |_| {
                            let mut layouts = editor.layouts;
                            layouts.write().reset_workspace(active);
                        },
                    }
                    div { class: "rule" }
                    Item {
                        label: "Reset All Layouts",
                        enabled: true,
                        soon: false,
                        onpress: move |_| {
                            let mut layouts = editor.layouts;
                            layouts.set(crate::editor::layout::Layouts::default());
                        },
                    }
                },
                AppMenu::Help => rsx! {
                    Item {
                        label: "About Xerxes Engine",
                        enabled: true,
                        soon: false,
                        onpress: move |_| editor.info(format!("Xerxes Engine {}", env!("CARGO_PKG_VERSION"))),
                    }
                    Item {
                        label: "Keyboard Shortcuts",
                        enabled: true,
                        soon: false,
                        onpress: move |_| {
                            for line in SHORTCUTS {
                                editor.info(*line);
                            }
                        },
                    }
                },
            }
        }
    }
}

const SHORTCUTS: &[&str] = &[
    "Viewport: W / E / R move, rotate, scale; X global or local; F frame the selection",
    "Viewport camera: right mouse look and fly (W A S D Q E); middle mouse pan; wheel dolly",
    "Areas: drag an edge to resize; right click an edge for split, join and swap",
    "Layouts: click a tab to switch, drag to reorder, right click for its menu",
];
