//! The color picker, Unity's: a hue wheel with a saturation / value square inside it, R G B A
//! channel bars with numbers (0 to 255), and the old color beside the new one.
//!
//! Everything is built from clickable cells (a ring of 72 hue segments, a 20 x 20 square, bars
//! of 28 steps) so it works the same in the browser and in Blitz: it needs no pointer
//! coordinates, only "the pointer is pressed on this cell" and "the pointer entered this cell
//! while pressed". A drag is the press plus entering cells; it ends on release or on leaving
//! the picker.

use dioxus::prelude::*;

const SEGMENTS: usize = 72;
const GRID: usize = 20;
/// Size of one cell of the square, and of the whole wheel, in pixels.
const CELL: f32 = 4.0;
const WHEEL: f32 = 156.0;
/// Radius at the middle of the hue ring.
const RING_MID: f32 = 64.0;

/// Hue in degrees (0 to 360), saturation and value (0 to 1) to RGB.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let h = h.rem_euclid(360.0) / 60.0;
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [r + m, g + m, b + m]
}

/// RGB to hue (degrees), saturation and value. `keep_hue` is used when the color is a gray
/// (the hue is then undefined, and the picker keeps the one it had).
pub fn rgb_to_hsv([r, g, b]: [f32; 3], keep_hue: f32) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let v = max;
    let s = if max <= 0.0 { 0.0 } else { d / max };
    let h = if d < 1e-6 {
        keep_hue
    } else if max == r {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    [h, s, v]
}

fn css([r, g, b]: [f32; 3]) -> String {
    format!(
        "rgb({}, {}, {})",
        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (b.clamp(0.0, 1.0) * 255.0).round() as u8
    )
}

/// The hue wheel and square, the channel bars and the old / new swatches. `on_change` gets the
/// new RGBA on every step of a drag.
#[component]
pub fn ColorPicker(color: [f32; 4], on_change: EventHandler<[f32; 4]>) -> Element {
    // Hue and saturation are lost in gray and black, so the picker keeps its own HSV and only
    // follows the color when something else changed it (an undo, a typed value).
    let mut hsv = use_signal(|| rgb_to_hsv([color[0], color[1], color[2]], 0.0));
    let original = use_hook(|| color);
    let mut dragging = use_signal(|| 0u8);

    let kept = *hsv.peek();
    let from_kept = hsv_to_rgb(kept[0], kept[1], kept[2]);
    let stale = (0..3).any(|i| (from_kept[i] - color[i]).abs() > 0.004);
    let [h, s, v] = if stale {
        rgb_to_hsv([color[0], color[1], color[2]], kept[0])
    } else {
        kept
    };
    let alpha = color[3];
    let rgb = hsv_to_rgb(h, s, v);

    let mut pick = move |h: f32, s: f32, v: f32| {
        hsv.set([h, s, v]);
        let c = hsv_to_rgb(h, s, v);
        on_change.call([c[0], c[1], c[2], alpha]);
    };

    let half = WHEEL / 2.0;
    let square_from = (WHEEL - GRID as f32 * CELL) / 2.0;
    let picked = (
        (s * (GRID - 1) as f32).round() as usize,
        ((1.0 - v) * (GRID - 1) as f32).round() as usize,
    );
    let hot_segment = ((h.rem_euclid(360.0) / 360.0 * SEGMENTS as f32).round() as usize) % SEGMENTS;

    rsx! {
        div {
            class: "cp",
            onmouseup: move |_| dragging.set(0),
            onmouseleave: move |_| dragging.set(0),
            div { class: "cp-wheel", style: "width: {WHEEL}px; height: {WHEEL}px;",
                for i in 0..SEGMENTS {
                    {
                        let hue = i as f32 * 360.0 / SEGMENTS as f32;
                        let a = hue.to_radians() - std::f32::consts::FRAC_PI_2;
                        let (x, y) = (half + RING_MID * a.cos() - 4.0, half + RING_MID * a.sin() - 12.0);
                        let fill = css(hsv_to_rgb(hue, 1.0, 1.0));
                        let mark = if i == hot_segment { "cp-seg hot" } else { "cp-seg" };
                        rsx! {
                            div {
                                key: "{i}",
                                class: "{mark}",
                                style: "left: {x}px; top: {y}px; background: {fill}; transform: rotate({hue}deg);",
                                onmousedown: move |_| {
                                    dragging.set(1);
                                    pick(hue, s, v);
                                },
                                onmouseenter: move |_| {
                                    if *dragging.peek() == 1 {
                                        pick(hue, s, v);
                                    }
                                },
                            }
                        }
                    }
                }
                div {
                    class: "cp-square",
                    style: "left: {square_from}px; top: {square_from}px; width: {GRID as f32 * CELL}px;",
                    for gy in 0..GRID {
                        div { key: "{gy}", class: "cp-srow",
                            for gx in 0..GRID {
                                {
                                    let cs = gx as f32 / (GRID - 1) as f32;
                                    let cv = 1.0 - gy as f32 / (GRID - 1) as f32;
                                    let fill = css(hsv_to_rgb(h, cs, cv));
                                    let cls = if (gx, gy) == picked { "cp-sv hot" } else { "cp-sv" };
                                    rsx! {
                                        div {
                                            key: "{gx}",
                                            class: "{cls}",
                                            style: "background: {fill};",
                                            onmousedown: move |_| {
                                                dragging.set(2);
                                                pick(h, cs, cv);
                                            },
                                            onmouseenter: move |_| {
                                                if *dragging.peek() == 2 {
                                                    pick(h, cs, cv);
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for (i , name) in ["R", "G", "B", "A"].into_iter().enumerate() {
                Channel {
                    key: "{name}",
                    label: name.to_string(),
                    index: i,
                    color: [rgb[0], rgb[1], rgb[2], alpha],
                    on_value: move |value: f32| {
                        let mut c = [rgb[0], rgb[1], rgb[2], alpha];
                        c[i] = value.clamp(0.0, 1.0);
                        if i < 3 {
                            let next = rgb_to_hsv([c[0], c[1], c[2]], h);
                            hsv.set(next);
                        }
                        on_change.call(c);
                    },
                }
            }
            div { class: "cp-olds",
                div { class: "cp-old", style: "background: {css([original[0], original[1], original[2]])};", title: "The color before" }
                div { class: "cp-new", style: "background: {css(rgb)};", title: "The color now" }
            }
        }
    }
}

/// One channel: a bar of steps from 0 to 1 (the channel's range with the others as they are), a
/// marker on the current step and the 0 to 255 number.
#[component]
fn Channel(
    label: String,
    index: usize,
    color: [f32; 4],
    on_value: EventHandler<f32>,
) -> Element {
    const STEPS: usize = 28;
    let mut pressed = use_signal(|| false);
    let value = color[index];
    let current = (value * (STEPS - 1) as f32).round() as usize;
    rsx! {
        div {
            class: "cp-chan",
            onmouseup: move |_| pressed.set(false),
            onmouseleave: move |_| pressed.set(false),
            div { class: "cp-name", "{label}" }
            div { class: "cp-bar",
                for step in 0..STEPS {
                    {
                        let t = step as f32 / (STEPS - 1) as f32;
                        let mut c = color;
                        c[index] = t;
                        // Alpha is shown as the color over a dark ground.
                        let shown = if index == 3 {
                            [c[0] * t, c[1] * t, c[2] * t]
                        } else {
                            [c[0], c[1], c[2]]
                        };
                        let fill = css(shown);
                        let cls = if step == current { "cp-step hot" } else { "cp-step" };
                        rsx! {
                            div {
                                key: "{step}",
                                class: "{cls}",
                                style: "background: {fill};",
                                onmousedown: move |_| {
                                    pressed.set(true);
                                    on_value.call(t);
                                },
                                onmouseenter: move |_| {
                                    if *pressed.peek() {
                                        on_value.call(t);
                                    }
                                },
                            }
                        }
                    }
                }
            }
            input {
                class: "num cp-byte",
                value: "{(value * 255.0).round() as i32}",
                oninput: move |e| {
                    if let Ok(n) = e.value().trim().parse::<f32>() {
                        if n.is_finite() {
                            on_value.call((n / 255.0).clamp(0.0, 1.0));
                        }
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_and_rgb_agree() {
        for (h, s, v) in [(0.0, 1.0, 1.0), (120.0, 0.5, 0.8), (250.0, 0.9, 0.3), (330.0, 0.2, 1.0)] {
            let rgb = hsv_to_rgb(h, s, v);
            let back = rgb_to_hsv(rgb, 0.0);
            assert!((back[0] - h).abs() < 0.01, "{h}: {back:?}");
            assert!((back[1] - s).abs() < 0.001);
            assert!((back[2] - v).abs() < 0.001);
        }
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), [1.0, 0.0, 0.0]);
        assert_eq!(hsv_to_rgb(240.0, 1.0, 1.0), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn a_gray_keeps_the_hue_it_had() {
        let hsv = rgb_to_hsv([0.5, 0.5, 0.5], 200.0);
        assert_eq!(hsv, [200.0, 0.0, 0.5]);
    }
}
