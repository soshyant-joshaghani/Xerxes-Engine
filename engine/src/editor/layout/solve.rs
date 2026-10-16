//! The solver: where every area and edge is, for a rectangle of the screen.
//!
//! Called by the Dioxus UI (to place the areas) and by the Bevy stage (to place the viewport
//! cameras) with the same tree and the same rectangle size, so they always agree. The areas
//! tile the rectangle exactly (no gaps, no overlap); borders are drawn inside them.

use super::editor_kind::EditorKind;
use super::geom::{Rect, header};
use super::tree::{Area, AreaId, Axis, Node, Path, Side, Workspace};

/// An area and where it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub area: Area,
    pub rect: Rect,
}

/// The border between the two children of a split.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// Identifies the split (see [`Workspace::edge`]).
    pub path: Path,
    pub axis: Axis,
    /// Where the line is: its x for `Axis::X` (vertical line), its y for `Axis::Y`.
    pub pos: f32,
    /// The line runs from here to `to` (y for a vertical line, x for a horizontal one).
    pub from: f32,
    pub to: f32,
    /// The rectangle of the whole split, to turn a pointer position into a ratio.
    pub parent: Rect,
}

impl Edge {
    /// The grab zone of thickness `t`, centered on the line.
    pub fn hit(&self, t: f32) -> Rect {
        match self.axis {
            Axis::X => Rect::new(self.pos - t / 2.0, self.from, t, self.to - self.from),
            Axis::Y => Rect::new(self.from, self.pos - t / 2.0, self.to - self.from, t),
        }
    }

    /// The split ratio that puts the line at `pointer` (x or y, like `pos`).
    pub fn ratio_at(&self, pointer: f32) -> f32 {
        let (start, len) = match self.axis {
            Axis::X => (self.parent.x, self.parent.w),
            Axis::Y => (self.parent.y, self.parent.h),
        };
        if len <= 0.0 {
            return 0.5;
        }
        ((pointer - start) / len).clamp(0.01, 0.99)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Solved {
    pub areas: Vec<Placed>,
    /// Empty while an area is maximized.
    pub edges: Vec<Edge>,
}

impl Solved {
    pub fn rect_of(&self, id: AreaId) -> Option<Rect> {
        self.areas.iter().find(|p| p.area.id == id).map(|p| p.rect)
    }

    pub fn first(&self, kind: EditorKind) -> Option<&Placed> {
        self.areas.iter().find(|p| p.area.editor == kind)
    }

    /// The part of an area under its header: where its editor draws.
    pub fn body_of(&self, kind: EditorKind) -> Option<Rect> {
        self.first(kind).map(|p| p.rect.below(header()))
    }
}

/// Places the workspace's areas inside `bounds`.
pub fn solve(workspace: &Workspace, bounds: Rect) -> Solved {
    let mut out = Solved::default();
    if let Some(id) = workspace.maximized
        && let Some(area) = workspace.area(id)
    {
        out.areas.push(Placed {
            area: area.clone(),
            rect: bounds,
        });
        return out;
    }
    place(&workspace.root, bounds, &mut Vec::new(), &mut out);
    out
}

/// How much of `total` the first child gets: what it asked for, but never so little that
/// either child falls under its minimum. When the minimums do not fit at all (a tiny
/// window), the space is shared in proportion to them.
fn first_share(ratio: f32, total: f32, a_min: f32, b_min: f32) -> f32 {
    if a_min + b_min >= total {
        return total * a_min / (a_min + b_min);
    }
    (ratio * total).clamp(a_min, total - b_min)
}

fn place(node: &Node, rect: Rect, path: &mut Path, out: &mut Solved) {
    match node {
        Node::Area(area) => out.areas.push(Placed {
            area: area.clone(),
            rect,
        }),
        Node::Split(s) => {
            let (a_min, b_min) = (Workspace::min_size(&s.a), Workspace::min_size(&s.b));
            let (a_rect, b_rect, pos, from, to) = match s.axis {
                Axis::X => {
                    let w = first_share(s.ratio, rect.w, a_min.0, b_min.0);
                    (
                        Rect::new(rect.x, rect.y, w, rect.h),
                        Rect::new(rect.x + w, rect.y, rect.w - w, rect.h),
                        rect.x + w,
                        rect.y,
                        rect.bottom(),
                    )
                }
                Axis::Y => {
                    let h = first_share(s.ratio, rect.h, a_min.1, b_min.1);
                    (
                        Rect::new(rect.x, rect.y, rect.w, h),
                        Rect::new(rect.x, rect.y + h, rect.w, rect.h - h),
                        rect.y + h,
                        rect.x,
                        rect.right(),
                    )
                }
            };
            out.edges.push(Edge {
                path: path.clone(),
                axis: s.axis,
                pos,
                from,
                to,
                parent: rect,
            });
            path.push(Side::A);
            place(&s.a, a_rect, path, out);
            path.pop();
            path.push(Side::B);
            place(&s.b, b_rect, path, out);
            path.pop();
        }
    }
}
