//! The editor's screen layout, Blender style: a workspace is a tree of **areas**; every area
//! shows one **editor** (Viewport, Outliner, Properties...) chosen with the button at its
//! header's left. Areas are split, joined, swapped and resized; a screen holds several
//! workspaces (saved layouts). See `__docs__/editor-layout.md`.
//!
//! This module is plain data and arithmetic: no Bevy, no Dioxus. That is the point. The
//! Dioxus UI places the areas from [`solve`] and the Bevy stage places its viewport cameras
//! from the very same function, so both agree on every platform without anything measuring
//! the page (Blitz, the native renderer, cannot report element positions).
//!
//! - `geom`         [`Rect`] and the sizes every area shares
//! - `editor_kind`  [`EditorKind`]: the editors an area can show, and their menu grouping
//! - `tree`         [`Workspace`], [`Node`], [`Area`]: the data and the operations on it
//! - `join`         joining two areas that share a whole edge (rebuilds the tree around them)
//! - `solve`        [`solve`]: tree + window rectangle -> area rectangles and edges
//! - `layouts`      [`Layouts`]: all the saved workspaces (the tabs) and the one showing
//! - `presets`      the shipped layouts

mod editor_kind;
mod geom;
mod join;
mod layouts;
mod presets;
mod solve;
mod tree;

pub use editor_kind::{Category, EditorKind, MAX_VIEWPORTS};
pub use geom::{
    MIN_WIDTH, Rect, edge, header, min_height, set_touch, set_touch_auto, touch, touch_auto,
};
pub use join::SNAP;
pub use layouts::{Layouts, NAME_LIMIT};
pub use presets::{NAMES as PRESET_NAMES, by_name as preset, for_phone, is_phone, layout};
pub use solve::{Edge, Placed, Solved, solve};
pub use tree::{Area, AreaId, Axis, LayoutError, Node, Path, Side, Split, Workspace};
