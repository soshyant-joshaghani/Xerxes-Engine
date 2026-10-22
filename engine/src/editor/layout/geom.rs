//! Rectangles and the sizes every area shares, in logical pixels.

use std::sync::atomic::{AtomicBool, Ordering};

/// Touch screens get bigger targets: header, edge grab zone, bars and rows (see `theme`).
/// One flag for the whole editor, because the UI (to place the areas) and the Scene view (to
/// place its cameras) must always use the same sizes. The stage switches it on (Android, a
/// coarse pointer on the web, or the first finger that touches the screen).
static TOUCH: AtomicBool = AtomicBool::new(false);
/// Whether a finger touching the screen may switch touch sizes on (the `Auto` choice).
static TOUCH_AUTO: AtomicBool = AtomicBool::new(true);

pub fn set_touch(on: bool) {
    TOUCH.store(on, Ordering::Relaxed);
}

pub fn set_touch_auto(on: bool) {
    TOUCH_AUTO.store(on, Ordering::Relaxed);
}

pub fn touch_auto() -> bool {
    TOUCH_AUTO.load(Ordering::Relaxed)
}

pub fn touch() -> bool {
    TOUCH.load(Ordering::Relaxed)
}

/// Height of an area's header (its toolbar).
pub fn header() -> f32 {
    if touch() { 44.0 } else { 26.0 }
}

/// The smallest an area can be made by resizing.
pub const MIN_WIDTH: f32 = 120.0;

pub fn min_height() -> f32 {
    header() + 60.0
}

/// Thickness of the grab zone on an edge (centered on it).
pub fn edge() -> f32 {
    if touch() { 22.0 } else { 6.0 }
}

/// A rectangle: `x`, `y` is the top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Shrinks every side by `d` (never below zero size).
    pub fn inset(&self, d: f32) -> Rect {
        let d = d.min(self.w / 2.0).min(self.h / 2.0).max(0.0);
        Rect::new(self.x + d, self.y + d, self.w - 2.0 * d, self.h - 2.0 * d)
    }

    /// What is left under a header of height `h`: an area's body.
    pub fn below(&self, h: f32) -> Rect {
        let h = h.min(self.h);
        Rect::new(self.x, self.y + h, self.w, self.h - h)
    }

    pub fn moved(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.w, self.h)
    }

    pub fn area(&self) -> f32 {
        self.w * self.h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inset_and_below() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.inset(3.0), Rect::new(13.0, 23.0, 94.0, 44.0));
        // Never past the shorter side: the height collapses first.
        assert_eq!(r.inset(1000.0).h, 0.0);
        assert_eq!(r.below(header()), Rect::new(10.0, 46.0, 100.0, 24.0));
        assert!(r.contains(10.0, 20.0) && !r.contains(110.0, 20.0));
    }
}
