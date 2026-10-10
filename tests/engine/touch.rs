//! Touch sizes: a finger needs bigger targets than a mouse. The switch is one flag shared by
//! the UI and the Scene view, so it is tested in a binary of its own (the other layout tests
//! assume the mouse sizes).

use xerxes_engine::editor::layout::{
    Axis, EditorKind, MIN_WIDTH, Rect, Workspace, edge, for_phone, header, layout, min_height,
    preset, set_touch, solve, touch,
};
use xerxes_engine::editor::theme::{Insets, Layout, toolbar};

fn tiles(ws: &Workspace, bounds: Rect) {
    let solved = solve(ws, bounds);
    let total: f32 = solved.areas.iter().map(|p| p.rect.area()).sum();
    assert!(
        (total - bounds.area()).abs() < bounds.area() * 1e-4,
        "{}: areas cover {total} of {}",
        ws.name,
        bounds.area()
    );
}

#[test]
fn touch_sizes_are_bigger_and_everything_still_fits() {
    // Mouse first.
    assert!(!touch());
    let (mouse_header, mouse_edge, mouse_bar) = (header(), edge(), toolbar());

    set_touch(true);
    assert!(touch());
    assert!(header() > mouse_header && edge() > mouse_edge && toolbar() > mouse_bar);
    assert!(header() >= 40.0, "a header a finger can hit");
    assert!(edge() >= 18.0, "an edge a finger can grab");
    assert_eq!(min_height(), header() + 60.0);

    // The frame follows: the content starts under the taller bar and says so.
    let frame = Layout::for_window(bevy_vec(1100.0, 519.0), Insets::default());
    assert!(frame.touch);
    assert_eq!(frame.content().y, toolbar());

    // Every shipped layout still tiles a phone, a tablet and a desktop, in both orientations.
    for name in xerxes_engine::editor::layout::PRESET_NAMES {
        let ws = preset(name).unwrap();
        for (w, h) in [
            (1100.0, 519.0 - toolbar()),
            (400.0, 800.0 - toolbar()),
            (1280.0, 800.0 - toolbar()),
        ] {
            tiles(&ws, Rect::new(0.0, 0.0, w, h));
        }
    }
    // The phone layouts keep their areas usable with the bigger header.
    for ws in [for_phone(1100.0, 519.0), for_phone(400.0, 800.0)] {
        let size = if ws.name == "Phone" {
            Rect::new(0.0, 0.0, 1100.0, 519.0 - toolbar())
        } else {
            Rect::new(0.0, 0.0, 400.0, 800.0 - toolbar())
        };
        for placed in solve(&ws, size).areas {
            assert!(
                placed.rect.w >= MIN_WIDTH - 1e-3,
                "{}: {:?}",
                ws.name,
                placed.rect
            );
            assert!(
                placed.rect.h >= min_height() - 1e-3,
                "{}: {:?}",
                ws.name,
                placed.rect
            );
            let body = placed.rect.below(header());
            assert!(body.h >= 60.0 - 1e-3);
        }
    }
    // A split needs room for two areas at the touch size.
    assert!(!Workspace::can_split(
        Rect::new(0.0, 0.0, 500.0, 2.0 * min_height() - 1.0),
        Axis::Y
    ));
    assert!(layout().count(EditorKind::Viewport) == 1);

    // And back to the mouse.
    set_touch(false);
    assert_eq!(
        (header(), edge(), toolbar()),
        (mouse_header, mouse_edge, mouse_bar)
    );
}

fn bevy_vec(x: f32, y: f32) -> bevy::math::Vec2 {
    bevy::math::Vec2::new(x, y)
}
