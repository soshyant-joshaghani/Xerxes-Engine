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
    /// The objects selected besides the primary one (multi-select).
    SelectAlso(Vec<u32>),
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
    /// What the Scene view shows besides the scene (grid, icons, gizmos, projection).
    SetViewOptions(ViewOptions),
}

/// Viewport display options (the header's Gizmos and Display menus, and the nav column).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewOptions {
    /// The ground grid and axes.
    pub grid: bool,
    /// The transform gizmo (move / rotate / scale handles) and the rect tool's handles.
    pub transform_gizmo: bool,
    /// The selection's outline.
    pub selection: bool,
    /// Camera frustums and light markers.
    pub icons: bool,
    /// The rectangle drawn around each canvas.
    pub canvases: bool,
    /// The axes widget and camera buttons on the right of the viewport.
    pub nav_gizmo: bool,
    /// The tool column on the left of the viewport.
    pub tool_shelf: bool,
    /// A 3D scene seen without perspective.
    pub orthographic: bool,
    /// How far one step of the mouse wheel zooms (1.0 was the first version's; 0.5 is the
    /// default). Set in the View menu.
    pub zoom_speed: f32,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self {
            grid: true,
            transform_gizmo: true,
            selection: true,
            icons: true,
            canvases: true,
            nav_gizmo: true,
            tool_shelf: true,
            orthographic: false,
            zoom_speed: 0.5,
        }
    }
}

/// How the active Viewport camera is oriented (for the axes widget).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewBasis {
    /// The camera's right, up and forward axes in world space.
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub forward: [f32; 3],
    /// A 2D (flat) scene: only X and Y matter.
    pub flat: bool,
}

/// What a drag of a UI element does: resize it (the Rect tool, or Move for a plain move),
/// scale it or rotate it (its `UiTransform`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RectKind {
    #[default]
    Resize,
    Scale,
    Rotate,
}

/// A drag with the Rect tool, in the object's plane: which handle (`[0, 0]` the body, else the
/// side on each axis, -1 / 0 / 1; y up) and how far it moved since it started (world units).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectDrag {
    pub id: u32,
    pub handle: [i8; 2],
    pub delta: [f32; 2],
    /// The drag's number (one per press), so the UI can tell a new drag from the same one.
    pub seq: u64,
    /// Pixels per world unit of the canvas the object is on (1 off a canvas).
    pub pixels_per_unit: f32,
    /// The size the object had when the drag started, in the same units as `delta` times
    /// `pixels_per_unit` (canvas pixels for UI, world units for a sprite).
    pub size: [f32; 2],
    /// The drag is over: this is its final value (kept until the next drag starts).
    pub ended: bool,
    /// The object is a UI element (a `RectTransform` or a flexbox `Node`), not a sprite.
    pub ui: bool,
    pub kind: RectKind,
    /// For `Scale`: the factor on each axis of the element's scale (1 = unchanged).
    pub scale: [f32; 2],
    /// For `Rotate`: degrees to add to the element's rotation (clockwise on screen).
    pub rotation: f32,
    /// Where a UI element's top-left corner is, relative to its parent's, in canvas pixels
    /// (y down). Pins an out-of-flow element when it is dragged.
    pub offset: [f32; 2],
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
    Bottom,
    Back,
    Left,
    /// Closer / farther by a step.
    ZoomIn,
    ZoomOut,
}

/// The transform tool (W / E / R).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    /// Click to select; no handles.
    Select,
    #[default]
    Move,
    Rotate,
    Scale,
    /// Unity's Rect tool (T): move and resize a sprite or a UI element in its plane.
    Rect,
}

/// Stage → UI.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditorSnapshot {
    /// Frames per second (0 until the first second has passed).
    pub fps: u32,
    /// The last object clicked in the Scene view (`seq` changes on every click, so clicking
    /// the same object again still reaches the UI). `None` = clicked empty space.
    pub picked: Option<(Option<u32>, u64, bool)>,
    /// Ctrl / Shift are held (read by the stage, so a click in the UI knows on every platform).
    pub held: Held,
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
    /// The active Viewport camera's axes, for the navigation widget.
    pub view_basis: ViewBasis,
    /// A Rect-tool drag in progress (or just finished), and its sequence number.
    pub rect: Option<RectDrag>,
}

/// The modifier keys held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Held {
    /// Ctrl (Cmd on macOS).
    pub ctrl: bool,
    pub shift: bool,
}

/// The axes the move and rotate gizmos use (X toggles, like Unity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Space {
    #[default]
    Global,
    Local,
}
