//! Anchoring: Unity's `RectTransform` for UI elements. An element is placed relative to its
//! parent's rectangle by two anchors (fractions of the parent, bottom-left is `(0, 0)`), a pivot,
//! a position and a size.
//!
//! ```ignore
//! // 16 px from the top-left corner of the parent, sized by its content.
//! .with(RectTransform::anchored(Anchor::TopLeft, Vec2::new(16.0, -16.0), Vec2::new(0.0, 0.0)))
//! // Fill the parent.
//! .with(RectTransform::stretch(0.0, 0.0, 0.0, 0.0))
//! ```
//!
//! The component is the source of the element's `Node` position and size: it sets
//! `position_type`, `left/right/top/bottom`, the margins that carry the pixel offsets, `width`
//! and `height`, and (only when it has to) the `UiTransform` translation that applies the pivot.
//! Flexbox fields (`flex_direction`, `padding`, ...) stay the `Node`'s own.
//!
//! The pivot is folded into the layout box itself (margins, or anchoring to the far edge)
//! whenever the box's size on that axis is known — which is every case except a content-sized
//! axis with a fractional (non-edge) pivot. A `UiTransform.translation` is used only for that
//! remaining case. This matters because bevy_ui picking hit-tests the element's layout box, not
//! its rendered position: a translation moves the render box away from the layout box, so a
//! right/bottom-anchored button would draw in one place and be clickable in another. Keeping the
//! layout box equal to the render box keeps anchored elements clickable where they are drawn.

use bevy::prelude::*;

/// Where on the parent an element is anchored (Unity's anchor presets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Middle,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Anchor {
    /// The point on the parent, `(0, 0)` bottom-left to `(1, 1)` top-right.
    pub fn point(self) -> Vec2 {
        let x = match self {
            Anchor::TopLeft | Anchor::MiddleLeft | Anchor::BottomLeft => 0.0,
            Anchor::TopCenter | Anchor::Middle | Anchor::BottomCenter => 0.5,
            Anchor::TopRight | Anchor::MiddleRight | Anchor::BottomRight => 1.0,
        };
        let y = match self {
            Anchor::BottomLeft | Anchor::BottomCenter | Anchor::BottomRight => 0.0,
            Anchor::MiddleLeft | Anchor::Middle | Anchor::MiddleRight => 0.5,
            Anchor::TopLeft | Anchor::TopCenter | Anchor::TopRight => 1.0,
        };
        Vec2::new(x, y)
    }
}

/// Unity's RectTransform. Y is up.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct RectTransform {
    /// The anchors, as fractions of the parent. Equal on an axis: the element has a size there
    /// (`size`). Different: it stretches between them.
    pub anchor_min: Vec2,
    pub anchor_max: Vec2,
    /// The point of the element that `position` places, as a fraction of its own size.
    pub pivot: Vec2,
    /// Unity's anchoredPosition: the pivot's offset from the anchor (pixels, y up).
    pub position: Vec2,
    /// Unity's sizeDelta: the size on an axis with equal anchors (0 = the size of the content),
    /// or the growth beyond the anchors on a stretched axis.
    pub size: Vec2,
}

impl RectTransform {
    /// A fixed point of the parent: the pivot is the anchor, so `position` is the offset of the
    /// element's matching corner or edge from it.
    pub fn anchored(anchor: Anchor, position: Vec2, size: Vec2) -> Self {
        let point = anchor.point();
        Self {
            anchor_min: point,
            anchor_max: point,
            pivot: point,
            position,
            size,
        }
    }

    /// Fills the parent, inset by these distances (pixels, positive is inwards).
    pub fn stretch(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        let min = Vec2::new(left, bottom);
        let max = Vec2::new(-right, -top);
        Self {
            anchor_min: Vec2::ZERO,
            anchor_max: Vec2::ONE,
            pivot: Vec2::splat(0.5),
            position: (min + max) / 2.0,
            size: max - min,
        }
    }

    pub fn pivot(mut self, pivot: Vec2) -> Self {
        self.pivot = pivot;
        self
    }

    fn stretched_x(&self) -> bool {
        self.anchor_min.x != self.anchor_max.x
    }

    fn stretched_y(&self) -> bool {
        self.anchor_min.y != self.anchor_max.y
    }
}

impl Default for RectTransform {
    fn default() -> Self {
        Self::stretch(0.0, 0.0, 0.0, 0.0)
    }
}

/// Writes the `Node` fields a [`RectTransform`] owns.
#[allow(unused_assignments)]
pub fn apply(rect: &RectTransform, node: &mut Node, transform: &mut UiTransform) {
    node.position_type = PositionType::Absolute;
    let (mut left, mut right, mut top, mut bottom) = (Val::Auto, Val::Auto, Val::Auto, Val::Auto);
    let (mut ml, mut mr, mut mt, mut mb) = (Val::ZERO, Val::ZERO, Val::ZERO, Val::ZERO);
    let (mut tx, mut ty) = (Val::ZERO, Val::ZERO);

    if rect.stretched_x() {
        // offsetMin / offsetMax of Unity: the edges are the anchors plus these.
        let min = rect.position.x - rect.size.x * rect.pivot.x;
        let max = rect.position.x + rect.size.x * (1.0 - rect.pivot.x);
        left = percent(rect.anchor_min.x * 100.0);
        right = percent((1.0 - rect.anchor_max.x) * 100.0);
        ml = px(min);
        mr = px(-max);
        node.width = Val::Auto;
    } else if rect.size.x > 0.0 {
        // Known width: the whole box, pivot included, is a layout box — no translation, so the
        // picking hit-box matches what is drawn. Anchor the left edge and fold the pivot offset
        // (`-pivot.x * width`) into the left margin.
        left = percent(rect.anchor_min.x * 100.0);
        ml = px(rect.position.x - rect.pivot.x * rect.size.x);
        node.width = px(rect.size.x);
    } else if rect.pivot.x <= 0.0 {
        // Content width, left pivot: the left edge is the anchored point, no offset to apply.
        left = percent(rect.anchor_min.x * 100.0);
        ml = px(rect.position.x);
        node.width = Val::Auto;
    } else if rect.pivot.x >= 1.0 {
        // Content width, right pivot: anchor the right edge instead, so the unknown width grows
        // leftward from it. Still a pure layout box — no translation.
        right = percent((1.0 - rect.anchor_min.x) * 100.0);
        mr = px(-rect.position.x);
        node.width = Val::Auto;
    } else {
        // Content width, fractional pivot: the offset depends on the unknown width, so it can only
        // be expressed as a translation (percent of own size). Picking will follow the layout box,
        // not this translated position — the one case anchoring can't keep clickable in place.
        left = percent(rect.anchor_min.x * 100.0);
        ml = px(rect.position.x);
        node.width = Val::Auto;
        tx = percent(-rect.pivot.x * 100.0);
    }
    if rect.stretched_y() {
        let min = rect.position.y - rect.size.y * rect.pivot.y;
        let max = rect.position.y + rect.size.y * (1.0 - rect.pivot.y);
        top = percent((1.0 - rect.anchor_max.y) * 100.0);
        bottom = percent(rect.anchor_min.y * 100.0);
        mt = px(-max);
        mb = px(min);
        node.height = Val::Auto;
    } else if rect.size.y > 0.0 {
        // Known height (y up): anchor the top edge, fold the pivot offset into the top margin.
        top = percent((1.0 - rect.anchor_min.y) * 100.0);
        mt = px(-rect.position.y - (1.0 - rect.pivot.y) * rect.size.y);
        node.height = px(rect.size.y);
    } else if rect.pivot.y >= 1.0 {
        // Content height, top pivot: the top edge is the anchored point.
        top = percent((1.0 - rect.anchor_min.y) * 100.0);
        mt = px(-rect.position.y);
        node.height = Val::Auto;
    } else if rect.pivot.y <= 0.0 {
        // Content height, bottom pivot: anchor the bottom edge, height grows upward from it.
        bottom = percent(rect.anchor_min.y * 100.0);
        mb = px(rect.position.y);
        node.height = Val::Auto;
    } else {
        // Content height, fractional pivot: only expressible as a translation (see the X case).
        top = percent((1.0 - rect.anchor_min.y) * 100.0);
        mt = px(-rect.position.y);
        node.height = Val::Auto;
        ty = percent(-(1.0 - rect.pivot.y) * 100.0);
    }

    node.left = left;
    node.right = right;
    node.top = top;
    node.bottom = bottom;
    node.margin = UiRect {
        left: ml,
        right: mr,
        top: mt,
        bottom: mb,
    };
    transform.translation = Val2::new(tx, ty);
}

pub(super) fn apply_rect_transforms(
    mut elements: Query<(&RectTransform, &mut Node, &mut UiTransform), Changed<RectTransform>>,
) {
    for (rect, mut node, mut transform) in &mut elements {
        apply(rect, &mut node, &mut transform);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placed(rect: RectTransform) -> (Node, UiTransform) {
        let (mut node, mut transform) = (Node::default(), UiTransform::default());
        apply(&rect, &mut node, &mut transform);
        (node, transform)
    }

    #[test]
    fn a_top_left_anchor_offsets_from_the_corner() {
        let (node, t) = placed(RectTransform::anchored(
            Anchor::TopLeft,
            Vec2::new(16.0, -16.0),
            Vec2::new(200.0, 0.0),
        ));
        assert_eq!(node.position_type, PositionType::Absolute);
        assert_eq!(node.left, Val::Percent(0.0));
        assert_eq!(node.margin.left, Val::Px(16.0));
        assert_eq!(node.top, Val::Percent(0.0));
        assert_eq!(node.margin.top, Val::Px(16.0));
        assert_eq!(node.width, Val::Px(200.0));
        assert_eq!(node.height, Val::Auto);
        // No translation: the layout box is already the render box, so picking matches it.
        assert_eq!(t.translation, Val2::new(Val::ZERO, Val::ZERO));
    }

    #[test]
    fn a_fixed_size_bottom_right_anchor_folds_the_pivot_into_the_margins() {
        // The whole box stays a layout box (no translation), so its picking hit-box is where it
        // is drawn. Left edge = parent right (100%) + position.x - pivot.x * width.
        let (node, t) = placed(RectTransform::anchored(
            Anchor::BottomRight,
            Vec2::new(-10.0, 10.0),
            Vec2::new(80.0, 30.0),
        ));
        assert_eq!(node.left, Val::Percent(100.0));
        assert_eq!(node.margin.left, Val::Px(-10.0 - 80.0));
        assert_eq!(node.width, Val::Px(80.0));
        assert_eq!(node.top, Val::Percent(100.0));
        assert_eq!(node.margin.top, Val::Px(-10.0 - 30.0));
        assert_eq!(node.height, Val::Px(30.0));
        assert_eq!(t.translation, Val2::new(Val::ZERO, Val::ZERO));
    }

    #[test]
    fn a_fixed_size_top_right_anchor_is_a_pure_layout_box() {
        // Regression for the picking bug: a TopRight-anchored button with a fixed size must carry
        // no UiTransform translation, so bevy_ui picking hit-tests it where it renders.
        let (node, t) = placed(RectTransform::anchored(
            Anchor::TopRight,
            Vec2::new(-10.0, -8.0),
            Vec2::new(28.0, 24.0),
        ));
        assert_eq!(node.left, Val::Percent(100.0));
        assert_eq!(node.margin.left, Val::Px(-10.0 - 28.0));
        assert_eq!(node.width, Val::Px(28.0));
        assert_eq!(node.top, Val::Percent(0.0));
        assert_eq!(node.margin.top, Val::Px(8.0));
        assert_eq!(node.height, Val::Px(24.0));
        assert_eq!(t.translation, Val2::new(Val::ZERO, Val::ZERO));
    }

    #[test]
    fn a_content_sized_right_pivot_anchors_the_far_edge_without_translating() {
        // Width unknown, right pivot: anchor the right edge so the box grows leftward from it,
        // still with no translation.
        let (node, t) = placed(RectTransform::anchored(
            Anchor::MiddleRight,
            Vec2::new(-12.0, 0.0),
            Vec2::new(0.0, 0.0),
        ));
        assert_eq!(node.right, Val::Percent(0.0));
        assert_eq!(node.margin.right, Val::Px(12.0));
        assert_eq!(node.left, Val::Auto);
        assert_eq!(node.width, Val::Auto);
        assert_eq!(t.translation.x, Val::ZERO);
    }

    #[test]
    fn a_content_sized_bottom_pivot_anchors_the_bottom_edge_without_translating() {
        let (node, t) = placed(RectTransform::anchored(
            Anchor::BottomCenter,
            Vec2::new(0.0, 6.0),
            Vec2::new(0.0, 0.0),
        ));
        assert_eq!(node.bottom, Val::Percent(0.0));
        assert_eq!(node.margin.bottom, Val::Px(6.0));
        assert_eq!(node.top, Val::Auto);
        assert_eq!(node.height, Val::Auto);
        assert_eq!(t.translation.y, Val::ZERO);
    }

    #[test]
    fn the_system_applies_a_top_right_anchor_with_no_translation() {
        // Drive the real `apply_rect_transforms` system in a running App: a TopRight button must
        // end up with its pivot folded into the layout box (translation stays identity), so
        // bevy_ui picking hit-tests it where it is drawn.
        let mut app = App::new();
        app.add_systems(Update, apply_rect_transforms);
        let button = app
            .world_mut()
            .spawn((
                RectTransform::anchored(
                    Anchor::TopRight,
                    Vec2::new(-10.0, -8.0),
                    Vec2::new(28.0, 24.0),
                ),
                Node::default(),
                UiTransform::default(),
            ))
            .id();
        app.update();

        let world = app.world();
        let node = world.get::<Node>(button).unwrap();
        let transform = world.get::<UiTransform>(button).unwrap();
        assert_eq!(node.left, Val::Percent(100.0));
        assert_eq!(node.margin.left, Val::Px(-38.0));
        assert_eq!(node.width, Val::Px(28.0));
        assert_eq!(transform.translation, Val2::new(Val::ZERO, Val::ZERO));
    }

    #[test]
    fn a_content_sized_center_pivot_still_needs_a_translation() {
        // The one case the layout box can't express: unknown size, fractional pivot.
        let (node, t) = placed(
            RectTransform::anchored(Anchor::Middle, Vec2::ZERO, Vec2::ZERO)
                .pivot(Vec2::new(0.5, 0.5)),
        );
        assert_eq!(node.width, Val::Auto);
        assert_eq!(node.height, Val::Auto);
        assert_eq!(
            t.translation,
            Val2::new(Val::Percent(-50.0), Val::Percent(-50.0))
        );
    }

    #[test]
    fn stretch_insets_every_edge() {
        let (node, _) = placed(RectTransform::stretch(4.0, 8.0, 12.0, 16.0));
        assert_eq!(node.left, Val::Percent(0.0));
        assert_eq!(node.margin.left, Val::Px(4.0));
        assert_eq!(node.right, Val::Percent(0.0));
        assert_eq!(node.margin.right, Val::Px(8.0));
        assert_eq!(node.top, Val::Percent(0.0));
        assert_eq!(node.margin.top, Val::Px(12.0));
        assert_eq!(node.bottom, Val::Percent(0.0));
        assert_eq!(node.margin.bottom, Val::Px(16.0));
        assert_eq!(node.width, Val::Auto);
    }
}
