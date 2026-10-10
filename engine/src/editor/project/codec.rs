//! Reading the `.rs` assets into editable documents (and, in `write`, back). The files are
//! typed data literals (__docs__/engine.md); the editor understands the subset it writes:
//!
//! ```ignore
//! Object::new("Crate")                       // or Object::prefab(path) [.named("..")]
//!     .with(Transform::from_xyz(3.0, 0.5, -2.0))
//!     .with(MeshDef::Cube { size: 1.0 })
//!     .with(CameraTarget3d)
//!     .child(Object::new("Lid"))
//! ```
//!
//! Anything else (a call the editor cannot evaluate, a closure, a loop) is kept as source
//! text: shown read-only, written back untouched, never lost.

use quote::ToTokens;

pub use crate::modules::assets::Dimension;
use syn::punctuated::Punctuated;
use syn::{
    Expr, ExprCall, ExprLit, ExprMethodCall, ExprPath, ExprStruct, ExprUnary, Lit, Token, UnOp,
};

/// A scene: 2D or 3D, and its objects in file order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SceneDoc {
    pub dimension: Dimension,
    pub objects: Vec<ObjectDoc>,
}

/// One object. `id`s are assigned in document order when parsed; they identify objects in
/// the editor session only (never written).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObjectDoc {
    pub id: u32,
    /// Empty on a prefab instance that keeps the prefab's name.
    pub name: String,
    /// The prefab function's path as written (`crate::assets::prefabs::player_prefab::prefab`).
    pub prefab: Option<String>,
    pub components: Vec<ComponentDoc>,
    pub children: Vec<ObjectDoc>,
}

/// A component: its type (`Transform`, `MeshDef::Cube`, `CharacterController3d`) and value.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentDoc {
    pub ty: String,
    pub value: Value,
    /// The source as read. Written back as is until the component is edited (`None` then),
    /// so saving only changes what the user changed.
    pub original: Option<String>,
}

impl ComponentDoc {
    pub fn new(ty: impl Into<String>, value: Value) -> Self {
        Self {
            ty: ty.into(),
            value,
            original: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `Path { field: value, .. }` (`rest` when it ends in `..default()`).
    Struct {
        fields: Vec<(String, Field)>,
        rest: bool,
    },
    /// A unit struct or variant, or `Type::default()`: the exact source (`CameraTarget3d`,
    /// `Camera3d::default()`).
    Unit(String),
    Transform(TransformData),
    /// Source the editor does not edit.
    Expr(String),
}

/// A field value the editor can edit; anything else is `Expr`.
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    /// A float literal (`6.0`).
    Num(f32),
    /// An integer literal (`3`): written back as an integer, so integer fields still compile.
    Int(i64),
    Bool(bool),
    Str(String),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    /// sRGB(A), 0..1.
    Color([f32; 4]),
    /// `Some("...")` / `None`.
    OptStr(Option<String>),
    /// An enum path (`Filter::Nearest`).
    Path(String),
    Expr(String),
}

/// Position, rotation (degrees about X, Y, Z; applied in Bevy's YXZ order: yaw, pitch, roll)
/// and scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformData {
    pub translation: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl Default for TransformData {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation: [0.0; 3],
            scale: [1.0; 3],
        }
    }
}

impl ObjectDoc {
    pub fn component(&self, ty: &str) -> Option<&ComponentDoc> {
        self.components
            .iter()
            .find(|c| c.ty == ty || c.ty.starts_with(&format!("{ty}::")))
    }

    pub fn transform(&self) -> Option<TransformData> {
        match self.component("Transform")?.value {
            Value::Transform(t) => Some(t),
            _ => None,
        }
    }

    /// Every object in this tree, depth first.
    pub fn walk<'a>(&'a self, out: &mut Vec<&'a ObjectDoc>) {
        out.push(self);
        for child in &self.children {
            child.walk(out);
        }
    }
}

impl SceneDoc {
    pub fn all(&self) -> Vec<&ObjectDoc> {
        let mut out = Vec::new();
        for object in &self.objects {
            object.walk(&mut out);
        }
        out
    }

    pub fn find(&self, id: u32) -> Option<&ObjectDoc> {
        self.all().into_iter().find(|o| o.id == id)
    }
}

/// Source text of a syntax node (for read-only display and for write-back).
pub fn source(node: &impl ToTokens) -> String {
    node.to_token_stream().to_string()
}

/// Parses a `*.scene.rs`: the `SceneAsset { dimension, objects: vec![...] }` its function
/// returns (a scene without `dimension` is 3D).
pub fn parse_scene(src: &str) -> Result<SceneDoc, String> {
    let body = returned_struct(src, "SceneAsset")?;
    let objects = struct_field(&body, "objects").ok_or("SceneAsset has no `objects`")?;
    // A scene without `dimension` is 3D (the default).
    let dimension = match struct_field(&body, "dimension").map(|e| source(e).replace(' ', "")) {
        Some(d) if d.ends_with("D2") => Dimension::D2,
        Some(d) if d.ends_with("D3") => Dimension::D3,
        Some(d) => {
            return Err(format!(
                "unknown dimension `{d}` (Dimension::D2 or Dimension::D3)"
            ));
        }
        None => Dimension::D3,
    };
    let mut next = 0;
    let objects = vec_items(objects)?
        .iter()
        .map(|e| parse_object(e, &mut next))
        .collect::<Result<_, _>>()?;
    Ok(SceneDoc { dimension, objects })
}

/// Parses a `*.prefab.rs`: the root object of the `PrefabAsset { root: ... }` it returns.
pub fn parse_prefab(src: &str) -> Result<ObjectDoc, String> {
    let body = returned_struct(src, "PrefabAsset")?;
    let root = struct_field(&body, "root").ok_or("PrefabAsset has no `root`")?;
    parse_object(root, &mut 0)
}

/// The struct literal of type `ty` the file's function returns.
fn returned_struct(src: &str, ty: &str) -> Result<ExprStruct, String> {
    let file = syn::parse_file(src).map_err(|e| format!("not valid Rust: {e}"))?;
    for item in &file.items {
        let syn::Item::Fn(function) = item else {
            continue;
        };
        let returns = match &function.sig.output {
            syn::ReturnType::Type(_, t) => source(t).replace(' ', ""),
            _ => continue,
        };
        if !returns.ends_with(ty) {
            continue;
        }
        if let Some(syn::Stmt::Expr(Expr::Struct(s), None)) = function.block.stmts.last() {
            return Ok(s.clone());
        }
        return Err(format!(
            "`fn {}` must end with a `{ty} {{ .. }}` literal",
            function.sig.ident
        ));
    }
    Err(format!("no function returning `{ty}`"))
}

fn struct_field<'a>(s: &'a ExprStruct, name: &str) -> Option<&'a Expr> {
    s.fields
        .iter()
        .find(|f| source(&f.member) == name)
        .map(|f| &f.expr)
}

/// The items of `vec![a, b, c]`.
fn vec_items(expr: &Expr) -> Result<Vec<Expr>, String> {
    let Expr::Macro(m) = expr else {
        return Err("expected `vec![...]`".into());
    };
    m.mac
        .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
        .map(|items| items.into_iter().collect())
        .map_err(|e| format!("in vec![]: {e}"))
}

fn parse_object(expr: &Expr, next: &mut u32) -> Result<ObjectDoc, String> {
    // Unwind the method chain: base call first, then .with/.child/.named in order.
    let mut calls: Vec<&ExprMethodCall> = Vec::new();
    let mut base = expr;
    while let Expr::MethodCall(call) = base {
        calls.push(call);
        base = &call.receiver;
    }
    calls.reverse();

    let id = *next;
    *next += 1;
    let mut object = ObjectDoc {
        id,
        ..Default::default()
    };
    let Expr::Call(ExprCall { func, args, .. }) = base else {
        return Err(format!(
            "an object starts with Object::new(..) or Object::prefab(..), not `{}`",
            source(base)
        ));
    };
    match source(func).replace(' ', "").as_str() {
        "Object::new" => {
            object.name = string_lit(args.first().ok_or("Object::new needs a name")?)
                .ok_or("Object::new takes a string literal")?
        }
        "Object::prefab" => {
            object.prefab =
                Some(source(args.first().ok_or("Object::prefab needs a prefab")?).replace(' ', ""))
        }
        other => return Err(format!("unknown object constructor `{other}`")),
    }
    for call in calls {
        let arg = call
            .args
            .first()
            .ok_or_else(|| format!(".{}() needs an argument", call.method))?;
        match call.method.to_string().as_str() {
            "with" => object.components.push(parse_component(arg)),
            "child" => object.children.push(parse_object(arg, next)?),
            "named" => object.name = string_lit(arg).ok_or(".named() takes a string literal")?,
            other => return Err(format!("unknown object method `.{other}()`")),
        }
    }
    Ok(object)
}

pub fn parse_component(expr: &Expr) -> ComponentDoc {
    let mut component = read_component(expr);
    component.original = Some(source(expr));
    component
}

fn read_component(expr: &Expr) -> ComponentDoc {
    if let Some(transform) = transform_of(expr) {
        return ComponentDoc::new("Transform", Value::Transform(transform));
    }
    match expr {
        Expr::Struct(s) => ComponentDoc::new(
            source(&s.path).replace(' ', ""),
            Value::Struct {
                fields: s
                    .fields
                    .iter()
                    .map(|f| (source(&f.member), parse_field(&f.expr)))
                    .collect(),
                rest: s.rest.is_some(),
            },
        ),
        Expr::Path(p) => {
            let path = source(&p.path).replace(' ', "");
            ComponentDoc::new(path.clone(), Value::Unit(path))
        }
        Expr::Call(ExprCall { func, args, .. })
            if args.is_empty() && source(func).replace(' ', "").ends_with("::default") =>
        {
            let path = source(func).replace(' ', "");
            ComponentDoc::new(
                path.trim_end_matches("::default"),
                Value::Unit(format!("{path}()")),
            )
        }
        _ => ComponentDoc::new(type_of_expr(expr), Value::Expr(source(expr))),
    }
}

/// The type an expression most likely builds (`Foo::bar(..)` → `Foo`), for display.
fn type_of_expr(expr: &Expr) -> String {
    match expr {
        Expr::Call(c) => {
            let path = source(&c.func).replace(' ', "");
            path.rsplit_once("::")
                .map(|(t, _)| t.to_string())
                .unwrap_or(path)
        }
        Expr::MethodCall(m) => type_of_expr(&m.receiver),
        _ => "?".into(),
    }
}

pub fn parse_field(expr: &Expr) -> Field {
    if let Some(i) = integer(expr) {
        return Field::Int(i);
    }
    if let Some(n) = number(expr) {
        return Field::Num(n);
    }
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Bool(b), ..
        }) => Field::Bool(b.value),
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => Field::Str(s.value()),
        Expr::Path(p) if source(p).replace(' ', "") == "None" => Field::OptStr(None),
        Expr::Call(c) if source(&c.func) == "Some" => match c.args.first().and_then(string_lit) {
            Some(s) => Field::OptStr(Some(s)),
            None => Field::Expr(source(expr)),
        },
        Expr::Call(c) => {
            let func = source(&c.func).replace(' ', "");
            let nums: Option<Vec<f32>> = c.args.iter().map(number).collect();
            match (func.as_str(), nums.as_deref()) {
                ("Vec3::new", Some([x, y, z])) => Field::Vec3([*x, *y, *z]),
                ("Vec2::new", Some([x, y])) => Field::Vec2([*x, *y]),
                ("Color::srgb", Some([r, g, b])) => Field::Color([*r, *g, *b, 1.0]),
                ("Color::srgba", Some([r, g, b, a])) => Field::Color([*r, *g, *b, *a]),
                _ => Field::Expr(source(expr)),
            }
        }
        Expr::Path(p) => match source(p).replace(' ', "").as_str() {
            "Color::WHITE" => Field::Color([1.0, 1.0, 1.0, 1.0]),
            "Color::BLACK" => Field::Color([0.0, 0.0, 0.0, 1.0]),
            "Vec3::ZERO" => Field::Vec3([0.0; 3]),
            "Vec3::ONE" => Field::Vec3([1.0; 3]),
            path => Field::Path(path.to_string()),
        },
        _ => Field::Expr(source(expr)),
    }
}

fn number(expr: &Expr) -> Option<f32> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Float(f), ..
        }) => f.base10_parse().ok(),
        Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
        }) => i.base10_parse::<i64>().ok().map(|i| i as f32),
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr,
            ..
        }) => number(expr).map(|n| -n),
        Expr::Paren(p) => number(&p.expr),
        _ => None,
    }
}

fn integer(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
        }) => i.base10_parse().ok(),
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr,
            ..
        }) => integer(expr).map(|n| -n),
        _ => None,
    }
}

/// An angle in degrees: `45.0_f32.to_radians()` (how the editor writes them) or a plain
/// number, which Bevy reads as radians.
fn degrees(expr: &Expr) -> Option<f32> {
    match expr {
        Expr::MethodCall(m) if m.method == "to_radians" && m.args.is_empty() => number(&m.receiver),
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr,
            ..
        }) => degrees(expr).map(|d| -d),
        _ => number(expr).map(f32::to_degrees),
    }
}

fn string_lit(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => Some(s.value()),
        _ => None,
    }
}

/// Evaluates the Transform forms the editor knows: `Transform::from_xyz(..)`,
/// `Transform::default()` / `IDENTITY`, `.looking_at(Vec3, Vec3)`, `.with_scale(Vec3)`,
/// `Transform::from_scale(..)`, and the struct literal the editor writes.
pub fn transform_of(expr: &Expr) -> Option<TransformData> {
    use bevy::math::{EulerRot, Quat, Vec3};
    use bevy::transform::components::Transform;

    fn vec3(expr: &Expr) -> Option<Vec3> {
        match parse_field(expr) {
            Field::Vec3(v) => Some(Vec3::from_array(v)),
            Field::Path(p) if p == "Vec3::Y" => Some(Vec3::Y),
            Field::Path(p) if p == "Vec3::X" => Some(Vec3::X),
            Field::Path(p) if p == "Vec3::Z" => Some(Vec3::Z),
            Field::Num(n) => Some(Vec3::splat(n)),
            _ => None,
        }
    }
    fn eval(expr: &Expr) -> Option<Transform> {
        match expr {
            Expr::Path(ExprPath { path, .. })
                if source(path).replace(' ', "") == "Transform::IDENTITY" =>
            {
                Some(Transform::IDENTITY)
            }
            Expr::Call(call) => {
                let func = source(&call.func).replace(' ', "");
                let args: Vec<&Expr> = call.args.iter().collect();
                match (func.as_str(), args.as_slice()) {
                    ("Transform::default", []) => Some(Transform::IDENTITY),
                    ("Transform::from_xyz", [x, y, z]) => {
                        Some(Transform::from_xyz(number(x)?, number(y)?, number(z)?))
                    }
                    ("Transform::from_translation", [v]) => {
                        Some(Transform::from_translation(vec3(v)?))
                    }
                    ("Transform::from_scale", [v]) => Some(Transform::from_scale(vec3(v)?)),
                    _ => None,
                }
            }
            Expr::MethodCall(call) => {
                let base = eval(&call.receiver)?;
                let args: Vec<&Expr> = call.args.iter().collect();
                match (call.method.to_string().as_str(), args.as_slice()) {
                    ("looking_at", [target, up]) => Some(base.looking_at(vec3(target)?, vec3(up)?)),
                    ("with_scale", [v]) => Some(base.with_scale(vec3(v)?)),
                    _ => None,
                }
            }
            Expr::Struct(s) if source(&s.path).replace(' ', "") == "Transform" => {
                let mut t = Transform::IDENTITY;
                for field in &s.fields {
                    match source(&field.member).as_str() {
                        "translation" => t.translation = vec3(&field.expr)?,
                        "scale" => t.scale = vec3(&field.expr)?,
                        "rotation" => t.rotation = rotation(&field.expr)?,
                        _ => return None,
                    }
                }
                Some(t)
            }
            _ => None,
        }
    }
    fn rotation(expr: &Expr) -> Option<Quat> {
        let Expr::Call(call) = expr else {
            return (source(expr).replace(' ', "") == "Quat::IDENTITY").then_some(Quat::IDENTITY);
        };
        let args: Vec<&Expr> = call.args.iter().collect();
        match (
            source(&call.func).replace(' ', "").as_str(),
            args.as_slice(),
        ) {
            ("Quat::from_euler", [order, a, b, c])
                if source(order).replace(' ', "") == "EulerRot::YXZ" =>
            {
                Some(Quat::from_euler(
                    EulerRot::YXZ,
                    degrees(a)?.to_radians(),
                    degrees(b)?.to_radians(),
                    degrees(c)?.to_radians(),
                ))
            }
            _ => None,
        }
    }

    let t = eval(expr)?;
    let (y, x, z) = t.rotation.to_euler(EulerRot::YXZ);
    let round = |v: f32| (v * 1000.0).round() / 1000.0;
    Some(TransformData {
        translation: t.translation.to_array().map(round),
        rotation: [x, y, z].map(|r| round(r.to_degrees())),
        scale: t.scale.to_array().map(round),
    })
}

// ---- writing ---------------------------------------------------------------------------

/// Writes `doc` into the scene file `src`: the `SceneAsset` literal its function returns is
/// regenerated, the rest of the file (`use` lines, docs, other items) is kept, and the result
/// is formatted. Components the user did not edit are written back as they were read.
pub fn write_scene(src: &str, doc: &SceneDoc) -> Result<String, String> {
    let objects: Vec<String> = doc.objects.iter().map(object_source).collect();
    // prettyplease prints macro bodies as raw tokens (`count : 3`), so the list is formatted
    // as a call to a placeholder function and turned into `vec![...]` afterwards.
    let dimension = match doc.dimension {
        Dimension::D2 => "Dimension::D2",
        Dimension::D3 => "Dimension::D3",
    };
    let text = write_returned(
        src,
        "SceneAsset",
        &format!(
            "SceneAsset {{ dimension: {dimension}, objects: {VEC_MARK}([{}]) }}",
            objects.join(", ")
        ),
    )?;
    Ok(unmark_vec(&text))
}

const VEC_MARK: &str = "__xerxes_vec";

/// `__xerxes_vec([a, b])` → `vec![a, b]`, matching the brackets outside string literals.
fn unmark_vec(text: &str) -> String {
    let open = format!("{VEC_MARK}([");
    let Some(start) = text.find(&open) else {
        return text.to_string();
    };
    let body_start = start + open.len();
    let bytes = text.as_bytes();
    let (mut depth, mut i, mut in_str) = (1i32, body_start, false);
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if in_str => i += 1,
            b'"' => in_str = !in_str,
            b'[' | b'(' | b'{' if !in_str => depth += 1,
            b']' | b')' | b'}' if !in_str => {
                depth -= 1;
                if depth == 0 {
                    // `]` closes the array; the `)` after it closes the call.
                    let body = &text[body_start..i];
                    let rest = text[i + 1..].strip_prefix(')').unwrap_or(&text[i + 1..]);
                    return format!("{}vec![{body}]{rest}", &text[..start]);
                }
            }
            _ => {}
        }
        i += 1;
    }
    text.to_string()
}

/// Writes `root` into the prefab file `src` (its `PrefabAsset { root: .. }` literal).
pub fn write_prefab(src: &str, root: &ObjectDoc) -> Result<String, String> {
    write_returned(
        src,
        "PrefabAsset",
        &format!("PrefabAsset {{ root: {} }}", object_source(root)),
    )
}

fn write_returned(src: &str, ty: &str, literal: &str) -> Result<String, String> {
    let mut file = syn::parse_file(src).map_err(|e| format!("not valid Rust: {e}"))?;
    let expr: Expr = syn::parse_str(literal)
        .map_err(|e| format!("the editor produced invalid Rust ({e}): {literal}"))?;
    let function = file
        .items
        .iter_mut()
        .find_map(|item| match item {
            syn::Item::Fn(f) if matches!(&f.sig.output, syn::ReturnType::Type(_, t) if source(t).replace(' ', "").ends_with(ty)) => Some(f),
            _ => None,
        })
        .ok_or_else(|| format!("no function returning `{ty}`"))?;
    match function.block.stmts.last_mut() {
        Some(syn::Stmt::Expr(last, None)) => *last = expr,
        _ => {
            return Err(format!(
                "`fn {}` must end with a `{ty} {{ .. }}` literal",
                function.sig.ident
            ));
        }
    }
    Ok(prettyplease::unparse(&file))
}

/// The builder chain for an object.
pub fn object_source(object: &ObjectDoc) -> String {
    let mut out = match &object.prefab {
        Some(prefab) if object.name.is_empty() => format!("Object::prefab({prefab})"),
        Some(prefab) => format!("Object::prefab({prefab}).named({:?})", object.name),
        None => format!("Object::new({:?})", object.name),
    };
    for component in &object.components {
        out.push_str(&format!(".with({})", component_source(component)));
    }
    for child in &object.children {
        out.push_str(&format!(".child({})", object_source(child)));
    }
    out
}

pub fn component_source(component: &ComponentDoc) -> String {
    if let Some(original) = &component.original {
        return original.clone();
    }
    match &component.value {
        Value::Transform(t) => transform_source(t),
        Value::Struct { fields, rest } => {
            let mut parts: Vec<String> = fields
                .iter()
                .map(|(name, field)| format!("{name}: {}", field_source(field)))
                .collect();
            if *rest {
                parts.push("..default()".into());
            }
            format!("{} {{ {} }}", component.ty, parts.join(", "))
        }
        Value::Unit(src) | Value::Expr(src) => src.clone(),
    }
}

/// A float literal Rust accepts (`3.0`, `-0.5`); never NaN or infinite.
fn float(n: f32) -> String {
    // No NaN/infinity, and no `-0.0`.
    let n = if n.is_finite() && n != 0.0 { n } else { 0.0 };
    format!("{n:?}")
}

pub fn field_source(field: &Field) -> String {
    match field {
        Field::Num(n) => float(*n),
        Field::Int(i) => i.to_string(),
        Field::Bool(b) => b.to_string(),
        Field::Str(s) => format!("{s:?}"),
        Field::Vec2([x, y]) => format!("Vec2::new({}, {})", float(*x), float(*y)),
        Field::Vec3([x, y, z]) => format!("Vec3::new({}, {}, {})", float(*x), float(*y), float(*z)),
        Field::Color([r, g, b, a]) if *a >= 1.0 => {
            format!("Color::srgb({}, {}, {})", float(*r), float(*g), float(*b))
        }
        Field::Color([r, g, b, a]) => format!(
            "Color::srgba({}, {}, {}, {})",
            float(*r),
            float(*g),
            float(*b),
            float(*a)
        ),
        Field::OptStr(Some(s)) => format!("Some({s:?})"),
        Field::OptStr(None) => "None".into(),
        Field::Path(p) | Field::Expr(p) => p.clone(),
    }
}

/// `Transform::from_xyz(..)` when it is only a position; otherwise the full struct, with
/// rotation in degrees (`45.0_f32.to_radians()`, Bevy's YXZ order: yaw, pitch, roll).
pub fn transform_source(t: &TransformData) -> String {
    let [x, y, z] = t.translation;
    if t.rotation == [0.0; 3] && t.scale == [1.0; 3] {
        return format!(
            "Transform::from_xyz({}, {}, {})",
            float(x),
            float(y),
            float(z)
        );
    }
    let [rx, ry, rz] = t.rotation;
    let [sx, sy, sz] = t.scale;
    format!(
        "Transform {{ translation: Vec3::new({}, {}, {}), rotation: Quat::from_euler(EulerRot::YXZ, {}_f32.to_radians(), {}_f32.to_radians(), {}_f32.to_radians()), scale: Vec3::new({}, {}, {}) }}",
        float(x),
        float(y),
        float(z),
        float(ry),
        float(rx),
        float(rz),
        float(sx),
        float(sy),
        float(sz)
    )
}
