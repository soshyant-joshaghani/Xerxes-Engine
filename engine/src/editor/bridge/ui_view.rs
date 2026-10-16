//! The UI part of the Scene view: what a scene's `Canvas`, `Node`, `Text`, `Button`,
//! `BackgroundColor` and widget components say, read from the parsed `.rs` (no Bevy). The stage
//! turns it into the same bevy_ui entities the game builds.
//!
//! The editor evaluates the literal forms scene files are written in: `Color::srgb(..)`,
//! `Text::new("..")`, `px(..)`, `percent(..)`, `Canvas::overlay()`, `Toggle::new(true)`, and a
//! file's own `const NAME: T = ..;` items. Anything else is left out of the view and still kept
//! as source in the document.

use syn::{Expr, ExprCall, ExprLit, ExprMethodCall, ExprUnary, Lit, UnOp};

use crate::editor::project::codec::{ComponentDoc, Field, Value};

/// What an object is as UI. `None` fields were not authored (the engine's defaults apply).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ViewUi {
    /// The `Node` struct's fields as (name, expression source), constants substituted.
    pub node: Option<Vec<(String, String)>>,
    pub background: Option<[f32; 4]>,
    pub text: Option<String>,
    pub text_size: Option<f32>,
    pub text_color: Option<[f32; 4]>,
    pub button: bool,
    /// `BorderColor::all(..)`: the color of the `Node`'s border.
    pub border_color: Option<[f32; 4]>,
    /// `ZIndex(n)`: the order among siblings.
    pub z_index: Option<i32>,
    /// `Visibility::Hidden` was authored (a banner that shows only sometimes).
    pub hidden: bool,
    pub widget: Option<ViewWidget>,
    pub canvas: Option<ViewCanvas>,
    pub rect: Option<ViewRect>,
    /// `UiTransform { rotation, scale, .. }`: turned / scaled around its centre.
    pub ui_transform: Option<ViewXform>,
}

/// A UI element's rotation (degrees, clockwise on screen) and scale: bevy_ui's `UiTransform`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewXform {
    pub rotation: f32,
    pub scale: [f32; 2],
}

impl Default for ViewXform {
    fn default() -> Self {
        Self {
            rotation: 0.0,
            scale: [1.0, 1.0],
        }
    }
}

/// A `RectTransform`: anchors, pivot, position and size (Unity's names; y is up).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewRect {
    pub anchor_min: [f32; 2],
    pub anchor_max: [f32; 2],
    pub pivot: [f32; 2],
    pub position: [f32; 2],
    pub size: [f32; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewWidget {
    Toggle(bool),
    Slider {
        min: f32,
        max: f32,
        value: f32,
    },
    Dropdown {
        options: Vec<String>,
        selected: usize,
    },
    /// A floating virtual joystick: its radius and dead zone.
    Joystick {
        radius: f32,
        dead_zone: f32,
    },
    /// A touch pad zone.
    TouchPad,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewCanvas {
    pub mode: ViewCanvasMode,
    pub sort_order: i32,
    /// `None`: the project's.
    pub reference: Option<[u32; 2]>,
    /// For a camera canvas: the camera object's name (`None`: the active camera).
    pub camera: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewCanvasMode {
    Overlay,
    Camera,
    World {
        size: [u32; 2],
        pixels_per_unit: f32,
    },
}

type Consts<'a> = &'a [(String, String)];

/// The UI an object's resolved components describe, or `None` when it is no UI element.
pub fn extract(components: &[&ComponentDoc], consts: Consts) -> Option<ViewUi> {
    let mut ui = ViewUi::default();
    let mut any = false;
    for component in components {
        let base = component.ty.split("::").next().unwrap_or(&component.ty);
        let expr = || component_expr(component, consts);
        match base {
            "Node" => {
                if let Value::Struct { fields, .. } = &component.value {
                    ui.node = Some(
                        fields
                            .iter()
                            .map(|(name, field)| (name.clone(), field_source(field, consts)))
                            .collect(),
                    );
                } else {
                    ui.node = Some(Vec::new());
                }
                any = true;
            }
            "BackgroundColor" => {
                ui.background = expr().and_then(|e| wrapped_color(&e, "BackgroundColor", consts));
                any = true;
            }
            "TextColor" => {
                ui.text_color = expr().and_then(|e| wrapped_color(&e, "TextColor", consts));
                any = true;
            }
            "Text" => {
                ui.text = expr().and_then(|e| first_string(&e));
                any = true;
            }
            "TextFont" => {
                ui.text_size = font_size(component, expr());
                any = true;
            }
            "Button" => {
                ui.button = true;
                any = true;
            }
            "BorderColor" => {
                // `BorderColor::all(c)` or the tuple form `BorderColor(c)`.
                ui.border_color = expr().and_then(|e| match e {
                    Expr::Call(call) => color(call.args.first()?, consts),
                    _ => None,
                });
                any = true;
            }
            "ZIndex" => {
                ui.z_index = expr().and_then(|e| match e {
                    Expr::Call(call) => call.args.first().and_then(number).map(|n| n as i32),
                    _ => None,
                });
                any = true;
            }
            "Visibility" => {
                ui.hidden = matches!(&component.value, Value::Unit(p) if p.ends_with("Hidden"));
            }
            "Canvas" => {
                ui.canvas = expr().and_then(|e| canvas(&e));
                any = true;
            }
            "RectTransform" => {
                ui.rect = expr()
                    .and_then(|e| rect(&e))
                    .or_else(|| rect_struct(component));
                any = true;
            }
            "UiTransform" => {
                ui.ui_transform = xform_of(component);
                any = true;
            }
            "Toggle" | "Slider" | "Dropdown" | "VirtualJoystick" | "TouchPad" => {
                ui.widget = expr().and_then(|e| widget(&e, base));
                any = true;
            }
            _ => {}
        }
    }
    any.then_some(ui)
}

/// The component as an expression: a call as written, a unit path, or its original source.
fn component_expr(component: &ComponentDoc, consts: Consts) -> Option<Expr> {
    let source = match &component.value {
        Value::Expr(src) | Value::Unit(src) => src.clone(),
        _ => component.original.clone()?,
    };
    parse(&source, consts)
}

fn parse(source: &str, consts: Consts) -> Option<Expr> {
    let expr: Expr = syn::parse_str(source).ok()?;
    Some(resolve(expr, consts, 0))
}

/// A single identifier naming a file constant becomes the constant's expression.
fn resolve(expr: Expr, consts: Consts, depth: usize) -> Expr {
    if depth > 6 {
        return expr;
    }
    if let Expr::Path(path) = &expr {
        if let Some(ident) = path.path.get_ident() {
            let name = ident.to_string();
            if let Some((_, source)) = consts.iter().find(|(n, _)| *n == name) {
                if let Ok(inner) = syn::parse_str::<Expr>(source) {
                    return resolve(inner, consts, depth + 1);
                }
            }
        }
    }
    expr
}

fn field_source(field: &Field, consts: Consts) -> String {
    let source = match field {
        Field::Num(n) => format!("{n:?}"),
        Field::Int(i) => i.to_string(),
        Field::Bool(b) => b.to_string(),
        Field::Str(s) => format!("{s:?}"),
        Field::Path(p) => p.clone(),
        Field::Expr(e) => e.clone(),
        Field::OptStr(Some(s)) => format!("Some({s:?})"),
        Field::OptStr(None) => "None".into(),
        Field::Vec2([x, y]) => format!("Vec2::new({x:?}, {y:?})"),
        Field::Vec3([x, y, z]) => format!("Vec3::new({x:?}, {y:?}, {z:?})"),
        Field::Color([r, g, b, a]) => format!("Color::srgba({r:?}, {g:?}, {b:?}, {a:?})"),
    };
    // A constant used as a whole value (`width: PANEL_WIDTH`).
    match parse(&source, consts) {
        Some(expr) => quote::ToTokens::to_token_stream(&expr).to_string(),
        None => source,
    }
}

fn number(expr: &Expr) -> Option<f32> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Float(f), ..
        }) => f.base10_parse().ok(),
        Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
        }) => i.base10_parse::<i64>().ok().map(|n| n as f32),
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr,
            ..
        }) => number(expr).map(|n| -n),
        Expr::Paren(p) => number(&p.expr),
        Expr::Cast(c) => number(&c.expr),
        _ => None,
    }
}

fn string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => Some(s.value()),
        _ => None,
    }
}

fn first_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Call(call) => call.args.first().and_then(string),
        _ => None,
    }
}

fn path_text(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(p) => Some(
            p.path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
        ),
        _ => None,
    }
}

/// `Color::srgb(..)`, `Color::srgba(..)`, `Color::srgb_u8(..)`, `Color::WHITE`, `Color::BLACK`,
/// `Color::NONE`.
pub fn color(expr: &Expr, consts: Consts) -> Option<[f32; 4]> {
    let expr = resolve(expr.clone(), consts, 0);
    if let Some(path) = path_text(&expr) {
        return match path.as_str() {
            "Color::WHITE" => Some([1.0; 4]),
            "Color::BLACK" => Some([0.0, 0.0, 0.0, 1.0]),
            "Color::NONE" => Some([0.0; 4]),
            _ => None,
        };
    }
    let Expr::Call(ExprCall { func, args, .. }) = &expr else {
        return None;
    };
    let func = path_text(func)?;
    let n: Vec<f32> = args.iter().filter_map(number).collect();
    if n.len() != args.len() {
        return None;
    }
    match (func.as_str(), n.as_slice()) {
        ("Color::srgb", [r, g, b]) => Some([*r, *g, *b, 1.0]),
        ("Color::srgba", [r, g, b, a]) => Some([*r, *g, *b, *a]),
        ("Color::srgb_u8", [r, g, b]) => Some([r / 255.0, g / 255.0, b / 255.0, 1.0]),
        ("Color::srgba_u8", [r, g, b, a]) => Some([r / 255.0, g / 255.0, b / 255.0, a / 255.0]),
        _ => None,
    }
}

/// The color a file constant named `name` holds (`const WALL: Color = Color::srgb(..)`).
pub fn const_color(name: &str, consts: &[(String, String)]) -> Option<[f32; 4]> {
    let (_, source) = consts.iter().find(|(n, _)| n == name)?;
    color(&syn::parse_str::<Expr>(source).ok()?, consts)
}

/// `Wrapper(color)`.
fn wrapped_color(expr: &Expr, wrapper: &str, consts: Consts) -> Option<[f32; 4]> {
    let Expr::Call(call) = expr else {
        return None;
    };
    if path_text(&call.func).as_deref() != Some(wrapper) {
        return None;
    }
    color(call.args.first()?, consts)
}

fn font_size(component: &ComponentDoc, expr: Option<Expr>) -> Option<f32> {
    if let Value::Struct { fields, .. } = &component.value {
        return fields.iter().find_map(|(n, f)| match (n.as_str(), f) {
            ("font_size", Field::Num(v)) => Some(*v),
            ("font_size", Field::Int(v)) => Some(*v as f32),
            _ => None,
        });
    }
    match expr? {
        Expr::Call(call)
            if path_text(&call.func).as_deref() == Some("TextFont::from_font_size") =>
        {
            call.args.first().and_then(number)
        }
        _ => None,
    }
}

/// `Canvas::overlay()` / `camera()` / `world(w, h, ppu)`, then `.sorted(n)` / `.reference(w, h)`.
fn canvas(expr: &Expr) -> Option<ViewCanvas> {
    let mut calls: Vec<&ExprMethodCall> = Vec::new();
    let mut base = expr;
    while let Expr::MethodCall(call) = base {
        calls.push(call);
        base = &call.receiver;
    }
    calls.reverse();
    let Expr::Call(ExprCall { func, args, .. }) = base else {
        return None;
    };
    let mut canvas = ViewCanvas {
        mode: ViewCanvasMode::Overlay,
        sort_order: 0,
        reference: None,
        camera: None,
    };
    match path_text(func)?.as_str() {
        "Canvas::overlay" => {}
        "Canvas::camera" => canvas.mode = ViewCanvasMode::Camera,
        "Canvas::world" => {
            let n: Vec<f32> = args.iter().filter_map(number).collect();
            let [w, h, ppu] = n.as_slice() else {
                return None;
            };
            canvas.mode = ViewCanvasMode::World {
                size: [*w as u32, *h as u32],
                pixels_per_unit: *ppu,
            };
            canvas.reference = Some([*w as u32, *h as u32]);
        }
        _ => return None,
    }
    for call in calls {
        if call.method == "camera_named" {
            canvas.camera = call.args.first().and_then(string);
            continue;
        }
        let n: Vec<f32> = call.args.iter().filter_map(number).collect();
        match (call.method.to_string().as_str(), n.as_slice()) {
            ("sorted", [order]) => canvas.sort_order = *order as i32,
            ("reference", [w, h]) => canvas.reference = Some([*w as u32, *h as u32]),
            _ => {}
        }
    }
    Some(canvas)
}

/// `Vec2::new(x, y)`.
fn vec2(expr: &Expr) -> Option<[f32; 2]> {
    let Expr::Call(call) = expr else {
        return None;
    };
    if path_text(&call.func).as_deref() != Some("Vec2::new") {
        return None;
    }
    let n: Vec<f32> = call.args.iter().filter_map(number).collect();
    match n.as_slice() {
        [x, y] => Some([*x, *y]),
        _ => None,
    }
}

/// `Anchor::TopLeft` and the other presets: the point on the parent (y up).
fn anchor_point(expr: &Expr) -> Option<[f32; 2]> {
    let name = path_text(expr)?;
    let name = name.strip_prefix("Anchor::")?;
    let x = match name {
        "TopLeft" | "MiddleLeft" | "BottomLeft" => 0.0,
        "TopCenter" | "Middle" | "BottomCenter" => 0.5,
        "TopRight" | "MiddleRight" | "BottomRight" => 1.0,
        _ => return None,
    };
    let y = match name {
        "BottomLeft" | "BottomCenter" | "BottomRight" => 0.0,
        "MiddleLeft" | "Middle" | "MiddleRight" => 0.5,
        _ => 1.0,
    };
    Some([x, y])
}

/// `RectTransform::anchored(Anchor::X, Vec2::new(..), Vec2::new(..))`,
/// `RectTransform::stretch(l, r, t, b)`, then `.pivot(Vec2::new(..))`.
fn rect(expr: &Expr) -> Option<ViewRect> {
    let (expr, pivot) = match expr {
        Expr::MethodCall(call) if call.method == "pivot" => {
            (&*call.receiver, call.args.first().and_then(vec2))
        }
        other => (other, None),
    };
    let Expr::Call(call) = expr else {
        return None;
    };
    let mut rect = match path_text(&call.func)?.as_str() {
        "RectTransform::anchored" => {
            let point = anchor_point(call.args.first()?)?;
            ViewRect {
                anchor_min: point,
                anchor_max: point,
                pivot: point,
                position: vec2(call.args.get(1)?)?,
                size: vec2(call.args.get(2)?)?,
            }
        }
        "RectTransform::stretch" => {
            let n: Vec<f32> = call.args.iter().filter_map(number).collect();
            let [left, right, top, bottom] = n.as_slice() else {
                return None;
            };
            let (min, max) = ([*left, *bottom], [-*right, -*top]);
            ViewRect {
                anchor_min: [0.0, 0.0],
                anchor_max: [1.0, 1.0],
                pivot: [0.5, 0.5],
                position: [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0],
                size: [max[0] - min[0], max[1] - min[1]],
            }
        }
        _ => return None,
    };
    if let Some(pivot) = pivot {
        rect.pivot = pivot;
    }
    Some(rect)
}

/// A `RectTransform { anchor_min: Vec2::new(..), .. }` literal.
fn rect_struct(component: &ComponentDoc) -> Option<ViewRect> {
    let Value::Struct { fields, .. } = &component.value else {
        return None;
    };
    let get = |name: &str, default: [f32; 2]| {
        fields
            .iter()
            .find_map(|(n, f)| match (n.as_str() == name, f) {
                (true, Field::Vec2(v)) => Some(*v),
                _ => None,
            })
            .unwrap_or(default)
    };
    Some(ViewRect {
        anchor_min: get("anchor_min", [0.0, 0.0]),
        anchor_max: get("anchor_max", [1.0, 1.0]),
        pivot: get("pivot", [0.5, 0.5]),
        position: get("position", [0.0, 0.0]),
        size: get("size", [0.0, 0.0]),
    })
}

fn widget(expr: &Expr, base: &str) -> Option<ViewWidget> {
    // Builder calls on top of the constructor: `.selected(i)`, `.dead_zone(f)`, `.with_mouse()`.
    let mut calls: Vec<&ExprMethodCall> = Vec::new();
    let mut inner = expr;
    while let Expr::MethodCall(call) = inner {
        calls.push(call);
        inner = &call.receiver;
    }
    let Expr::Call(call) = inner else {
        return None;
    };
    let called = |name: &str| {
        calls
            .iter()
            .find(|c| c.method == name)
            .and_then(|c| c.args.first())
            .and_then(number)
    };
    match base {
        "Toggle" => match call.args.first()? {
            Expr::Lit(ExprLit {
                lit: Lit::Bool(b), ..
            }) => Some(ViewWidget::Toggle(b.value)),
            _ => None,
        },
        "Slider" => {
            let n: Vec<f32> = call.args.iter().filter_map(number).collect();
            let [min, max, value] = n.as_slice() else {
                return None;
            };
            Some(ViewWidget::Slider {
                min: *min,
                max: *max,
                value: *value,
            })
        }
        "Dropdown" => {
            let options = match call.args.first()? {
                Expr::Array(array) => array.elems.iter().filter_map(string).collect(),
                _ => return None,
            };
            Some(ViewWidget::Dropdown {
                options,
                selected: called("selected").map_or(0, |n| n as usize),
            })
        }
        "VirtualJoystick" => Some(ViewWidget::Joystick {
            radius: call.args.first().and_then(number).unwrap_or(80.0),
            dead_zone: called("dead_zone").unwrap_or(0.1),
        }),
        "TouchPad" => Some(ViewWidget::TouchPad),
        _ => None,
    }
}

/// The canvas a `Canvas` component describes.
pub fn canvas_of(component: &ComponentDoc) -> Option<ViewCanvas> {
    component_expr(component, &[]).and_then(|e| canvas(&e))
}

/// The source that makes `canvas` (what the Properties panel writes).
pub fn canvas_source(canvas: &ViewCanvas) -> String {
    let mut out = match canvas.mode {
        ViewCanvasMode::Overlay => "Canvas::overlay()".to_string(),
        ViewCanvasMode::Camera => "Canvas::camera()".to_string(),
        ViewCanvasMode::World {
            size,
            pixels_per_unit,
        } => format!(
            "Canvas::world({}, {}, {:?})",
            size[0], size[1], pixels_per_unit
        ),
    };
    if let Some(name) = &canvas.camera {
        out.push_str(&format!(".camera_named({name:?})"));
    }
    if canvas.sort_order != 0 {
        out.push_str(&format!(".sorted({})", canvas.sort_order));
    }
    if let (Some([w, h]), true) = (
        canvas.reference,
        !matches!(canvas.mode, ViewCanvasMode::World { .. }),
    ) {
        out.push_str(&format!(".reference({w}, {h})"));
    }
    out
}

/// The rect a `RectTransform` component holds (a call or a struct literal).
pub fn rect_of(component: &ComponentDoc) -> Option<ViewRect> {
    component_expr(component, &[])
        .and_then(|e| rect(&e))
        .or_else(|| rect_struct(component))
}

/// A `RectTransform` after dragging a Rect-tool handle by `delta` canvas pixels (y up).
/// `handle` is the side on each axis (-1 left / bottom, 1 right / top, 0 the body: the body
/// handle `[0, 0]` moves). `size_px` is the element's current size, used when the rect asks
/// for its content size (0 on an axis with equal anchors).
pub fn drag_rect(rect: ViewRect, handle: [i8; 2], delta: [f32; 2], size_px: [f32; 2]) -> ViewRect {
    let mut r = rect;
    let moving = handle == [0, 0];
    for axis in 0..2 {
        let stretched = r.anchor_min[axis] != r.anchor_max[axis];
        // Auto size becomes the size it has now, so resizing starts from what is on screen.
        if !stretched && r.size[axis] <= 0.0 && handle[axis] != 0 {
            r.size[axis] = size_px[axis];
        }
        let d = delta[axis];
        let p = r.pivot[axis];
        match handle[axis] {
            // The low edge (left / bottom) moves: the high edge stays.
            -1 => {
                r.size[axis] -= d;
                r.position[axis] += d * (1.0 - p);
            }
            1 => {
                r.size[axis] += d;
                r.position[axis] += d * p;
            }
            _ if moving => r.position[axis] += d,
            _ => {}
        }
        if !stretched {
            r.size[axis] = r.size[axis].max(1.0);
        }
    }
    r
}

fn rounded(n: f32) -> f32 {
    (n * 1000.0).round() / 1000.0
}

/// The rotation and scale a `UiTransform { rotation: Rot2::degrees(..), scale: Vec2::new(..), .. }`
/// component says.
pub fn xform_of(component: &ComponentDoc) -> Option<ViewXform> {
    let Value::Struct { fields, .. } = &component.value else {
        return None;
    };
    let mut x = ViewXform::default();
    for (name, field) in fields {
        match (name.as_str(), field) {
            ("rotation", Field::Expr(src)) => {
                if let Some(Expr::Call(call)) = syn::parse_str::<Expr>(src).ok() {
                    let n = call.args.first().and_then(number);
                    match (path_text(&call.func).as_deref(), n) {
                        (Some("Rot2::degrees"), Some(n)) => x.rotation = n,
                        (Some("Rot2::radians"), Some(n)) => x.rotation = n.to_degrees(),
                        _ => {}
                    }
                }
            }
            ("scale", Field::Vec2(v)) => x.scale = *v,
            ("scale", Field::Expr(src)) => {
                if let Some(Expr::Call(call)) = syn::parse_str::<Expr>(src).ok() {
                    if path_text(&call.func).as_deref() == Some("Vec2::splat") {
                        if let Some(n) = call.args.first().and_then(number) {
                            x.scale = [n, n];
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Some(x)
}

/// The `UiTransform` component that says `x`.
pub fn xform_component(x: ViewXform) -> ComponentDoc {
    ComponentDoc::new(
        "UiTransform",
        Value::Struct {
            fields: vec![
                (
                    "rotation".into(),
                    Field::Expr(format!("Rot2::degrees({:?})", rounded(x.rotation))),
                ),
                (
                    "scale".into(),
                    Field::Vec2([rounded(x.scale[0]), rounded(x.scale[1])]),
                ),
            ],
            rest: true,
        },
    )
}

/// `Color::srgb(..)` / `Color::srgba(..)` for a color.
pub fn color_source([r, g, b, a]: [f32; 4]) -> String {
    let (r, g, b, a) = (rounded(r), rounded(g), rounded(b), rounded(a));
    if a >= 1.0 {
        format!("Color::srgb({r:?}, {g:?}, {b:?})")
    } else {
        format!("Color::srgba({r:?}, {g:?}, {b:?}, {a:?})")
    }
}

/// The color of a `BackgroundColor(..)` / `TextColor(..)` / `BorderColor::all(..)` component,
/// and the file constant it names (`BackgroundColor(PANEL)`), if it does.
pub fn component_color(
    component: &ComponentDoc,
    consts: &[(String, String)],
) -> Option<([f32; 4], Option<String>)> {
    let Expr::Call(call) = component_expr(component, &[])? else {
        return None;
    };
    let arg = call.args.first()?;
    let name = match arg {
        Expr::Path(p) => p.path.get_ident().map(|i| i.to_string()),
        _ => None,
    };
    let c = color(arg, consts)?;
    Some((c, name))
}

/// `#RRGGBB` (and `AA` when translucent).
pub fn color_hex([r, g, b, a]: [f32; 4]) -> String {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    if a >= 1.0 {
        format!("#{:02X}{:02X}{:02X}", byte(r), byte(g), byte(b))
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", byte(r), byte(g), byte(b), byte(a))
    }
}

/// `#RRGGBB`, `RRGGBB`, `#RRGGBBAA`.
pub fn parse_hex(text: &str) -> Option<[f32; 4]> {
    let t = text.trim().trim_start_matches('#');
    if !(t.len() == 6 || t.len() == 8) || !t.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&t[i..i + 2], 16).ok().map(|v| v as f32 / 255.0);
    Some([
        byte(0)?,
        byte(2)?,
        byte(4)?,
        if t.len() == 8 { byte(6)? } else { 1.0 },
    ])
}

/// The string of a `Text::new("..")`.
pub fn text_of(component: &ComponentDoc) -> Option<String> {
    component_expr(component, &[]).and_then(|e| first_string(&e))
}

/// The size of a `TextFont::from_font_size(..)` (or `TextFont { font_size, .. }`).
pub fn text_size_of(component: &ComponentDoc) -> Option<f32> {
    font_size(component, component_expr(component, &[]))
}

/// A bevy_ui `Val` as scenes write it (`px(10)`, `percent(50)`, `auto()`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewVal {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

/// `px(10)`, `percent(50)`, `vw(..)`, `vh(..)`, `auto()`, `Val::Px(10.0)`, `Val::Auto`.
pub fn parse_val(source: &str) -> Option<ViewVal> {
    let expr: Expr = syn::parse_str(source).ok()?;
    if let Some(path) = path_text(&expr) {
        return match path.rsplit("::").next()? {
            "Auto" => Some(ViewVal::Auto),
            "ZERO" => Some(ViewVal::Px(0.0)),
            _ => None,
        };
    }
    let Expr::Call(call) = &expr else {
        return None;
    };
    let name = path_text(&call.func)?;
    let n = call.args.first().and_then(number);
    match (name.rsplit("::").next()?, n) {
        ("px" | "Px", Some(n)) => Some(ViewVal::Px(n)),
        ("percent" | "Percent", Some(n)) => Some(ViewVal::Percent(n)),
        ("vw" | "Vw", Some(n)) => Some(ViewVal::Vw(n)),
        ("vh" | "Vh", Some(n)) => Some(ViewVal::Vh(n)),
        ("auto", None) => Some(ViewVal::Auto),
        _ => None,
    }
}

/// The source that reads back as `val`.
pub fn val_source(val: ViewVal) -> String {
    let n = |n: f32| {
        if n.fract() == 0.0 && n.abs() < 1.0e6 {
            format!("{}", n as i64)
        } else {
            format!("{:?}", (n * 1000.0).round() / 1000.0)
        }
    };
    match val {
        ViewVal::Auto => "auto()".into(),
        ViewVal::Px(v) => format!("px({})", n(v)),
        ViewVal::Percent(v) => format!("percent({})", n(v)),
        ViewVal::Vw(v) => format!("vw({})", n(v)),
        ViewVal::Vh(v) => format!("vh({})", n(v)),
    }
}

/// `UiRect::all(px(8))`, `axes(h, v)`, `horizontal(v)`, `vertical(v)`, `left/right/top/bottom(v)`
/// or `new(l, r, t, b)`, as pixels: left, right, top, bottom. `None` when a side is not in px.
pub fn parse_ui_rect(source: &str) -> Option<[f32; 4]> {
    let expr: Expr = syn::parse_str(source).ok()?;
    let Expr::Call(call) = &expr else {
        return None;
    };
    let name = path_text(&call.func)?;
    let px = |i: usize| {
        let arg = quote::ToTokens::to_token_stream(call.args.get(i)?).to_string();
        match parse_val(&arg)? {
            ViewVal::Px(v) => Some(v),
            _ => None,
        }
    };
    Some(match name.rsplit("::").next()? {
        "all" => [px(0)?; 4],
        "axes" => [px(0)?, px(0)?, px(1)?, px(1)?],
        "horizontal" => [px(0)?, px(0)?, 0.0, 0.0],
        "vertical" => [0.0, 0.0, px(0)?, px(0)?],
        "left" => [px(0)?, 0.0, 0.0, 0.0],
        "right" => [0.0, px(0)?, 0.0, 0.0],
        "top" => [0.0, 0.0, px(0)?, 0.0],
        "bottom" => [0.0, 0.0, 0.0, px(0)?],
        "new" => [px(0)?, px(1)?, px(2)?, px(3)?],
        _ => return None,
    })
}

/// The shortest `UiRect` call for `[left, right, top, bottom]` pixels.
pub fn ui_rect_source([l, r, t, b]: [f32; 4]) -> String {
    let v = |n: f32| val_source(ViewVal::Px(n));
    if l == r && r == t && t == b {
        format!("UiRect::all({})", v(l))
    } else if l == r && t == b {
        format!("UiRect::axes({}, {})", v(l), v(t))
    } else {
        format!("UiRect::new({}, {}, {}, {})", v(l), v(r), v(t), v(b))
    }
}

/// `BorderRadius::all(px(6))`: the radius in pixels.
pub fn parse_radius(source: &str) -> Option<f32> {
    let expr: Expr = syn::parse_str(source).ok()?;
    let Expr::Call(call) = &expr else {
        return None;
    };
    if path_text(&call.func)?.rsplit("::").next()? != "all" {
        return None;
    }
    let arg = quote::ToTokens::to_token_stream(call.args.first()?).to_string();
    match parse_val(&arg)? {
        ViewVal::Px(v) => Some(v),
        _ => None,
    }
}

pub fn radius_source(px: f32) -> String {
    format!("BorderRadius::all({})", val_source(ViewVal::Px(px)))
}

/// What a flexbox element (a `Node` without `RectTransform`) says about where it sits: whether
/// it is out of the flow (`PositionType::Absolute`) and its inset values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeFrame {
    pub absolute: bool,
    pub left: ViewVal,
    pub top: ViewVal,
    pub right: ViewVal,
    pub bottom: ViewVal,
}

impl Default for NodeFrame {
    fn default() -> Self {
        Self {
            absolute: false,
            left: ViewVal::Auto,
            top: ViewVal::Auto,
            right: ViewVal::Auto,
            bottom: ViewVal::Auto,
        }
    }
}

/// The `Node` values a Rect-tool drag of a flexbox element sets. `None` leaves the field as it
/// is. Positions are pixels (y down, as bevy_ui's `left` / `top`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NodeEdit {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    /// An out-of-flow element is pinned by `left` / `top` and its size: `right` / `bottom`
    /// would fight them, so they go back to `auto`.
    pub clear_right: bool,
    pub clear_bottom: bool,
}

/// A flexbox element after dragging a Rect-tool handle by `delta` canvas pixels (y up).
///
/// In the flow, a move is an offset from where the layout puts the element (`left` / `top`
/// with `PositionType::Relative`, which the layout keeps in the flow) and a resize sets
/// `width` / `height`; dragging the left or top edge also shifts the offset so the opposite edge
/// stays. Out of the flow (`Absolute`), the element is pinned from the parent's corner:
/// `offset` is where it is now, relative to the parent's top-left (canvas pixels, y down).
pub fn drag_node(
    frame: &NodeFrame,
    handle: [i8; 2],
    delta: [f32; 2],
    size: [f32; 2],
    offset: [f32; 2],
) -> NodeEdit {
    let moving = handle == [0, 0];
    let mut edit = NodeEdit::default();
    let mut left_top = [0.0f32; 2];
    let mut new_size = size;
    let mut moved = [false; 2];
    let mut resized = [false; 2];
    for axis in 0..2 {
        // Toward the end (right / down): bevy_ui's y grows downward, the drag's grows up.
        let d = if axis == 0 { delta[0] } else { -delta[1] };
        let side = if axis == 0 { handle[0] } else { -handle[1] };
        let authored = match (axis, frame.left, frame.top) {
            (0, ViewVal::Px(v), _) | (1, _, ViewVal::Px(v)) => Some(v),
            _ => None,
        };
        let start = match (frame.absolute, authored) {
            (_, Some(v)) => v,
            (true, None) => offset[axis],
            (false, None) => 0.0,
        };
        let mut at = start;
        if moving {
            at += d;
            moved[axis] = d != 0.0;
        }
        match side {
            -1 => {
                new_size[axis] = (new_size[axis] - d).max(1.0);
                at += d;
                moved[axis] = d != 0.0;
                resized[axis] = true;
            }
            1 => {
                new_size[axis] = (new_size[axis] + d).max(1.0);
                resized[axis] = true;
            }
            _ => {}
        }
        left_top[axis] = at;
    }
    if frame.absolute {
        edit.left = Some(left_top[0]);
        edit.top = Some(left_top[1]);
        edit.width = Some(new_size[0]);
        edit.height = Some(new_size[1]);
        edit.clear_right = true;
        edit.clear_bottom = true;
    } else {
        edit.left = moved[0].then_some(left_top[0]);
        edit.top = moved[1].then_some(left_top[1]);
        edit.width = resized[0].then_some(new_size[0]);
        edit.height = resized[1].then_some(new_size[1]);
    }
    edit
}

/// A sprite's centre and size after dragging a handle by `delta` world units.
pub fn drag_sprite(
    center: [f32; 2],
    size: [f32; 2],
    handle: [i8; 2],
    delta: [f32; 2],
) -> ([f32; 2], [f32; 2]) {
    let (mut c, mut s) = (center, size);
    for axis in 0..2 {
        match handle[axis] {
            -1 => {
                s[axis] = (s[axis] - delta[axis]).max(1.0);
                c[axis] += (size[axis] - s[axis]) * 0.5;
            }
            1 => {
                s[axis] = (s[axis] + delta[axis]).max(1.0);
                c[axis] += (s[axis] - size[axis]) * 0.5;
            }
            _ if handle == [0, 0] => c[axis] += delta[axis],
            _ => {}
        }
    }
    (c, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn consts() -> Vec<(String, String)> {
        vec![("PANEL".into(), "Color::srgb(0.1, 0.2, 0.3)".into())]
    }

    #[test]
    fn colors_read_literals_and_constants() {
        let c = consts();
        assert_eq!(
            color(
                &syn::parse_str("Color::srgba(1.0, 0.0, 0.5, 0.25)").unwrap(),
                &c
            ),
            Some([1.0, 0.0, 0.5, 0.25])
        );
        assert_eq!(
            color(&syn::parse_str("PANEL").unwrap(), &c),
            Some([0.1, 0.2, 0.3, 1.0])
        );
        assert_eq!(color(&syn::parse_str("something()").unwrap(), &c), None);
    }

    #[test]
    fn canvas_reads_mode_order_and_reference() {
        let e: Expr = syn::parse_str("Canvas::world(400, 300, 100.0).sorted(2)").unwrap();
        let read = canvas(&e).unwrap();
        assert_eq!(
            read.mode,
            ViewCanvasMode::World {
                size: [400, 300],
                pixels_per_unit: 100.0
            }
        );
        assert_eq!(read.sort_order, 2);
        let e: Expr = syn::parse_str("Canvas::camera().reference(1920, 1080)").unwrap();
        let read = canvas(&e).unwrap();
        assert_eq!(read.mode, ViewCanvasMode::Camera);
        assert_eq!(read.reference, Some([1920, 1080]));
    }

    fn rect(anchor: [f32; 2], pivot: [f32; 2], position: [f32; 2], size: [f32; 2]) -> ViewRect {
        ViewRect {
            anchor_min: anchor,
            anchor_max: anchor,
            pivot,
            position,
            size,
        }
    }

    #[test]
    fn dragging_a_ui_rect_moves_and_resizes_it_around_its_fixed_edge() {
        let r = rect([0.0, 1.0], [0.0, 1.0], [16.0, -16.0], [200.0, 100.0]);
        // The body moves by the delta.
        let moved = drag_rect(r, [0, 0], [5.0, -3.0], [200.0, 100.0]);
        assert_eq!(
            (moved.position, moved.size),
            ([21.0, -19.0], [200.0, 100.0])
        );
        // The right edge grows it to the right: the left edge (pivot) stays.
        let wider = drag_rect(r, [1, 0], [40.0, 0.0], [200.0, 100.0]);
        assert_eq!(
            (wider.position, wider.size),
            ([16.0, -16.0], [240.0, 100.0])
        );
        // The left edge moves right: smaller, and the position follows the left edge.
        let narrower = drag_rect(r, [-1, 0], [30.0, 0.0], [200.0, 100.0]);
        assert_eq!((narrower.position[0], narrower.size[0]), (46.0, 170.0));
        // Auto size starts from the size it has on screen.
        let auto = rect([0.5, 0.5], [0.5, 0.5], [0.0, 0.0], [0.0, 0.0]);
        let grown = drag_rect(auto, [1, 0], [10.0, 0.0], [80.0, 30.0]);
        assert_eq!(grown.size[0], 90.0);
        assert_eq!(grown.position[0], 5.0, "a centred pivot shifts by half");
    }

    #[test]
    fn flexbox_elements_read_their_border_and_order() {
        let c = consts();
        let doc = |ty: &str, src: &str| {
            crate::editor::project::codec::ComponentDoc::new(
                ty,
                crate::editor::project::codec::Value::Expr(src.into()),
            )
        };
        let border = doc("BorderColor::all", "BorderColor::all(PANEL)");
        let z = doc("ZIndex", "ZIndex(3)");
        let ui = extract(&[&border, &z], &c).unwrap();
        assert_eq!(ui.border_color, Some([0.1, 0.2, 0.3, 1.0]));
        assert_eq!(ui.z_index, Some(3));
    }

    #[test]
    fn vals_round_trip_through_their_source() {
        for (src, val) in [
            ("px(58)", ViewVal::Px(58.0)),
            ("px(2.5)", ViewVal::Px(2.5)),
            ("percent(100)", ViewVal::Percent(100.0)),
            ("auto()", ViewVal::Auto),
            ("vw(50)", ViewVal::Vw(50.0)),
        ] {
            assert_eq!(parse_val(src), Some(val));
            assert_eq!(val_source(val), src);
        }
        assert_eq!(parse_val("Val::Auto"), Some(ViewVal::Auto));
        assert_eq!(parse_val("Val::Px(4.0)"), Some(ViewVal::Px(4.0)));
        assert_eq!(parse_val("something_else()"), None);
    }

    #[test]
    fn a_flexbox_element_in_the_flow_moves_by_offset_and_resizes_by_size() {
        let frame = NodeFrame::default();
        // Moving: an offset from the layout position, the size untouched (y flips: up is -top).
        let moved = drag_node(&frame, [0, 0], [10.0, 4.0], [196.0, 58.0], [0.0, 0.0]);
        assert_eq!(
            moved,
            NodeEdit {
                left: Some(10.0),
                top: Some(-4.0),
                ..default_edit()
            }
        );
        // The right edge grows the width only.
        let wider = drag_node(&frame, [1, 0], [20.0, 0.0], [196.0, 58.0], [0.0, 0.0]);
        assert_eq!(
            wider,
            NodeEdit {
                width: Some(216.0),
                ..default_edit()
            }
        );
        // The top edge dragged up grows the height and lifts the element: the bottom stays.
        let taller = drag_node(&frame, [0, 1], [0.0, 12.0], [196.0, 58.0], [0.0, 0.0]);
        assert_eq!(
            taller,
            NodeEdit {
                top: Some(-12.0),
                height: Some(70.0),
                ..default_edit()
            }
        );
        // An authored offset is the start.
        let frame = NodeFrame {
            left: ViewVal::Px(6.0),
            ..NodeFrame::default()
        };
        let moved = drag_node(&frame, [0, 0], [4.0, 0.0], [10.0, 10.0], [0.0, 0.0]);
        assert_eq!(moved.left, Some(10.0));
    }

    #[test]
    fn an_absolute_element_is_pinned_from_the_parent_corner() {
        let frame = NodeFrame {
            absolute: true,
            right: ViewVal::Px(8.0),
            ..NodeFrame::default()
        };
        // It sat 100 px from the parent's left edge and 20 from its top; move it 5 right.
        let edit = drag_node(&frame, [0, 0], [5.0, 0.0], [80.0, 30.0], [100.0, 20.0]);
        assert_eq!(
            edit,
            NodeEdit {
                left: Some(105.0),
                top: Some(20.0),
                width: Some(80.0),
                height: Some(30.0),
                clear_right: true,
                clear_bottom: true,
            }
        );
    }

    #[test]
    fn ui_rects_and_radii_round_trip() {
        assert_eq!(parse_ui_rect("UiRect::all(px(16))"), Some([16.0; 4]));
        assert_eq!(
            parse_ui_rect("UiRect::axes(px(10), px(6))"),
            Some([10.0, 10.0, 6.0, 6.0])
        );
        assert_eq!(parse_ui_rect("UiRect::all(percent(5))"), None);
        for r in [[4.0; 4], [10.0, 10.0, 6.0, 6.0], [1.0, 2.0, 3.0, 4.0]] {
            assert_eq!(parse_ui_rect(&ui_rect_source(r)), Some(r));
        }
        assert_eq!(parse_radius("BorderRadius::all(px(6))"), Some(6.0));
        assert_eq!(parse_radius(&radius_source(2.5)), Some(2.5));
    }

    fn default_edit() -> NodeEdit {
        NodeEdit::default()
    }

    #[test]
    fn colors_texts_and_transforms_round_trip() {
        let c = consts();
        let doc = |ty: &str, src: &str| {
            crate::editor::project::codec::ComponentDoc::new(
                ty,
                crate::editor::project::codec::Value::Expr(src.into()),
            )
        };
        let bg = doc("BackgroundColor", "BackgroundColor(PANEL)");
        assert_eq!(
            component_color(&bg, &c),
            Some(([0.1, 0.2, 0.3, 1.0], Some("PANEL".into())))
        );
        let lit = doc("TextColor", "TextColor(Color::srgb(1.0, 0.5, 0.0))");
        assert_eq!(
            component_color(&lit, &c),
            Some(([1.0, 0.5, 0.0, 1.0], None))
        );
        assert_eq!(color_source([1.0, 0.5, 0.0, 1.0]), "Color::srgb(1.0, 0.5, 0.0)");
        assert_eq!(parse_hex("#FF8000"), Some([1.0, 128.0 / 255.0, 0.0, 1.0]));
        assert_eq!(color_hex([1.0, 0.0, 0.0, 1.0]), "#FF0000");
        assert_eq!(parse_hex("nope"), None);
        assert_eq!(
            text_of(&doc("Text", "Text::new(\"Darius\")")),
            Some("Darius".into())
        );
        assert_eq!(
            text_size_of(&doc("TextFont", "TextFont::from_font_size(34.0)")),
            Some(34.0)
        );
        let x = ViewXform {
            rotation: 30.0,
            scale: [2.0, 1.5],
        };
        assert_eq!(xform_of(&xform_component(x)), Some(x));
    }

    #[test]
    fn dragging_a_sprite_handle_keeps_the_opposite_edge() {
        let (c, s) = drag_sprite([0.0, 0.0], [100.0, 50.0], [1, 0], [20.0, 0.0]);
        assert_eq!((c, s), ([10.0, 0.0], [120.0, 50.0]));
        let (c, s) = drag_sprite([0.0, 0.0], [100.0, 50.0], [-1, 1], [-20.0, 10.0]);
        assert_eq!((c, s), ([-10.0, 5.0], [120.0, 60.0]));
        let (c, s) = drag_sprite([1.0, 2.0], [10.0, 10.0], [0, 0], [3.0, 4.0]);
        assert_eq!((c, s), ([4.0, 6.0], [10.0, 10.0]));
    }

    #[test]
    fn a_canvas_is_written_back_as_the_source_that_reads_as_it() {
        for src in [
            "Canvas::overlay()",
            "Canvas::camera().camera_named(\"Main Camera\").sorted(3)",
            "Canvas::world(800, 500, 1.0)",
            "Canvas::overlay().reference(1920, 1080)",
        ] {
            let e: Expr = syn::parse_str(src).unwrap();
            let read = canvas(&e).unwrap();
            assert_eq!(canvas_source(&read), src);
        }
    }

    #[test]
    fn touch_inputs_are_read() {
        let e: Expr =
            syn::parse_str("VirtualJoystick::new(80.0).dead_zone(0.2).with_mouse()").unwrap();
        assert_eq!(
            widget(&e, "VirtualJoystick"),
            Some(ViewWidget::Joystick {
                radius: 80.0,
                dead_zone: 0.2
            })
        );
        let e: Expr = syn::parse_str("TouchPad::new()").unwrap();
        assert_eq!(widget(&e, "TouchPad"), Some(ViewWidget::TouchPad));
    }

    #[test]
    fn widgets_read_their_data() {
        let e: Expr = syn::parse_str(r#"Dropdown::new(["Low", "High"]).selected(1)"#).unwrap();
        assert_eq!(
            widget(&e, "Dropdown"),
            Some(ViewWidget::Dropdown {
                options: vec!["Low".into(), "High".into()],
                selected: 1
            })
        );
        let e: Expr = syn::parse_str("Slider::new(0.0, 10.0, 2.5)").unwrap();
        assert_eq!(
            widget(&e, "Slider"),
            Some(ViewWidget::Slider {
                min: 0.0,
                max: 10.0,
                value: 2.5
            })
        );
    }
}
