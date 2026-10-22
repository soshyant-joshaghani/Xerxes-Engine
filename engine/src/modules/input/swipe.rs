//! Swipes: a drag on the screen, by a finger or the mouse, read as one of four directions.
//!
//! A press makes one swipe at most: after it, the drag is spent until the pointer is released, so
//! a long swipe is one swipe however far it goes. A swipe carries where its press started, as a
//! fraction of the window's height from the top, so a binding can ignore presses that began in a
//! header (see [`crate::modules::input::Binding::Swipe`]).
//!
//! Pure: [`Swipes::update`] takes the pointer each frame, so the rules are tested without a window.

use bevy::math::Vec2;

/// How far, in logical pixels, a drag must go along one axis to be a swipe.
pub const STEP: f32 = 36.0;

/// A direction of swipe. Up and down are on the screen (y grows downward).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Swipe {
    Left,
    Right,
    Up,
    Down,
}

/// A swipe that was made this frame, and where its press started (a fraction of the window's
/// height from the top, 0 at the top edge).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Made {
    pub swipe: Swipe,
    pub start: f32,
}

/// The state of the pointer's drag.
#[derive(Debug, Clone, Copy, Default)]
pub struct Swipes {
    anchor: Option<Vec2>,
    start: f32,
    spent: bool,
}

impl Swipes {
    /// Feeds this frame's pointer, in logical pixels (y down), and the window's height. `None`
    /// means nothing is pressed; the drag then ends, and the next press starts a new one.
    pub fn update(&mut self, pointer: Option<Vec2>, height: f32) -> Option<Made> {
        let Some(now) = pointer else {
            *self = Self::default();
            return None;
        };
        let Some(anchor) = self.anchor else {
            self.anchor = Some(now);
            self.start = if height > 0.0 { now.y / height } else { 0.0 };
            return None;
        };
        if self.spent {
            return None;
        }
        let by = now - anchor;
        let swipe = if by.x.abs() >= STEP && by.x.abs() > by.y.abs() {
            if by.x > 0.0 {
                Swipe::Right
            } else {
                Swipe::Left
            }
        } else if -by.y >= STEP && -by.y > by.x.abs() {
            Swipe::Up
        } else if by.y >= STEP && by.y > by.x.abs() {
            Swipe::Down
        } else {
            return None;
        };
        // One swipe per press: the rest of this drag is ignored until the pointer is released.
        self.spent = true;
        Some(Made {
            swipe,
            start: self.start,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEIGHT: f32 = 1000.0;

    fn at(x: f32, y: f32) -> Option<Vec2> {
        Some(Vec2::new(x, y))
    }

    #[test]
    fn a_still_pointer_makes_no_swipe() {
        let mut s = Swipes::default();
        assert_eq!(s.update(at(100.0, 300.0), HEIGHT), None);
        assert_eq!(s.update(at(101.0, 299.0), HEIGHT), None);
    }

    #[test]
    fn each_direction_is_read() {
        let cases = [
            (STEP, 0.0, Swipe::Right),
            (-STEP, 0.0, Swipe::Left),
            (0.0, -STEP, Swipe::Up),
            (0.0, STEP, Swipe::Down),
        ];
        for (dx, dy, want) in cases {
            let mut s = Swipes::default();
            s.update(at(500.0, 500.0), HEIGHT);
            let got = s.update(at(500.0 + dx, 500.0 + dy), HEIGHT);
            assert_eq!(got.map(|m| m.swipe), Some(want));
        }
    }

    #[test]
    fn less_than_a_step_is_not_a_swipe() {
        let mut s = Swipes::default();
        s.update(at(500.0, 500.0), HEIGHT);
        assert_eq!(s.update(at(500.0 + STEP - 1.0, 500.0), HEIGHT), None);
    }

    #[test]
    fn the_dominant_axis_decides_a_diagonal() {
        let mut s = Swipes::default();
        s.update(at(500.0, 500.0), HEIGHT);
        assert_eq!(
            s.update(at(550.0, 520.0), HEIGHT).map(|m| m.swipe),
            Some(Swipe::Right)
        );
    }

    #[test]
    fn a_long_swipe_is_one_swipe() {
        let mut s = Swipes::default();
        s.update(at(0.0, 500.0), HEIGHT);
        assert!(s.update(at(STEP, 500.0), HEIGHT).is_some());
        assert_eq!(s.update(at(4.0 * STEP, 500.0), HEIGHT), None);
    }

    #[test]
    fn a_fast_swipe_past_the_step_in_one_frame_is_one_swipe() {
        let mut s = Swipes::default();
        s.update(at(0.0, 500.0), HEIGHT);
        assert!(s.update(at(9.0 * STEP, 500.0), HEIGHT).is_some());
        assert_eq!(s.update(at(20.0 * STEP, 500.0), HEIGHT), None);
    }

    #[test]
    fn releasing_ends_the_press_and_the_next_press_is_new() {
        let mut s = Swipes::default();
        s.update(at(0.0, 500.0), HEIGHT);
        assert!(s.update(at(STEP, 500.0), HEIGHT).is_some());
        assert_eq!(s.update(None, HEIGHT), None);
        s.update(at(0.0, 500.0), HEIGHT);
        assert!(s.update(at(-STEP, 500.0), HEIGHT).is_some());
    }

    #[test]
    fn the_swipe_records_where_its_press_started() {
        let mut s = Swipes::default();
        s.update(at(500.0, 250.0), HEIGHT);
        let made = s.update(at(500.0 + STEP, 250.0), HEIGHT).unwrap();
        assert!((made.start - 0.25).abs() < 1e-6);
    }
}
