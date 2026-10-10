//! The layout data and every operation on it. Operations return an error instead of
//! panicking or silently doing nothing, so the UI can grey an item out (`can_*`) and a bad
//! saved file can never take the editor down.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::editor_kind::EditorKind;
use super::geom::Rect;
use super::geom::{MIN_WIDTH, header, min_height};

/// Identifies an area inside its workspace for as long as it exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AreaId(pub u32);

/// A rectangle of the screen showing one editor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Area {
    pub id: AreaId,
    pub editor: EditorKind,
}

/// Which way a split lays its two children out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    /// Side by side (left, right): the edge between them is a vertical line.
    X,
    /// Stacked (top, bottom): the edge is a horizontal line.
    Y,
}

/// A child of a split: `A` is the left or top one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    A,
    B,
}

/// Where a split sits in the tree: the sides to take from the root. Identifies an edge.
pub type Path = Vec<Side>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Split {
    pub axis: Axis,
    /// The share of the space that `a` asks for (0 to 1). The solver keeps both children at
    /// least their minimum size, so this is a wish, not a guarantee.
    pub ratio: f32,
    pub a: Node,
    pub b: Node,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Area(Area),
    Split(Box<Split>),
}

impl Node {
    /// An area for a preset (the workspace numbers it).
    pub fn area(editor: EditorKind) -> Node {
        Node::Area(Area {
            id: AreaId(0),
            editor,
        })
    }

    pub fn split(axis: Axis, ratio: f32, a: Node, b: Node) -> Node {
        Node::Split(Box::new(Split { axis, ratio, a, b }))
    }

    pub(super) fn at(&self, path: &[Side]) -> Option<&Node> {
        let Some((side, rest)) = path.split_first() else {
            return Some(self);
        };
        match self {
            Node::Split(s) => match side {
                Side::A => s.a.at(rest),
                Side::B => s.b.at(rest),
            },
            Node::Area(_) => None,
        }
    }

    pub(super) fn at_mut(&mut self, path: &[Side]) -> Option<&mut Node> {
        let Some((side, rest)) = path.split_first() else {
            return Some(self);
        };
        match self {
            Node::Split(s) => match side {
                Side::A => s.a.at_mut(rest),
                Side::B => s.b.at_mut(rest),
            },
            Node::Area(_) => None,
        }
    }

    pub(super) fn visit<'a>(&'a self, out: &mut Vec<&'a Area>) {
        match self {
            Node::Area(a) => out.push(a),
            Node::Split(s) => {
                s.a.visit(out);
                s.b.visit(out);
            }
        }
    }

    fn area_mut(&mut self, id: AreaId) -> Option<&mut Area> {
        match self {
            Node::Area(a) if a.id == id => Some(a),
            Node::Area(_) => None,
            Node::Split(s) => {
                if let Some(found) = s.a.area_mut(id) {
                    return Some(found);
                }
                s.b.area_mut(id)
            }
        }
    }

    /// The node that is exactly the area `id` (to replace it).
    fn leaf_mut(&mut self, id: AreaId) -> Option<&mut Node> {
        if matches!(self, Node::Area(a) if a.id == id) {
            return Some(self);
        }
        match self {
            Node::Area(_) => None,
            Node::Split(s) => {
                if let Some(found) = s.a.leaf_mut(id) {
                    return Some(found);
                }
                s.b.leaf_mut(id)
            }
        }
    }

    /// Removes the area `id`: its sibling takes the parent's place.
    fn remove(&mut self, id: AreaId) -> bool {
        let Node::Split(s) = &mut *self else {
            return false;
        };
        let is = |n: &Node| matches!(n, Node::Area(a) if a.id == id);
        if is(&s.a) {
            *self = s.b.clone();
            true
        } else if is(&s.b) {
            *self = s.a.clone();
            true
        } else {
            s.a.remove(id) || s.b.remove(id)
        }
    }

    fn renumber(&mut self, next: &mut u32) {
        match self {
            Node::Area(a) => {
                a.id = AreaId(*next);
                *next += 1;
            }
            Node::Split(s) => {
                s.a.renumber(next);
                s.b.renumber(next);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    NoArea(AreaId),
    NoEdge,
    /// The only area of a workspace cannot be closed.
    LastArea,
    /// The last workspace cannot be removed.
    LastWorkspace,
    /// The two areas are not neighbours that share a whole edge.
    NotAdjacent,
    /// Joining them would leave a layout that straight cuts cannot make.
    CannotRepresent,
    /// The editor is already open as many times as it allows.
    TooMany(EditorKind),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutError::NoArea(id) => write!(f, "no area {}", id.0),
            LayoutError::NoEdge => write!(f, "no such edge"),
            LayoutError::LastArea => write!(f, "the last area cannot be closed"),
            LayoutError::LastWorkspace => write!(f, "the last layout cannot be removed"),
            LayoutError::NotAdjacent => write!(f, "those areas do not share a whole edge"),
            LayoutError::CannotRepresent => write!(f, "that join would leave an impossible layout"),
            LayoutError::TooMany(kind) => {
                write!(f, "{} is already open", kind.title())
            }
        }
    }
}

impl std::error::Error for LayoutError {}

/// A named layout: the tree of areas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub root: Node,
    /// The area shown alone over the whole screen (Blender's Ctrl+Space), if any.
    #[serde(default)]
    pub maximized: Option<AreaId>,
    #[serde(default)]
    next_id: u32,
}

impl Workspace {
    /// A workspace from a tree built with [`Node::area`] / [`Node::split`]: the areas are
    /// numbered here, in reading order.
    pub fn new(name: impl Into<String>, root: Node) -> Self {
        let mut ws = Self {
            name: name.into(),
            root,
            maximized: None,
            next_id: 1,
        };
        ws.renumber();
        ws
    }

    fn renumber(&mut self) {
        let mut next = 1;
        self.root.renumber(&mut next);
        self.next_id = next;
    }

    /// Makes a workspace read from a file safe to use: unique ids, a valid maximized area,
    /// ratios in range.
    pub fn repair(&mut self) {
        let ids: Vec<u32> = self.areas().iter().map(|a| a.id.0).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        if sorted.len() != ids.len() || sorted.first() == Some(&0) {
            self.renumber();
            self.maximized = None;
        } else {
            self.next_id = sorted.last().map_or(1, |m| m + 1);
        }
        if let Some(id) = self.maximized
            && self.area(id).is_none()
        {
            self.maximized = None;
        }
        fn clamp(node: &mut Node) {
            if let Node::Split(s) = node {
                s.ratio = if s.ratio.is_finite() {
                    s.ratio.clamp(0.01, 0.99)
                } else {
                    0.5
                };
                clamp(&mut s.a);
                clamp(&mut s.b);
            }
        }
        clamp(&mut self.root);
    }

    /// Every area, left to right and top to bottom.
    pub fn areas(&self) -> Vec<&Area> {
        let mut out = Vec::new();
        self.root.visit(&mut out);
        out
    }

    pub fn area(&self, id: AreaId) -> Option<&Area> {
        self.areas().into_iter().find(|a| a.id == id)
    }

    /// How many areas show `kind`.
    pub fn count(&self, kind: EditorKind) -> usize {
        self.areas().iter().filter(|a| a.editor == kind).count()
    }

    /// Whether area `id` may show `kind` (it may always keep the one it has).
    pub fn can_show(&self, id: AreaId, kind: EditorKind) -> bool {
        let Some(area) = self.area(id) else {
            return false;
        };
        area.editor == kind
            || kind
                .max_instances()
                .is_none_or(|max| self.count(kind) < max)
    }

    /// Changes the editor an area shows.
    pub fn set_editor(&mut self, id: AreaId, kind: EditorKind) -> Result<(), LayoutError> {
        if self.area(id).is_none() {
            return Err(LayoutError::NoArea(id));
        }
        if !self.can_show(id, kind) {
            return Err(LayoutError::TooMany(kind));
        }
        if let Some(area) = self.root.area_mut(id) {
            area.editor = kind;
        }
        Ok(())
    }

    /// Whether an area of `rect` is big enough to split along `axis` into two areas.
    pub fn can_split(rect: Rect, axis: Axis) -> bool {
        match axis {
            Axis::X => rect.w >= 2.0 * MIN_WIDTH,
            Axis::Y => rect.h >= 2.0 * min_height(),
        }
    }

    /// Splits area `id` along `axis`; the old area keeps the first position and the new one
    /// shows the same editor (an editor that allows no second copy gives an `Empty` area).
    /// Returns the new area.
    pub fn split(&mut self, id: AreaId, axis: Axis, ratio: f32) -> Result<AreaId, LayoutError> {
        let editor = self.area(id).ok_or(LayoutError::NoArea(id))?.editor;
        let copy = if editor
            .max_instances()
            .is_some_and(|max| self.count(editor) >= max)
        {
            EditorKind::Empty
        } else {
            editor
        };
        let new = AreaId(self.next_id);
        self.next_id += 1;
        let leaf = self.root.leaf_mut(id).ok_or(LayoutError::NoArea(id))?;
        let old = leaf.clone();
        *leaf = Node::split(
            axis,
            ratio.clamp(0.01, 0.99),
            old,
            Node::Area(Area {
                id: new,
                editor: copy,
            }),
        );
        Ok(new)
    }

    /// Closes area `id`: its sibling takes the space.
    pub fn close(&mut self, id: AreaId) -> Result<(), LayoutError> {
        if self.area(id).is_none() {
            return Err(LayoutError::NoArea(id));
        }
        if !self.root.remove(id) {
            return Err(LayoutError::LastArea);
        }
        if self.maximized == Some(id) {
            self.maximized = None;
        }
        Ok(())
    }

    /// The split at `path` (an edge).
    pub fn edge(&self, path: &[Side]) -> Option<&Split> {
        match self.root.at(path)? {
            Node::Split(s) => Some(s),
            Node::Area(_) => None,
        }
    }

    /// Swaps what two areas show; they keep their places and sizes.
    pub fn swap(&mut self, a: AreaId, b: AreaId) -> Result<(), LayoutError> {
        let first = self.area(a).ok_or(LayoutError::NoArea(a))?.editor;
        let second = self.area(b).ok_or(LayoutError::NoArea(b))?.editor;
        if let Some(area) = self.root.area_mut(a) {
            area.editor = second;
        }
        if let Some(area) = self.root.area_mut(b) {
            area.editor = first;
        }
        Ok(())
    }

    /// Sets the share of the split at `path` (see [`Split::ratio`]).
    pub fn resize(&mut self, path: &[Side], ratio: f32) -> Result<(), LayoutError> {
        match self.root.at_mut(path) {
            Some(Node::Split(s)) => {
                s.ratio = if ratio.is_finite() {
                    ratio.clamp(0.01, 0.99)
                } else {
                    0.5
                };
                Ok(())
            }
            _ => Err(LayoutError::NoEdge),
        }
    }

    /// Shows area `id` alone over the whole screen, or brings the layout back.
    pub fn toggle_maximize(&mut self, id: AreaId) {
        self.maximized = if self.maximized == Some(id) || self.area(id).is_none() {
            None
        } else {
            Some(id)
        };
    }

    /// The smallest size of a node: used by the solver.
    pub(super) fn min_size(node: &Node) -> (f32, f32) {
        match node {
            Node::Area(_) => (MIN_WIDTH, min_height().max(header())),
            Node::Split(s) => {
                let (aw, ah) = Self::min_size(&s.a);
                let (bw, bh) = Self::min_size(&s.b);
                match s.axis {
                    Axis::X => (aw + bw, ah.max(bh)),
                    Axis::Y => (aw.max(bw), ah + bh),
                }
            }
        }
    }
}
