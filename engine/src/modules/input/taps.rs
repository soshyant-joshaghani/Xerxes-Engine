//! Double taps: two presses in quick succession, close together, by a finger or the mouse.
//!
//! Pure: [`DoubleTaps::press`] takes each new press with its time, so the rules are tested without
//! a window. A double tap uses both presses: a third press starts a new one.

use bevy::math::Vec2;

/// The longest gap, in seconds, between the two presses of a double tap.
pub const GAP: f32 = 0.3;

/// How far, in logical pixels, the second press may be from the first.
pub const RANGE: f32 = 40.0;

/// The last press, where and when it was.
#[derive(Debug, Clone, Copy)]
struct Tap {
    at: Vec2,
    time: f32,
}

/// The state of the double-tap detector.
#[derive(Debug, Clone, Copy, Default)]
pub struct DoubleTaps {
    last: Option<Tap>,
}

impl DoubleTaps {
    /// Feeds one new press (its place in logical pixels, y down, and the time in seconds) and the
    /// window's height. Returns where the second press began, as a fraction of the height from the
    /// top, when it completes a double tap.
    pub fn press(&mut self, at: Vec2, time: f32, height: f32) -> Option<f32> {
        let previous = self.last.replace(Tap { at, time });
        match previous {
            Some(prev) if time - prev.time <= GAP && at.distance(prev.at) <= RANGE => {
                self.last = None;
                Some(if height > 0.0 { at.y / height } else { 0.0 })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEIGHT: f32 = 1000.0;

    #[test]
    fn two_quick_presses_in_place_are_a_double_tap() {
        let mut t = DoubleTaps::default();
        assert_eq!(t.press(Vec2::new(100.0, 300.0), 1.0, HEIGHT), None);
        let start = t.press(Vec2::new(102.0, 301.0), 1.2, HEIGHT).unwrap();
        assert!((start - 0.301).abs() < 1e-6);
    }

    #[test]
    fn presses_too_far_apart_in_time_are_not_a_double_tap() {
        let mut t = DoubleTaps::default();
        t.press(Vec2::new(100.0, 300.0), 1.0, HEIGHT);
        assert_eq!(
            t.press(Vec2::new(100.0, 300.0), 1.0 + GAP + 0.01, HEIGHT),
            None
        );
    }

    #[test]
    fn presses_too_far_apart_in_place_are_not_a_double_tap() {
        let mut t = DoubleTaps::default();
        t.press(Vec2::new(100.0, 300.0), 1.0, HEIGHT);
        assert_eq!(
            t.press(Vec2::new(100.0 + RANGE + 1.0, 300.0), 1.1, HEIGHT),
            None
        );
    }

    #[test]
    fn a_triple_tap_is_one_double_tap_and_a_single() {
        let mut t = DoubleTaps::default();
        t.press(Vec2::new(100.0, 300.0), 1.0, HEIGHT);
        assert!(t.press(Vec2::new(100.0, 300.0), 1.1, HEIGHT).is_some());
        // The third press starts a new tap: it alone is not a double tap...
        assert_eq!(t.press(Vec2::new(100.0, 300.0), 1.2, HEIGHT), None);
        // ...but a fourth, quick after it, is.
        assert!(t.press(Vec2::new(100.0, 300.0), 1.3, HEIGHT).is_some());
    }
}
