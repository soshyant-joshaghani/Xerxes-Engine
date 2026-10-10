//! What crosses the bridge between the editor UI (Dioxus) and the Scene view (Bevy).

use std::sync::Arc;

use super::view::SceneView;
use crate::editor::project::codec::TransformData;

/// UI → stage.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorCommand {
    /// Replace what the Scene view shows.
    ShowScene(SceneView),
    /// An image a material or sprite uses, by its authored path (bytes fetched by the UI).
    Texture {
        path: String,
        bytes: Arc<Vec<u8>>,
    },
    Select(Option<u32>),
    SetTool(Tool),
    /// Gizmo axes: the world's or the selected object's (scale always uses the object's).
    SetSpace(Space),
    /// Frame the selected object (like F).
    Frame,
    /// A window covers the Scene view: the stage ignores the pointer.
    Modal(bool),
    /// The workspace on screen: the stage places its Viewport camera from it (the UI places
    /// the areas from the same tree).
    SetLayout(Arc<crate::editor::layout::Workspace>),
    /// A command from a Viewport's header menu, for that Viewport's camera.
    ViewAction(crate::editor::layout::AreaId, ViewAction),
}

/// What a key can ask the editor to do (the keymap binds keys to these).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Undo,
    Redo,
    Save,
    Duplicate,
    Delete,
    /// Open the command palette.
    Palette,
    /// Maximize the area under the pointer, or bring it back.
    ToggleMaximize,
}

impl Action {
    pub const ALL: [Action; 7] = [
        Action::Undo,
        Action::Redo,
        Action::Save,
        Action::Duplicate,
        Action::Delete,
        Action::Palette,
        Action::ToggleMaximize,
    ];

    /// The name shown in the Preferences.
    pub fn title(self) -> &'static str {
        match self {
            Action::Undo => "Undo",
            Action::Redo => "Redo",
            Action::Save => "Save",
            Action::Duplicate => "Duplicate",
            Action::Delete => "Delete",
            Action::Palette => "Command Palette",
            Action::ToggleMaximize => "Maximize Area",
        }
    }

    /// The name used in the keymap file.
    pub fn id(self) -> &'static str {
        match self {
            Action::Undo => "undo",
            Action::Redo => "redo",
            Action::Save => "save",
            Action::Duplicate => "duplicate",
            Action::Delete => "delete",
            Action::Palette => "palette",
            Action::ToggleMaximize => "maximize",
        }
    }
}

/// What a Viewport's View menu does to its camera.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewAction {
    /// Put the selected object in the middle.
    Frame,
    /// Back to the starting view.
    Reset,
    /// Look straight down, from the front, from the right (3D scenes).
    Top,
    Front,
    Right,
}

/// The transform tool (W / E / R).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Move,
    Rotate,
    Scale,
}

/// Stage → UI.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditorSnapshot {
    /// Frames per second (0 until the first second has passed).
    pub fps: u32,
    /// The last object clicked in the Scene view (`seq` changes on every click, so clicking
    /// the same object again still reaches the UI). `None` = clicked empty space.
    pub picked: Option<(Option<u32>, u64)>,
    /// A gizmo drag in progress or just finished: the object and its new transform.
    pub dragged: Option<(u32, TransformData, u64)>,
    /// The last tool/space change made with the keyboard (W/E/R, X) and its sequence number:
    /// the UI applies each one once. (Not the current tool: a snapshot sent before the stage
    /// saw a toolbar click would otherwise switch the toolbar back.)
    pub keyed: Option<(Tool, Space, u64)>,
    /// The last action a key fired (see `editor::keymap`), its sequence number, and the area
    /// under the pointer then: the UI does each one once.
    pub action: Option<(Action, u64, Option<crate::editor::layout::AreaId>)>,
    /// The editor's frame for the current window (its size and safe area): the UI solves the
    /// workspace inside it, like the stage does.
    pub layout: crate::editor::theme::Layout,
}

/// The axes the move and rotate gizmos use (X toggles, like Unity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Space {
    #[default]
    Global,
    Local,
}
