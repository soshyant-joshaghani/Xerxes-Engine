//! UI in the Scene view, Unity's way: a canvas is a rectangle in the scene (centred on its
//! origin, the size of its reference resolution), and its UI is the real bevy_ui the game runs,
//! drawn by a UI camera into an image shown on that rectangle (the engine's world-space
//! canvas). Selecting a UI element outlines its rectangle; clicking picks it.
//!
//! In a 2D scene one pixel is one world unit. In a 3D scene a canvas would cover 1280 units, so
//! it is drawn at 100 pixels to the unit (a 12.8 x 7.2 rectangle).

use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use syn::{Expr, ExprCall, ExprLit, Lit};

use crate::editor::bridge::ui_view::{ViewCanvasMode, ViewUi, ViewWidget};
use crate::modules::ui::{
    Canvas, CanvasMode, CanvasSurface, Dropdown, RectTransform, Slider, Toggle, TouchPad,
    VirtualJoystick,
};

/// Pixels per world unit for a canvas shown in the Scene view.
pub fn pixels_per_unit(flat: bool) -> f32 {
    if flat { 1.0 } else { 100.0 }
}

/// Gives the entity the UI components `ui` describes. `is_root` is true when no UI element is
/// above it: such an element is a canvas (an explicit `Canvas`, or an implicit overlay one).
pub fn insert_ui(
    world: &mut World,
    id: Entity,
    ui: &ViewUi,
    is_root: bool,
    flat: bool,
    reference: [u32; 2],
) {
    let ppu = pixels_per_unit(flat);
    let mut entity = world.entity_mut(id);
    if let Some(fields) = &ui.node {
        entity.insert(node(fields));
    }
    if let Some(color) = ui.background {
        entity.insert(BackgroundColor(srgba(color)));
    }
    if let Some(text) = &ui.text {
        entity.insert((
            Text::new(text.clone()),
            TextFont::from_font_size(ui.text_size.unwrap_or(20.0)),
            TextColor(ui.text_color.map_or(Color::WHITE, srgba)),
        ));
    }
    if let Some(x) = &ui.ui_transform {
        entity.insert(UiTransform {
            rotation: Rot2::degrees(x.rotation),
            scale: Vec2::from_array(x.scale),
            ..default()
        });
    }
    if let Some(color) = ui.border_color {
        entity.insert(BorderColor::all(srgba(color)));
    }
    if let Some(z) = ui.z_index {
        entity.insert(ZIndex(z));
    }
    if ui.button {
        entity.insert(Button);
    }
    if ui.hidden {
        entity.insert(Visibility::Hidden);
    }
    if let Some(r) = &ui.rect {
        entity.insert(RectTransform {
            anchor_min: Vec2::from_array(r.anchor_min),
            anchor_max: Vec2::from_array(r.anchor_max),
            pivot: Vec2::from_array(r.pivot),
            position: Vec2::from_array(r.position),
            size: Vec2::from_array(r.size),
        });
    }
    match &ui.widget {
        Some(ViewWidget::Toggle(on)) => {
            entity.insert(Toggle::new(*on));
        }
        Some(ViewWidget::Slider { min, max, value }) => {
            entity.insert(Slider::new(*min, *max, *value));
        }
        Some(ViewWidget::Dropdown { options, selected }) => {
            entity.insert(Dropdown::new(options.clone()).selected(*selected));
        }
        Some(ViewWidget::Joystick { radius, dead_zone }) => {
            entity.insert(VirtualJoystick::new(*radius).dead_zone(*dead_zone));
        }
        Some(ViewWidget::TouchPad) => {
            entity.insert(TouchPad::new());
        }
        None => {}
    }
    // The canvas last: it never replaces an authored `Node`.
    let canvas = match &ui.canvas {
        Some(c) => {
            let [w, h] = match c.mode {
                ViewCanvasMode::World { size, .. } => size,
                _ => c.reference.unwrap_or(reference),
            };
            let ppu = match c.mode {
                ViewCanvasMode::World {
                    pixels_per_unit, ..
                } => pixels_per_unit,
                _ => ppu,
            };
            Some(Canvas::world(w, h, ppu).sorted(c.sort_order))
        }
        None if is_root => Some(Canvas::world(reference[0], reference[1], ppu)),
        None => None,
    };
    if let Some(canvas) = canvas {
        if entity.get::<Node>().is_none() {
            entity.insert(Node {
                width: percent(100),
                height: percent(100),
                ..default()
            });
        }
        entity.insert(canvas);
    }
}

fn srgba([r, g, b, a]: [f32; 4]) -> Color {
    Color::srgba(r, g, b, a)
}

// ---- Node fields ----------------------------------------------------------------------

fn expr(source: &str) -> Option<Expr> {
    syn::parse_str(source).ok()
}

fn num(expr: &Expr) -> Option<f32> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Float(f), ..
        }) => f.base10_parse().ok(),
        Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
        }) => i.base10_parse::<i64>().ok().map(|n| n as f32),
        Expr::Unary(u) if matches!(u.op, syn::UnOp::Neg(_)) => num(&u.expr).map(|n| -n),
        Expr::Paren(p) => num(&p.expr),
        _ => None,
    }
}

fn last_segment(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

fn call(expr: &Expr) -> Option<(String, Vec<&Expr>)> {
    let Expr::Call(ExprCall { func, args, .. }) = expr else {
        return None;
    };
    Some((last_segment(func)?, args.iter().collect()))
}

/// `px(10)`, `percent(50)`, `vw(..)`, `vh(..)`, `auto()`, `Val::Px(10.0)`, `Val::Auto`.
fn val(source: &str) -> Option<Val> {
    let e = expr(source)?;
    if let Some(name) = last_segment(&e) {
        return match name.as_str() {
            "Auto" => Some(Val::Auto),
            "ZERO" => Some(Val::ZERO),
            _ => None,
        };
    }
    let (name, args) = call(&e)?;
    let n = args.first().and_then(|a| num(a));
    match (name.as_str(), n) {
        ("px" | "Px", Some(n)) => Some(Val::Px(n)),
        ("percent" | "Percent", Some(n)) => Some(Val::Percent(n)),
        ("vw" | "Vw", Some(n)) => Some(Val::Vw(n)),
        ("vh" | "Vh", Some(n)) => Some(Val::Vh(n)),
        ("auto", None) => Some(Val::Auto),
        _ => None,
    }
}

/// `UiRect::all(v)`, `axes(h, v)`, `horizontal(v)`, `vertical(v)`, `top(v)`, `bottom(v)`,
/// `left(v)`, `right(v)`.
fn ui_rect(source: &str) -> Option<UiRect> {
    let e = expr(source)?;
    let (name, args) = call(&e)?;
    let v = |i: usize| args.get(i).and_then(|a| val(&quote_expr(a)));
    Some(match name.as_str() {
        "all" => UiRect::all(v(0)?),
        "axes" => UiRect::axes(v(0)?, v(1)?),
        "new" => UiRect::new(v(0)?, v(1)?, v(2)?, v(3)?),
        "horizontal" => UiRect::horizontal(v(0)?),
        "vertical" => UiRect::vertical(v(0)?),
        "top" => UiRect::top(v(0)?),
        "bottom" => UiRect::bottom(v(0)?),
        "left" => UiRect::left(v(0)?),
        "right" => UiRect::right(v(0)?),
        _ => return None,
    })
}

fn quote_expr(expr: &Expr) -> String {
    quote::ToTokens::to_token_stream(expr).to_string()
}

fn variant(source: &str) -> Option<String> {
    last_segment(&expr(source)?)
}

/// The `Node` the authored fields describe; fields the editor cannot read keep their default.
fn node(fields: &[(String, String)]) -> Node {
    use bevy::ui::{AlignItems, AlignSelf, FlexWrap};
    let mut node = Node::default();
    for (name, source) in fields {
        let v = || val(source);
        match name.as_str() {
            "width" => set(&mut node.width, v()),
            "height" => set(&mut node.height, v()),
            "min_width" => set(&mut node.min_width, v()),
            "min_height" => set(&mut node.min_height, v()),
            "max_width" => set(&mut node.max_width, v()),
            "max_height" => set(&mut node.max_height, v()),
            "left" => set(&mut node.left, v()),
            "right" => set(&mut node.right, v()),
            "top" => set(&mut node.top, v()),
            "bottom" => set(&mut node.bottom, v()),
            "row_gap" => set(&mut node.row_gap, v()),
            "column_gap" => set(&mut node.column_gap, v()),
            "flex_basis" => set(&mut node.flex_basis, v()),
            "flex_grow" => set(&mut node.flex_grow, expr(source).and_then(|e| num(&e))),
            "flex_shrink" => set(&mut node.flex_shrink, expr(source).and_then(|e| num(&e))),
            "padding" => set(&mut node.padding, ui_rect(source)),
            "margin" => set(&mut node.margin, ui_rect(source)),
            "border" => set(&mut node.border, ui_rect(source)),
            "border_radius" => {
                // `BorderRadius::all(px(8))`
                if let Some(e) = expr(source) {
                    if let Some((_, args)) = call(&e) {
                        if let Some(r) = args.first().and_then(|a| val(&quote_expr(a))) {
                            node.border_radius = BorderRadius::all(r);
                        }
                    }
                }
            }
            "flex_direction" => match variant(source).as_deref() {
                Some("Row") => node.flex_direction = FlexDirection::Row,
                Some("Column") => node.flex_direction = FlexDirection::Column,
                Some("RowReverse") => node.flex_direction = FlexDirection::RowReverse,
                Some("ColumnReverse") => node.flex_direction = FlexDirection::ColumnReverse,
                _ => {}
            },
            "flex_wrap" => match variant(source).as_deref() {
                Some("NoWrap") => node.flex_wrap = FlexWrap::NoWrap,
                Some("Wrap") => node.flex_wrap = FlexWrap::Wrap,
                Some("WrapReverse") => node.flex_wrap = FlexWrap::WrapReverse,
                _ => {}
            },
            "position_type" => match variant(source).as_deref() {
                Some("Relative") => node.position_type = PositionType::Relative,
                Some("Absolute") => node.position_type = PositionType::Absolute,
                _ => {}
            },
            "display" => match variant(source).as_deref() {
                Some("Flex") => node.display = Display::Flex,
                Some("None") => node.display = Display::None,
                Some("Grid") => node.display = Display::Grid,
                _ => {}
            },
            "justify_content" => {
                if let Some(v) = variant(source).as_deref().and_then(justify) {
                    node.justify_content = v;
                }
            }
            "align_content" => {
                if let Some(v) = variant(source).as_deref().and_then(align_content) {
                    node.align_content = v;
                }
            }
            "align_items" => match variant(source).as_deref() {
                Some("Default") => node.align_items = AlignItems::Default,
                Some("Start") => node.align_items = AlignItems::Start,
                Some("End") => node.align_items = AlignItems::End,
                Some("FlexStart") => node.align_items = AlignItems::FlexStart,
                Some("FlexEnd") => node.align_items = AlignItems::FlexEnd,
                Some("Center") => node.align_items = AlignItems::Center,
                Some("Baseline") => node.align_items = AlignItems::Baseline,
                Some("Stretch") => node.align_items = AlignItems::Stretch,
                _ => {}
            },
            "align_self" => match variant(source).as_deref() {
                Some("Auto") => node.align_self = AlignSelf::Auto,
                Some("Start") => node.align_self = AlignSelf::Start,
                Some("End") => node.align_self = AlignSelf::End,
                Some("FlexStart") => node.align_self = AlignSelf::FlexStart,
                Some("FlexEnd") => node.align_self = AlignSelf::FlexEnd,
                Some("Center") => node.align_self = AlignSelf::Center,
                Some("Baseline") => node.align_self = AlignSelf::Baseline,
                Some("Stretch") => node.align_self = AlignSelf::Stretch,
                _ => {}
            },
            _ => {}
        }
    }
    node
}

fn set<T>(target: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *target = value;
    }
}

fn justify(name: &str) -> Option<bevy::ui::JustifyContent> {
    use bevy::ui::JustifyContent::*;
    Some(match name {
        "Default" => Default,
        "Start" => Start,
        "End" => End,
        "FlexStart" => FlexStart,
        "FlexEnd" => FlexEnd,
        "Center" => Center,
        "Stretch" => Stretch,
        "SpaceBetween" => SpaceBetween,
        "SpaceEvenly" => SpaceEvenly,
        "SpaceAround" => SpaceAround,
        _ => return None,
    })
}

fn align_content(name: &str) -> Option<bevy::ui::AlignContent> {
    use bevy::ui::AlignContent::*;
    Some(match name {
        "Default" => Default,
        "Start" => Start,
        "End" => End,
        "FlexStart" => FlexStart,
        "FlexEnd" => FlexEnd,
        "Center" => Center,
        "Stretch" => Stretch,
        "SpaceBetween" => SpaceBetween,
        "SpaceEvenly" => SpaceEvenly,
        "SpaceAround" => SpaceAround,
        _ => return None,
    })
}

/// Each canvas's rectangle, outlined (Unity draws it too), so the UI's extent is visible.
pub fn draw_canvases(
    mut gizmos: Gizmos,
    opts: Res<super::ViewOpts>,
    canvases: Query<(&Canvas, &GlobalTransform)>,
) {
    if !opts.0.canvases {
        return;
    }
    for (canvas, at) in &canvases {
        let CanvasMode::WorldSpace {
            size,
            pixels_per_unit,
        } = canvas.mode
        else {
            continue;
        };
        let half = size.as_vec2() / pixels_per_unit / 2.0;
        let corners = [
            at.transform_point(Vec3::new(-half.x, half.y, 0.0)) + Vec3::Z * 5.0,
            at.transform_point(Vec3::new(half.x, half.y, 0.0)) + Vec3::Z * 5.0,
            at.transform_point(Vec3::new(half.x, -half.y, 0.0)) + Vec3::Z * 5.0,
            at.transform_point(Vec3::new(-half.x, -half.y, 0.0)) + Vec3::Z * 5.0,
        ];
        for i in 0..4 {
            gizmos.line(
                corners[i],
                corners[(i + 1) % 4],
                Color::srgb(0.55, 0.65, 0.8),
            );
        }
    }
}

// ---- Rectangles in the scene ------------------------------------------------------------

/// The four corners (world space) of a UI element's rectangle, and its area in pixels, once
/// layout has run. `None` before that, or for an element outside any canvas.
pub fn world_rect(
    entity: Entity,
    parents: &Query<&ChildOf>,
    surfaces: &Query<(&CanvasSurface, &GlobalTransform)>,
    nodes: &Query<(&ComputedNode, &UiGlobalTransform)>,
) -> Option<([Vec3; 4], f32)> {
    let (computed, at) = nodes.get(entity).ok()?;
    let mut current = entity;
    let (surface, canvas_at) = loop {
        if let Ok(found) = surfaces.get(current) {
            break found;
        }
        current = parents.get(current).ok()?.parent();
    };
    let size = computed.size();
    if size.x <= 0.0 || size.y <= 0.0 {
        return None;
    }
    let half = size / 2.0;
    let (w, h) = (surface.size.x as f32, surface.size.y as f32);
    let to_world = |x: f32, y: f32| {
        // Five pixels in front of the canvas: an outline on the quad itself is drawn behind it
        // (the quad is blended after the lines), as `draw_canvases` found for the frame.
        let local = Vec3::new(
            (x - w / 2.0) / surface.pixels_per_unit,
            (h / 2.0 - y) / surface.pixels_per_unit,
            5.0 / surface.pixels_per_unit,
        );
        canvas_at.transform_point(local)
    };
    // Through the element's own transform, so a rotated or scaled element is outlined as drawn.
    let corner = |x: f32, y: f32| {
        let p = at.transform_point2(Vec2::new(x, y));
        to_world(p.x, p.y)
    };
    Some((
        [
            corner(-half.x, -half.y),
            corner(half.x, -half.y),
            corner(half.x, half.y),
            corner(-half.x, half.y),
        ],
        size.x * size.y,
    ))
}

/// Pixels per world unit of the canvas an element is on.
pub fn surface_ppu(
    entity: Entity,
    parents: &Query<&ChildOf>,
    surfaces: &Query<(&CanvasSurface, &GlobalTransform)>,
) -> Option<f32> {
    let mut current = entity;
    loop {
        if let Ok((surface, _)) = surfaces.get(current) {
            return Some(surface.pixels_per_unit);
        }
        current = parents.get(current).ok()?.parent();
    }
}

/// Distance along `ray` to the quad `corners` (a rectangle), if it hits.
pub fn ray_quad(ray: Ray3d, corners: &[Vec3; 4]) -> Option<f32> {
    let (a, b, d) = (corners[0], corners[1], corners[3]);
    let (u, v) = (b - a, d - a);
    let normal = u.cross(v).normalize_or_zero();
    let denom = normal.dot(*ray.direction);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = normal.dot(a - ray.origin) / denom;
    if t < 0.0 {
        return None;
    }
    let p = ray.origin + *ray.direction * t - a;
    let (s, r) = (p.dot(u) / u.length_squared(), p.dot(v) / v.length_squared());
    ((0.0..=1.0).contains(&s) && (0.0..=1.0).contains(&r)).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vals_and_rects_are_read() {
        assert_eq!(val("px(14)"), Some(Val::Px(14.0)));
        assert_eq!(val("percent(100)"), Some(Val::Percent(100.0)));
        assert_eq!(val("Val::Px(2.5)"), Some(Val::Px(2.5)));
        assert_eq!(val("auto()"), Some(Val::Auto));
        assert_eq!(
            ui_rect("UiRect::axes(px(16), px(10))"),
            Some(UiRect::axes(Val::Px(16.0), Val::Px(10.0)))
        );
    }

    #[test]
    fn a_node_reads_its_authored_fields() {
        let fields = vec![
            ("width".to_string(), "percent(100)".to_string()),
            (
                "flex_direction".to_string(),
                "FlexDirection::Column".to_string(),
            ),
            (
                "justify_content".to_string(),
                "JustifyContent::Center".to_string(),
            ),
            ("row_gap".to_string(), "px(14)".to_string()),
            (
                "border_radius".to_string(),
                "BorderRadius::all(px(8))".to_string(),
            ),
        ];
        let node = node(&fields);
        assert_eq!(node.width, Val::Percent(100.0));
        assert_eq!(node.flex_direction, FlexDirection::Column);
        assert_eq!(node.justify_content, bevy::ui::JustifyContent::Center);
        assert_eq!(node.row_gap, Val::Px(14.0));
        assert_eq!(node.border_radius, BorderRadius::all(Val::Px(8.0)));
    }

    #[test]
    fn a_ray_hits_a_quad_in_front_of_it() {
        let corners = [
            Vec3::new(-1.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(-1.0, -1.0, 0.0),
        ];
        let hit = Ray3d::new(Vec3::new(0.2, 0.2, 5.0), Dir3::NEG_Z);
        assert_eq!(ray_quad(hit, &corners), Some(5.0));
        let miss = Ray3d::new(Vec3::new(3.0, 0.0, 5.0), Dir3::NEG_Z);
        assert_eq!(ray_quad(miss, &corners), None);
    }
}
