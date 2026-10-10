//! Everything the editor can be told to do, by name: the one list the menus, the keys and the
//! command palette share. A command says what it is called, whether it can run now, which key
//! runs it, and how to run it.

use dioxus::prelude::*;

use super::{Editor, Menu, edit, save};
use crate::editor::keymap;
use crate::editor::layout::{AreaId, EditorKind};
use crate::editor::protocol::{Action, EditorCommand, ViewAction};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Undo,
    Redo,
    Save,
    AddEmpty,
    Duplicate,
    Delete,
    SelectNone,
    /// A View command for the camera of a Viewport area.
    View(AreaId, ViewAction),
    /// Maximize an area, or bring the layout back (`None`: the Viewport).
    ToggleMaximize(Option<AreaId>),
    SwitchLayout(usize),
    ResetLayout,
    Palette,
    /// Play the open project (or stop it), Build it, Publish it.
    PlayOrStop,
    Build,
    Publish,
}

/// The command a key's action stands for (`area` is the area under the pointer then).
pub fn from_action(action: Action, area: Option<AreaId>) -> Command {
    match action {
        Action::Undo => Command::Undo,
        Action::Redo => Command::Redo,
        Action::Save => Command::Save,
        Action::Duplicate => Command::Duplicate,
        Action::Delete => Command::Delete,
        Action::Palette => Command::Palette,
        Action::ToggleMaximize => Command::ToggleMaximize(area),
    }
}

/// The key that runs a command, as the keymap spells it.
pub fn shortcut(command: Command) -> Option<String> {
    let action = match command {
        Command::Undo => Action::Undo,
        Command::Redo => Action::Redo,
        Command::Save => Action::Save,
        Command::Duplicate => Action::Duplicate,
        Command::Delete => Action::Delete,
        Command::Palette => Action::Palette,
        Command::ToggleMaximize(_) => Action::ToggleMaximize,
        Command::View(_, ViewAction::Frame) => return Some("F".into()),
        _ => return None,
    };
    keymap::label(action)
}

pub fn label(editor: Editor, command: Command) -> String {
    match command {
        Command::Undo => match editor.history.read().undo_label() {
            Some(step) => format!("Undo {step}"),
            None => "Undo".into(),
        },
        Command::Redo => match editor.history.read().redo_label() {
            Some(step) => format!("Redo {step}"),
            None => "Redo".into(),
        },
        Command::Save => "Save".into(),
        Command::AddEmpty => "Add Empty".into(),
        Command::Duplicate => "Duplicate".into(),
        Command::Delete => "Delete".into(),
        Command::SelectNone => "Select None".into(),
        Command::View(_, ViewAction::Frame) => "Frame Selected".into(),
        Command::View(_, ViewAction::Reset) => "Reset View".into(),
        Command::View(_, ViewAction::Top) => "View Top".into(),
        Command::View(_, ViewAction::Front) => "View Front".into(),
        Command::View(_, ViewAction::Right) => "View Right".into(),
        Command::ToggleMaximize(_) => "Maximize Area".into(),
        Command::SwitchLayout(index) => {
            let layouts = editor.layouts.read();
            let name = layouts
                .workspaces
                .get(index)
                .map(|w| w.name.as_str())
                .unwrap_or("?");
            format!("Layout: {name}")
        }
        Command::ResetLayout => "Reset Layout".into(),
        Command::Palette => "Command Palette".into(),
        Command::PlayOrStop if super::gamepreview::running(editor) => "Stop".into(),
        Command::PlayOrStop => "Play".into(),
        Command::Build => "Build".into(),
        Command::Publish => "Publish".into(),
    }
}

/// Whether the command can run now.
pub fn enabled(editor: Editor, command: Command) -> bool {
    let selected = editor.selected.read().is_some();
    match command {
        Command::Undo => edit::can_undo(editor),
        Command::Redo => edit::can_redo(editor),
        Command::Save => *editor.dirty.read(),
        Command::AddEmpty => editor.scene.read().is_some(),
        Command::Duplicate
        | Command::Delete
        | Command::SelectNone
        | Command::View(_, ViewAction::Frame) => selected,
        Command::PlayOrStop => {
            super::gamepreview::running(editor) || editor.project.read().is_some()
        }
        Command::Build | Command::Publish => {
            editor.project.read().is_some() && !super::gamepreview::running(editor)
        }
        _ => true,
    }
}

pub fn run(editor: Editor, port: &super::Port, command: Command) {
    let selected = *editor.selected.peek();
    match command {
        Command::Undo => edit::undo(editor),
        Command::Redo => edit::redo(editor),
        Command::Save => save(editor),
        Command::AddEmpty => edit::add_empty(editor, selected),
        Command::Duplicate => {
            if let Some(id) = selected {
                edit::duplicate(editor, id);
            }
        }
        Command::Delete => {
            if let Some(id) = selected {
                edit::delete(editor, id);
            }
        }
        Command::SelectNone => {
            let mut selected = editor.selected;
            selected.set(None);
        }
        Command::View(area, action) => port.send(EditorCommand::ViewAction(area, action)),
        Command::ToggleMaximize(area) => {
            let workspace = editor.workspace();
            let target = area
                .or(workspace.maximized)
                .or_else(|| {
                    workspace
                        .areas()
                        .iter()
                        .find(|a| a.editor == EditorKind::Viewport)
                        .map(|a| a.id)
                })
                .or_else(|| workspace.areas().first().map(|a| a.id));
            if let Some(id) = target {
                editor.edit_layout(|w| w.toggle_maximize(id));
            }
        }
        Command::SwitchLayout(index) => {
            let mut layouts = editor.layouts;
            layouts.write().select(index);
        }
        Command::ResetLayout => {
            let mut layouts = editor.layouts;
            let active = layouts.peek().active;
            layouts.write().reset_workspace(active);
        }
        Command::Palette => {
            let mut palette = editor.palette;
            palette.set(Some(String::new()));
        }
        Command::PlayOrStop if super::gamepreview::running(editor) => {
            super::gamepreview::stop(editor)
        }
        Command::PlayOrStop => super::gamepreview::run(editor, "dev"),
        Command::Build => super::gamepreview::run(editor, "build"),
        Command::Publish => super::gamepreview::run(editor, "publish"),
    }
}

/// A command with what the user sees of it.
pub struct Entry {
    pub command: Command,
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
}

/// Every command the palette offers.
pub fn entries(editor: Editor) -> Vec<Entry> {
    let workspace = editor.workspace();
    let mut commands = vec![
        Command::Undo,
        Command::Redo,
        Command::Save,
        Command::AddEmpty,
        Command::Duplicate,
        Command::Delete,
        Command::SelectNone,
    ];
    if let Some(viewport) = workspace
        .areas()
        .iter()
        .find(|a| a.editor == EditorKind::Viewport)
    {
        for action in [
            ViewAction::Frame,
            ViewAction::Reset,
            ViewAction::Top,
            ViewAction::Front,
            ViewAction::Right,
        ] {
            commands.push(Command::View(viewport.id, action));
        }
    }
    commands.extend([Command::PlayOrStop, Command::Build, Command::Publish]);
    commands.push(Command::ToggleMaximize(None));
    commands.push(Command::ResetLayout);
    let count = editor.layouts.read().workspaces.len();
    commands.extend((0..count).map(Command::SwitchLayout));
    commands
        .into_iter()
        .map(|command| Entry {
            command,
            label: label(editor, command),
            shortcut: shortcut(command),
            enabled: enabled(editor, command),
        })
        .collect()
}

/// A menu line for a command: its name, its key, greyed when it cannot run.
#[component]
pub fn CommandItem(command: Command) -> Element {
    let editor = use_context::<Editor>();
    let port = use_context::<super::Port>();
    let text = label(editor, command);
    let on = enabled(editor, command);
    let key = shortcut(command);
    rsx! {
        div {
            class: if on { "row" } else { "row off" },
            onclick: move |_| {
                if on {
                    let mut menu = editor.menu;
                    menu.set(None::<Menu>);
                    run(editor, &port, command);
                }
            },
            span { class: "grow", "{text}" }
            if let Some(key) = key {
                span { class: "muted", "{key}" }
            }
        }
    }
}
