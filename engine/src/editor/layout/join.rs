//! Joining two areas, Blender's rule: they may join when they share a whole edge, however the
//! tree happens to be built. Dragging two edges into one line is enough (within [`SNAP`]).
//!
//! The tree is rebuilt around the two areas: the smallest subtree that holds both is turned
//! into rectangles, the two rectangles become one, nearly equal lines are made equal, and a
//! tree is cut out of the rectangles again. Everything outside that subtree keeps its shape,
//! so dragging an edge elsewhere still moves what it moved before. A result that no sequence
//! of straight cuts can make (a pinwheel) is refused.

use super::geom::Rect;
use super::solve::solve;
use super::tree::{Area, AreaId, Axis, LayoutError, Node, Path, Side, Workspace};

/// Lines closer than this (logical pixels) count as the same line.
pub const SNAP: f32 = 3.0;
const EPS: f32 = 0.01;

fn path_to(node: &Node, id: AreaId) -> Option<Path> {
    match node {
        Node::Area(a) => (a.id == id).then(Vec::new),
        Node::Split(s) => {
            if let Some(mut p) = path_to(&s.a, id) {
                p.insert(0, Side::A);
                return Some(p);
            }
            path_to(&s.b, id).map(|mut p| {
                p.insert(0, Side::B);
                p
            })
        }
    }
}

struct Item {
    area: Area,
    rect: Rect,
}

/// Makes lines that are within `SNAP` of each other the same value; a line near the bounds
/// becomes the bound.
fn snap(values: &mut [&mut f32], low: f32, high: f32) {
    let mut sorted: Vec<f32> = values.iter().map(|v| **v).collect();
    sorted.sort_by(f32::total_cmp);
    let mut map: Vec<(f32, f32)> = Vec::new(); // (from, to)
    let mut i = 0;
    while i < sorted.len() {
        let mut j = i;
        while j + 1 < sorted.len() && sorted[j + 1] - sorted[j] <= SNAP {
            j += 1;
        }
        let (first, last) = (sorted[i], sorted[j]);
        let to = if first - low <= SNAP {
            low
        } else if high - last <= SNAP {
            high
        } else {
            first
        };
        for v in &sorted[i..=j] {
            map.push((*v, to));
        }
        i = j + 1;
    }
    for v in values.iter_mut() {
        if let Some((_, to)) = map.iter().find(|(from, _)| *from == **v) {
            **v = *to;
        }
    }
}

/// Cuts a tree out of rectangles that tile `bounds`, trying `prefer` first at every level.
fn build(items: Vec<Item>, bounds: Rect, prefer: Axis) -> Option<Node> {
    if items.len() == 1 {
        return items.into_iter().next().map(|i| Node::Area(i.area));
    }
    for axis in [prefer, other(prefer)] {
        let (lo, hi) = match axis {
            Axis::X => (bounds.x, bounds.right()),
            Axis::Y => (bounds.y, bounds.bottom()),
        };
        let end = |r: &Rect| match axis {
            Axis::X => r.right(),
            Axis::Y => r.bottom(),
        };
        let start = |r: &Rect| match axis {
            Axis::X => r.x,
            Axis::Y => r.y,
        };
        let mut cuts: Vec<f32> = items.iter().map(|i| end(&i.rect)).collect();
        cuts.sort_by(f32::total_cmp);
        cuts.dedup_by(|a, b| (*a - *b).abs() < EPS);
        let cut = cuts.into_iter().find(|c| {
            *c > lo + EPS
                && *c < hi - EPS
                && items
                    .iter()
                    .all(|i| end(&i.rect) <= *c + EPS || start(&i.rect) >= *c - EPS)
        });
        let Some(cut) = cut else { continue };
        let (first, second): (Vec<Item>, Vec<Item>) =
            items.into_iter().partition(|i| end(&i.rect) <= cut + EPS);
        let (b1, b2, ratio) = match axis {
            Axis::X => (
                Rect::new(bounds.x, bounds.y, cut - bounds.x, bounds.h),
                Rect::new(cut, bounds.y, bounds.right() - cut, bounds.h),
                (cut - bounds.x) / bounds.w,
            ),
            Axis::Y => (
                Rect::new(bounds.x, bounds.y, bounds.w, cut - bounds.y),
                Rect::new(bounds.x, cut, bounds.w, bounds.bottom() - cut),
                (cut - bounds.y) / bounds.h,
            ),
        };
        let a = build(first, b1, other(axis))?;
        let b = build(second, b2, other(axis))?;
        return Some(Node::split(axis, ratio, a, b));
    }
    None
}

fn other(axis: Axis) -> Axis {
    match axis {
        Axis::X => Axis::Y,
        Axis::Y => Axis::X,
    }
}

impl Workspace {
    /// Whether `gone` can be joined into `keep` in a screen of `bounds`.
    pub fn can_join(&self, keep: AreaId, gone: AreaId, bounds: Rect) -> bool {
        self.clone().join(keep, gone, bounds).is_ok()
    }

    /// Joins area `gone` into `keep`: `keep` grows over it. They must be neighbours that share
    /// a whole edge (within [`SNAP`]). Returns `keep`.
    pub fn join(
        &mut self,
        keep: AreaId,
        gone: AreaId,
        bounds: Rect,
    ) -> Result<AreaId, LayoutError> {
        if keep == gone {
            return Err(LayoutError::NotAdjacent);
        }
        // Solve the real layout, not a maximized one.
        let mut plain = self.clone();
        plain.maximized = None;
        let solved = solve(&plain, bounds);
        let rk = solved.rect_of(keep).ok_or(LayoutError::NoArea(keep))?;
        let rg = solved.rect_of(gone).ok_or(LayoutError::NoArea(gone))?;
        let near = |a: f32, b: f32| (a - b).abs() <= SNAP;
        let side_by_side = (near(rk.right(), rg.x) || near(rg.right(), rk.x))
            && near(rk.y, rg.y)
            && near(rk.bottom(), rg.bottom());
        let stacked = (near(rk.bottom(), rg.y) || near(rg.bottom(), rk.y))
            && near(rk.x, rg.x)
            && near(rk.right(), rg.right());
        if !side_by_side && !stacked {
            return Err(LayoutError::NotAdjacent);
        }

        // The smallest subtree holding both areas.
        let (pk, pg) = (
            path_to(&self.root, keep).ok_or(LayoutError::NoArea(keep))?,
            path_to(&self.root, gone).ok_or(LayoutError::NoArea(gone))?,
        );
        let common: Path = pk
            .iter()
            .zip(&pg)
            .take_while(|(a, b)| a == b)
            .map(|(a, _)| *a)
            .collect();
        let region = if common.is_empty() {
            bounds
        } else {
            solved
                .edges
                .iter()
                .find(|e| e.path == common)
                .map(|e| e.parent)
                .ok_or(LayoutError::NoEdge)?
        };
        let subtree = self.root.at(&common).ok_or(LayoutError::NoEdge)?;
        let prefer = match subtree {
            Node::Split(s) => s.axis,
            Node::Area(_) => Axis::X,
        };
        let mut ids = Vec::new();
        subtree.visit(&mut ids);

        // Rectangles of the subtree's areas, the two merged into one.
        let merged = {
            let x = rk.x.min(rg.x);
            let y = rk.y.min(rg.y);
            Rect::new(
                x,
                y,
                rk.right().max(rg.right()) - x,
                rk.bottom().max(rg.bottom()) - y,
            )
        };
        let mut items: Vec<Item> = ids
            .into_iter()
            .filter(|a| a.id != gone)
            .filter_map(|a| {
                let rect = if a.id == keep {
                    merged
                } else {
                    solved.rect_of(a.id)?
                };
                Some(Item {
                    area: a.clone(),
                    rect,
                })
            })
            .collect();

        // Nearly equal lines become one line.
        let mut xs: Vec<&mut f32> = Vec::new();
        let mut ys: Vec<&mut f32> = Vec::new();
        let mut spans: Vec<(f32, f32, f32, f32)> = items
            .iter()
            .map(|i| (i.rect.x, i.rect.right(), i.rect.y, i.rect.bottom()))
            .collect();
        for (x0, x1, y0, y1) in &mut spans {
            xs.push(x0);
            xs.push(x1);
            ys.push(y0);
            ys.push(y1);
        }
        snap(&mut xs, region.x, region.right());
        snap(&mut ys, region.y, region.bottom());
        for (item, (x0, x1, y0, y1)) in items.iter_mut().zip(spans) {
            item.rect = Rect::new(x0, y0, x1 - x0, y1 - y0);
        }

        let rebuilt = build(items, region, prefer).ok_or(LayoutError::CannotRepresent)?;
        *self.root.at_mut(&common).ok_or(LayoutError::NoEdge)? = rebuilt;
        if self.maximized == Some(gone) {
            self.maximized = None;
        }
        Ok(keep)
    }
}
