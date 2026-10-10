//! The menus in an area's header, after the editor-type button: the Viewport has View, Select,
//! Add and Object, like Blender's 3D Viewport. A menu is data (a list of entries) made of the
//! shared commands (`commands`); an entry with no command is planned and shown as "soon".
//!
//! The editor-type button and the menu buttons have fixed widths, so a menu is placed under
//! its button without measuring anything.

use dioxus::prelude::*;

use super::commands::{Command, CommandItem};
use super::{Editor, Menu};
use crate::editor::layout::{AreaId, EditorKind, Rect, header};
use crate::editor::protocol::ViewAction;
use crate::editor::theme::{ETYPE_WIDTH, HMENU_GAP, HMENU_WIDTH};

enum Entry {
    /// A command.
    Run(Command),
    /// A line that is not built yet.
    Soon(&'static str),
    Rule,
}

use Entry::{Rule, Run, Soon};

/// The menus of an editor's header, left to right.
pub fn menus(kind: EditorKind) -> &'static [&'static str] {
    match kind {
        EditorKind::Viewport => &["View", "Select", "Add", "Object"],
        _ => &[],
    }
}

fn entries(kind: EditorKind, menu: usize, area: AreaId) -> Vec<Entry> {
    let view = |action| Run(Command::View(area, action));
    match (kind, menu) {
        (EditorKind::Viewport, 0) => vec![
            view(ViewAction::Frame),
            view(ViewAction::Reset),
            Rule,
            view(ViewAction::Top),
            view(ViewAction::Front),
            view(ViewAction::Right),
            Rule,
            Run(Command::ToggleMaximize(Some(area))),
            Rule,
            Soon("Wireframe"),
            Soon("Orthographic"),
        ],
        (EditorKind::Viewport, 1) => vec![Run(Command::SelectNone), Soon("All")],
        (EditorKind::Viewport, 2) => vec![
            Run(Command::AddEmpty),
            Rule,
            Soon("Mesh"),
            Soon("Light"),
            Soon("Camera"),
        ],
        (EditorKind::Viewport, 3) => vec![Run(Command::Duplicate), Run(Command::Delete)],
        _ => Vec::new(),
    }
}

/// The buttons, in the header.
#[component]
pub fn HeaderMenus(area: AreaId, kind: EditorKind) -> Element {
    let editor = use_context::<Editor>();
    let open = match *editor.menu.read() {
        Some(Menu::Header(id, index)) if id == area => Some(index),
        _ => None,
    };
    rsx! {
        div { class: "hmenus",
            for (index, title) in menus(kind).iter().enumerate() {
                button {
                    key: "{title}",
                    class: if open == Some(index) { "on" } else { "" },
                    onclick: move |_| {
                        let mut menu = editor.menu;
                        menu.set(if open == Some(index) { None } else { Some(Menu::Header(area, index)) });
                    },
                    "{title}"
                }
            }
        }
    }
}

/// The open menu, under its button.
#[component]
pub fn HeaderMenuView(area: AreaId, kind: EditorKind, index: usize, anchor: Rect) -> Element {
    let editor = use_context::<Editor>();
    // The area's border, the header's padding, the editor-type button, the header's gap.
    let left = anchor.x + 1.0 + 4.0 + ETYPE_WIDTH + 4.0 + index as f32 * (HMENU_WIDTH + HMENU_GAP);
    let top = anchor.y + header() - 2.0;
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
            for (line, entry) in entries(kind, index, area).into_iter().enumerate() {
                match entry {
                    Rule => rsx! { div { key: "{line}", class: "rule" } },
                    Soon(label) => rsx! {
                        div { key: "{line}", class: "row off",
                            span { class: "grow", "{label}" }
                            span { class: "muted", "soon" }
                        }
                    },
                    Run(command) => rsx! { CommandItem { key: "{line}", command } },
                }
            }
        }
    }
}
