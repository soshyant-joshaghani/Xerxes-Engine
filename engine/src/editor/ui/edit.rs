//! Edits to the open scene. Every panel (Inspector, Hierarchy, gizmos, menus, keys) changes the
//! document through these, so they all behave the same: the Scene view follows, the scene is
//! marked dirty, the edit is recorded in the history (so Undo and Redo work everywhere), and
//! Save writes it back to its `.rs` file.

use dioxus::prelude::*;

use super::Editor;
use crate::editor::project::codec::{
    ComponentDoc, ConstDoc, Dimension, Field, ObjectDoc, SceneDoc, TransformData, Value,
};

/// What undo goes back to: the document and what was selected.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub scene: SceneDoc,
    pub selected: Option<u32>,
}

fn snapshot(editor: Editor) -> Option<Snapshot> {
    Some(Snapshot {
        scene: editor.scene.peek().clone()?,
        selected: *editor.selected.peek(),
    })
}

/// Records an edit that was just made (`before` is the state ahead of it) and marks the scene
/// dirty. Edits with the same `merge` key in a row (a drag, typing) become one step.
fn commit(editor: Editor, label: String, merge: Option<String>, before: Snapshot) {
    let Some(after) = snapshot(editor) else {
        return;
    };
    let (mut history, mut dirty) = (editor.history, editor.dirty);
    history
        .write()
        .record(&label, merge.as_deref(), before, after);
    dirty.set(true);
}

/// Every selected object: the primary one first, then the others.
pub fn selection(editor: Editor) -> Vec<u32> {
    let mut all: Vec<u32> = editor.selected.peek().iter().copied().collect();
    all.extend(editor.also.peek().iter().copied());
    all
}

/// Selects exactly this object (or nothing).
pub fn select_only(editor: Editor, id: Option<u32>) {
    let (mut selected, mut also) = (editor.selected, editor.also);
    selected.set(id);
    if !also.peek().is_empty() {
        also.set(Vec::new());
    }
}

/// Ctrl-click: adds the object to the selection, or takes it out. The last one added is the
/// primary (the one the Properties show and the gizmo sits on).
pub fn toggle_selection(editor: Editor, id: u32) {
    let mut all = selection(editor);
    if let Some(index) = all.iter().position(|o| *o == id) {
        all.remove(index);
    } else {
        all.insert(0, id);
    }
    let (mut selected, mut also) = (editor.selected, editor.also);
    selected.set(all.first().copied());
    also.set(all.into_iter().skip(1).collect());
}

/// Shift-click: everything between the primary object and this one, in the Outliner's order.
pub fn select_range(editor: Editor, id: u32) {
    let order = visible_order(editor);
    let anchor = *editor.selected.peek();
    let (Some(a), Some(b)) = (
        anchor.and_then(|a| order.iter().position(|o| *o == a)),
        order.iter().position(|o| *o == id),
    ) else {
        return select_only(editor, Some(id));
    };
    let (from, to) = (a.min(b), a.max(b));
    let mut also: Vec<u32> = order[from..=to]
        .iter()
        .copied()
        .filter(|o| Some(*o) != anchor)
        .collect();
    // The clicked one is the primary, the anchor stays in the set.
    also.retain(|o| *o != id);
    if let Some(anchor) = anchor {
        also.push(anchor);
    }
    let (mut selected, mut set) = (editor.selected, editor.also);
    selected.set(Some(id));
    set.set(also);
}

/// The objects in the Outliner's order, folded rows left out.
pub fn visible_order(editor: Editor) -> Vec<u32> {
    fn walk(objects: &[ObjectDoc], folded: &std::collections::HashSet<u32>, out: &mut Vec<u32>) {
        for object in objects {
            out.push(object.id);
            if !folded.contains(&object.id) {
                walk(&object.children, folded, out);
            }
        }
    }
    let mut out = Vec::new();
    if let Some(doc) = editor.scene.peek().as_ref() {
        walk(&doc.objects, &editor.collapsed.peek(), &mut out);
    }
    out
}

/// A gizmo drag of the primary object: the others of a multi-selection move by the same
/// distance (rotating and scaling act on the primary one).
pub fn move_selection(editor: Editor, id: u32, transform: TransformData) {
    let others: Vec<u32> = editor.also.peek().clone();
    let old = editor
        .scene
        .peek()
        .as_ref()
        .and_then(|d| d.find(id))
        .and_then(|o| o.transform());
    set_transform(editor, id, transform);
    let Some(old) = old else { return };
    let delta = [
        transform.translation[0] - old.translation[0],
        transform.translation[1] - old.translation[1],
        transform.translation[2] - old.translation[2],
    ];
    if others.is_empty() || delta == [0.0; 3] {
        return;
    }
    for other in others {
        let Some(mut at) = editor
            .scene
            .peek()
            .as_ref()
            .and_then(|d| d.find(other))
            .and_then(|o| o.transform())
        else {
            continue;
        };
        for axis in 0..3 {
            at.translation[axis] += delta[axis];
        }
        set_transform(editor, other, at);
    }
}

fn restore(editor: Editor, state: Snapshot) {
    let (mut scene, mut selected, mut dirty) = (editor.scene, editor.selected, editor.dirty);
    let mut also = editor.also;
    also.set(Vec::new());
    scene.set(Some(state.scene));
    selected.set(state.selected);
    dirty.set(!editor.history.peek().is_clean());
}

pub fn can_undo(editor: Editor) -> bool {
    editor.history.read().can_undo()
}

pub fn can_redo(editor: Editor) -> bool {
    editor.history.read().can_redo()
}

/// Reverts the last edit.
pub fn undo(editor: Editor) {
    let mut history = editor.history;
    let state = history.write().undo().cloned();
    if let Some(state) = state {
        restore(editor, state);
    }
}

/// Applies the last undone edit again.
pub fn redo(editor: Editor) {
    let mut history = editor.history;
    let state = history.write().redo().cloned();
    if let Some(state) = state {
        restore(editor, state);
    }
}

/// Goes to the state after `steps` edits (0: the scene as it was opened).
pub fn jump(editor: Editor, steps: usize) {
    let mut history = editor.history;
    let state = history.write().jump(steps);
    if let Some(state) = state {
        restore(editor, state);
    }
}

fn find_mut(objects: &mut [ObjectDoc], id: u32) -> Option<&mut ObjectDoc> {
    for object in objects {
        if object.id == id {
            return Some(object);
        }
        if let Some(found) = find_mut(&mut object.children, id) {
            return Some(found);
        }
    }
    None
}

fn max_id(doc: &SceneDoc) -> u32 {
    doc.all().iter().map(|o| o.id).max().unwrap_or(0)
}

/// The name an edit's label uses for an object.
fn name_of(editor: Editor, id: u32) -> String {
    editor
        .scene
        .peek()
        .as_ref()
        .and_then(|doc| doc.find(id))
        .map(|o| {
            if o.name.is_empty() {
                "object".to_string()
            } else {
                o.name.clone()
            }
        })
        .unwrap_or_else(|| "object".into())
}

/// Changes one object, records the edit and marks the scene dirty.
fn with_object(
    editor: Editor,
    id: u32,
    label: String,
    merge: Option<String>,
    change: impl FnOnce(&mut ObjectDoc),
) {
    let Some(before) = snapshot(editor) else {
        return;
    };
    let mut scene = editor.scene;
    let found = match scene.write().as_mut() {
        Some(doc) => match find_mut(&mut doc.objects, id) {
            Some(object) => {
                change(object);
                true
            }
            None => false,
        },
        None => false,
    };
    if found {
        commit(editor, label, merge, before);
    }
}

/// Makes the open scene 2D or 3D (its `dimension`); the Scene view switches with it. A prefab has
/// no dimension of its own — the editor reads one off what the prefab holds — so this does nothing
/// in prefab mode.
pub fn set_dimension(editor: Editor, dimension: Dimension) {
    if editor.in_prefab() {
        return;
    }
    let Some(before) = snapshot(editor) else {
        return;
    };
    let mut scene = editor.scene;
    let changed = match scene.write().as_mut() {
        Some(doc) if doc.dimension != dimension => {
            doc.dimension = dimension;
            true
        }
        _ => false,
    };
    if changed {
        let label = match dimension {
            Dimension::D2 => "Make scene 2D",
            Dimension::D3 => "Make scene 3D",
        };
        commit(editor, label.into(), None, before);
    }
}

/// The component type an override replaces (`MeshDef::Cube` → `MeshDef`).
fn base(ty: &str) -> &str {
    ty.split("::").next().unwrap_or(ty)
}

/// The object's own component of `ty`, created as an override of the prefab's (or empty)
/// when the object does not have one yet.
fn own_component<'a>(
    object: &'a mut ObjectDoc,
    ty: &str,
    inherited: Option<&ComponentDoc>,
) -> &'a mut ComponentDoc {
    let index = match object
        .components
        .iter()
        .position(|c| base(&c.ty) == base(ty))
    {
        Some(index) => index,
        None => {
            let component = inherited.cloned().unwrap_or_else(|| {
                ComponentDoc::new(
                    ty,
                    Value::Struct {
                        fields: Vec::new(),
                        // These are written as a few fields over their defaults.
                        rest: matches!(base(ty), "Node" | "UiTransform" | "TextFont"),
                    },
                )
            });
            object.components.push(component);
            object.components.len() - 1
        }
    };
    let component = &mut object.components[index];
    // Edited: write it from the document from now on, not from the source as read.
    component.original = None;
    component
}

pub fn set_transform(editor: Editor, id: u32, transform: TransformData) {
    let label = format!("Transform {}", name_of(editor, id));
    with_object(
        editor,
        id,
        label,
        Some(format!("transform:{id}")),
        |object| {
            let component = own_component(object, "Transform", None);
            component.ty = "Transform".into();
            component.value = Value::Transform(transform);
        },
    );
}

/// Sets one field of a component. `inherited` is the prefab's component when the object has
/// none of its own: editing it adds an override (like Unity's prefab overrides).
pub fn set_field(
    editor: Editor,
    id: u32,
    ty: &str,
    inherited: Option<ComponentDoc>,
    name: &str,
    value: Field,
) {
    let label = format!("Set {}.{name} of {}", base(ty), name_of(editor, id));
    let merge = Some(format!("field:{id}:{ty}:{name}"));
    with_object(editor, id, label, merge, |object| {
        let component = own_component(object, ty, inherited.as_ref());
        if let Value::Struct { fields, .. } = &mut component.value {
            match fields.iter_mut().find(|(n, _)| n == name) {
                Some((_, field)) => *field = value,
                None => fields.push((name.to_string(), value)),
            }
        }
    });
}

pub fn rename(editor: Editor, id: u32, name: String) {
    let label = format!("Rename {}", name_of(editor, id));
    with_object(editor, id, label, Some(format!("rename:{id}")), |object| {
        object.name = name
    });
}

/// Adds an empty object (a name and a Transform) under the selected object, or at the root;
/// selects it.
pub fn add_empty(editor: Editor, _parent: Option<u32>) {
    add_kind(editor, super::catalog::AddKind::Empty);
}

/// The component types that make an object a UI element (not a canvas).
const UI_ELEMENT: [&str; 6] = ["Node", "Text", "Button", "Toggle", "Slider", "Dropdown"];

fn has(object: &ObjectDoc, bases: &[&str]) -> bool {
    object
        .components
        .iter()
        .any(|c| bases.contains(&base(&c.ty)))
}

fn is_ui_element(object: &ObjectDoc) -> bool {
    has(object, &UI_ELEMENT) || has(object, &["RectTransform"])
}

/// The first canvas of the scene (depth first).
fn find_canvas(objects: &[ObjectDoc]) -> Option<u32> {
    for object in objects {
        if has(object, &["Canvas"]) {
            return Some(object.id);
        }
        if let Some(id) = find_canvas(&object.children) {
            return Some(id);
        }
    }
    None
}

fn from_spec(spec: &super::catalog::Spec, next: &mut u32) -> ObjectDoc {
    *next += 1;
    let id = *next;
    ObjectDoc {
        id,
        name: spec.name.clone(),
        components: spec.components.clone(),
        children: spec.children.iter().map(|c| from_spec(c, next)).collect(),
        ..Default::default()
    }
}

/// Adds a common object (a mesh, a light, a camera, a sprite, a UI element) and selects it.
/// A world object goes under the selected object when that is not a UI element; a UI
/// element goes under the selected UI object, else the first canvas, else a new canvas
/// that comes with it.
pub fn add_kind(editor: Editor, kind: super::catalog::AddKind) {
    use super::catalog::{self, AddKind, Spec};
    let Some(before) = snapshot(editor) else {
        return;
    };
    let selected = *editor.selected.peek();
    let mut scene = editor.scene;
    let mut chosen = None;
    if let Some(doc) = scene.write().as_mut() {
        let mut spec = catalog::spec(kind);
        let selected_object = selected.and_then(|id| doc.find(id));
        let parent = if kind == AddKind::Canvas {
            None
        } else if kind.is_ui_element() {
            let under = selected_object
                .filter(|o| is_ui_element(o) || has(o, &["Canvas"]))
                .map(|o| o.id)
                .or_else(|| find_canvas(&doc.objects));
            if under.is_none() {
                spec = Spec {
                    name: "Canvas".into(),
                    components: catalog::spec(AddKind::Canvas).components,
                    children: vec![spec],
                };
            }
            under
        } else {
            selected_object.filter(|o| !is_ui_element(o)).map(|o| o.id)
        };
        let mut next = max_id(doc);
        let object = from_spec(&spec, &mut next);
        // What gets selected: the element itself, not the canvas made around it.
        chosen = Some(if kind.is_ui_element() && parent.is_none() {
            object.children.first().map_or(object.id, |c| c.id)
        } else {
            object.id
        });
        match parent.and_then(|p| find_mut(&mut doc.objects, p)) {
            Some(parent) => parent.children.push(object),
            None => doc.objects.push(object),
        }
    }
    if let Some(id) = chosen {
        let mut selected = editor.selected;
        selected.set(Some(id));
        commit(editor, format!("Add {}", kind.label()), None, before);
    }
}

/// Replaces (or adds) an object's component of `base_ty` with this one: the Properties
/// panel's editors for components that are written as a call (`Canvas::overlay()`).
pub fn set_component(editor: Editor, id: u32, label: &str, component: ComponentDoc) {
    let label = format!("{label} of {}", name_of(editor, id));
    let key = Some(format!("component:{id}:{}", base(&component.ty)));
    with_object(editor, id, label, key, move |object| {
        match object
            .components
            .iter()
            .position(|c| base(&c.ty) == base(&component.ty))
        {
            Some(index) => object.components[index] = component,
            None => object.components.push(component),
        }
    });
}

/// Makes a camera orthographic or perspective, like Unity's one Camera with a projection
/// choice. Bevy has a 2D camera (always orthographic) and a 3D one: a 2D camera that becomes
/// perspective is a 3D camera, and a 3D camera that becomes orthographic keeps its type and
/// gets an orthographic `Projection`.
pub fn set_camera_projection(editor: Editor, id: u32, orthographic: bool) {
    let label = format!(
        "{} for {}",
        if orthographic {
            "Orthographic"
        } else {
            "Perspective"
        },
        name_of(editor, id)
    );
    with_object(editor, id, label, None, move |object| {
        let is_2d = object
            .components
            .iter()
            .any(|c| matches!(base(&c.ty), "Camera2d" | "GameCamera2D"));
        object.components.retain(|c| base(&c.ty) != "Projection");
        if orthographic {
            if !is_2d {
                object.components.push(ComponentDoc::new(
                    "Projection::Orthographic",
                    Value::Expr(
                        "Projection::Orthographic(OrthographicProjection { scale: 0.01, ..OrthographicProjection::default_3d() })"
                            .into(),
                    ),
                ));
            }
        } else if is_2d {
            // A perspective camera is a 3D game camera.
            for component in &mut object.components {
                match base(&component.ty) {
                    "Camera2d" => {
                        *component = ComponentDoc::new(
                            "Camera3d",
                            Value::Unit("Camera3d::default()".into()),
                        );
                    }
                    "GameCamera2D" => {
                        *component =
                            ComponentDoc::new("GameCamera3D", Value::Unit("GameCamera3D".into()));
                    }
                    _ => {}
                }
            }
        }
    });
}

/// Applies a finished Rect-tool drag: a UI element's `RectTransform`, or a sprite's size and
/// position. One edit, so one Undo.
pub fn apply_rect_drag(editor: Editor, drag: crate::editor::protocol::RectDrag) {
    use crate::editor::bridge::ui_view::{
        ViewXform, drag_rect, drag_sprite, rect_of, xform_component, xform_of,
    };
    use crate::editor::protocol::RectKind;
    if drag.kind == RectKind::Resize && drag.delta[0].abs() < 1e-4 && drag.delta[1].abs() < 1e-4 {
        return;
    }
    let Some(object) = editor
        .scene
        .peek()
        .as_ref()
        .and_then(|doc| doc.find(drag.id))
        .cloned()
    else {
        return;
    };
    let id = drag.id;
    if drag.kind != RectKind::Resize {
        // Scale or rotate a UI element: its `UiTransform`.
        let mut x = object
            .component("UiTransform")
            .and_then(xform_of)
            .unwrap_or_default();
        let label = match drag.kind {
            RectKind::Scale => {
                x.scale = [x.scale[0] * drag.scale[0], x.scale[1] * drag.scale[1]];
                format!("Scale {}", name_of(editor, id))
            }
            _ => {
                x.rotation = (x.rotation + drag.rotation + 180.0).rem_euclid(360.0) - 180.0;
                format!("Rotate {}", name_of(editor, id))
            }
        };
        let x = ViewXform {
            rotation: x.rotation,
            scale: x.scale,
        };
        with_object(editor, id, label, None, move |object| {
            let component = xform_component(x);
            match object
                .components
                .iter()
                .position(|c| base(&c.ty) == "UiTransform")
            {
                Some(index) => object.components[index] = component,
                None => object.components.push(component),
            }
        });
        return;
    }
    let label = format!("Resize {}", name_of(editor, drag.id));
    if drag.ui && object.component("RectTransform").is_none() {
        apply_node_drag(editor, &object, &drag, label);
        return;
    }
    if drag.ui {
        let Some(rect) = object
            .components
            .iter()
            .find(|c| base(&c.ty) == "RectTransform")
            .and_then(rect_of)
        else {
            return;
        };
        let delta = [
            drag.delta[0] * drag.pixels_per_unit,
            drag.delta[1] * drag.pixels_per_unit,
        ];
        let next = drag_rect(rect, drag.handle, delta, drag.size);
        let vec2 = |v: [f32; 2]| Field::Vec2(v);
        let component = ComponentDoc::new(
            "RectTransform",
            Value::Struct {
                fields: vec![
                    ("anchor_min".into(), vec2(next.anchor_min)),
                    ("anchor_max".into(), vec2(next.anchor_max)),
                    ("pivot".into(), vec2(next.pivot)),
                    ("position".into(), vec2(next.position)),
                    ("size".into(), vec2(next.size)),
                ],
                rest: false,
            },
        );
        with_object(editor, id, label, None, move |object| {
            match object
                .components
                .iter()
                .position(|c| base(&c.ty) == "RectTransform")
            {
                Some(index) => object.components[index] = component,
                None => object.components.push(component),
            }
        });
        return;
    }
    // A sprite: its `SpriteDef` size and its position.
    let size = object
        .component("SpriteDef")
        .and_then(|c| match &c.value {
            Value::Struct { fields, .. } => {
                fields.iter().find_map(|(n, f)| match (n.as_str(), f) {
                    ("size", Field::Expr(src)) => crate::editor::view::parse_some_vec2(src),
                    _ => None,
                })
            }
            _ => None,
        })
        .unwrap_or(drag.size);
    let at = object.transform().unwrap_or_default();
    let (shift, new_size) = drag_sprite([0.0, 0.0], size, drag.handle, drag.delta);
    with_object(editor, id, label, None, move |object| {
        let transform = own_component(object, "Transform", None);
        transform.ty = "Transform".into();
        let mut at = at;
        at.translation[0] += shift[0];
        at.translation[1] += shift[1];
        transform.value = Value::Transform(at);
        if let Some(sprite) = object
            .components
            .iter_mut()
            .find(|c| base(&c.ty) == "SpriteDef")
        {
            sprite.original = None;
            if let Value::Struct { fields, .. } = &mut sprite.value {
                let value = Field::Expr(format!(
                    "Some(Vec2::new({:?}, {:?}))",
                    new_size[0], new_size[1]
                ));
                match fields.iter_mut().find(|(n, _)| n == "size") {
                    Some((_, f)) => *f = value,
                    None => fields.push(("size".into(), value)),
                }
            }
        }
    });
}

/// A Rect-tool drag of a flexbox element (a `Node` with no `RectTransform`): the offset and the
/// size go into its `Node` fields (`left`, `top`, `width`, `height`), so the layout stays the
/// parent's and the element is only nudged or sized.
fn apply_node_drag(
    editor: Editor,
    object: &ObjectDoc,
    drag: &crate::editor::protocol::RectDrag,
    label: String,
) {
    use crate::editor::bridge::ui_view::{NodeFrame, ViewVal, drag_node, parse_val, val_source};
    let fields: Vec<(String, Field)> = match object.component("Node").map(|c| &c.value) {
        Some(Value::Struct { fields, .. }) => fields.clone(),
        _ => Vec::new(),
    };
    let authored = |name: &str| {
        fields
            .iter()
            .find(|(n, _)| n == name)
            .and_then(|(_, f)| match f {
                Field::Expr(src) | Field::Path(src) => parse_val(src),
                Field::Num(n) => Some(ViewVal::Px(*n)),
                Field::Int(n) => Some(ViewVal::Px(*n as f32)),
                _ => None,
            })
            .unwrap_or(ViewVal::Auto)
    };
    let absolute = fields.iter().any(|(n, f)| {
        n == "position_type" && matches!(f, Field::Path(p) if p.ends_with("Absolute"))
    });
    let frame = NodeFrame {
        absolute,
        left: authored("left"),
        top: authored("top"),
        right: authored("right"),
        bottom: authored("bottom"),
    };
    let delta = [
        drag.delta[0] * drag.pixels_per_unit,
        drag.delta[1] * drag.pixels_per_unit,
    ];
    let edit = drag_node(&frame, drag.handle, delta, drag.size, drag.offset);
    let id = drag.id;
    with_object(editor, id, label, None, move |object| {
        let component = own_component(object, "Node", None);
        if !matches!(component.value, Value::Struct { .. }) {
            // `Node::default()`: the same node, now with fields.
            component.ty = "Node".into();
            component.value = Value::Struct {
                fields: Vec::new(),
                rest: true,
            };
        }
        let Value::Struct { fields, .. } = &mut component.value else {
            return;
        };
        let mut set = |name: &str, v: f32| {
            let value = Field::Expr(val_source(ViewVal::Px(v)));
            match fields.iter_mut().find(|(n, _)| n == name) {
                Some((_, f)) => *f = value,
                None => fields.push((name.to_string(), value)),
            }
        };
        for (name, v) in [
            ("left", edit.left),
            ("top", edit.top),
            ("width", edit.width),
            ("height", edit.height),
        ] {
            if let Some(v) = v {
                set(name, v);
            }
        }
        for (name, clear) in [("right", edit.clear_right), ("bottom", edit.clear_bottom)] {
            if clear {
                fields.retain(|(n, _)| n != name);
            }
        }
    });
}

/// Moves an object (with its children) under `parent` (`None`: the scene's root), at the end
/// of its children. An object cannot go under itself or its own descendants.
pub fn reparent(editor: Editor, id: u32, parent: Option<u32>) {
    let Some(before) = snapshot(editor) else {
        return;
    };
    let label = format!("Move {}", name_of(editor, id));
    let mut scene = editor.scene;
    let mut moved = false;
    if let Some(doc) = scene.write().as_mut() {
        if let Some(target) = parent {
            let Some(object) = doc.find(id) else {
                return;
            };
            let mut below = Vec::new();
            object.walk(&mut below);
            if below.iter().any(|o| o.id == target) {
                return;
            }
        }
        // Already there: nothing to do.
        let current: Option<u32> = doc
            .all()
            .into_iter()
            .find(|o| o.children.iter().any(|c| c.id == id))
            .map(|o| o.id);
        if current == parent && (parent.is_some() || doc.objects.iter().any(|o| o.id == id)) {
            return;
        }
        let Some(object) = remove(&mut doc.objects, id) else {
            return;
        };
        match parent.and_then(|p| find_mut(&mut doc.objects, p)) {
            Some(p) => p.children.push(object),
            None => doc.objects.push(object),
        }
        moved = true;
    }
    if moved {
        commit(editor, label, None, before);
    }
}

/// Shows or hides an object (`Visibility::Hidden`), and everything under it.
pub fn set_hidden(editor: Editor, id: u32, hidden: bool) {
    let label = format!(
        "{} {}",
        if hidden { "Hide" } else { "Show" },
        name_of(editor, id)
    );
    with_object(editor, id, label, None, |object| {
        object.components.retain(|c| base(&c.ty) != "Visibility");
        if hidden {
            object.components.push(ComponentDoc::new(
                "Visibility::Hidden",
                Value::Unit("Visibility::Hidden".into()),
            ));
        }
    });
}

/// Makes a camera active or inactive in the game (`Camera { is_active: false, .. }`).
pub fn set_camera_active(editor: Editor, id: u32, active: bool) {
    let label = format!(
        "{} camera {}",
        if active { "Activate" } else { "Deactivate" },
        name_of(editor, id)
    );
    with_object(editor, id, label, None, |object| {
        object.components.retain(|c| base(&c.ty) != "Camera");
        if !active {
            object.components.push(ComponentDoc::new(
                "Camera",
                Value::Struct {
                    fields: vec![("is_active".into(), Field::Bool(false))],
                    rest: true,
                },
            ));
        }
    });
}

fn remove(objects: &mut Vec<ObjectDoc>, id: u32) -> Option<ObjectDoc> {
    if let Some(index) = objects.iter().position(|o| o.id == id) {
        return Some(objects.remove(index));
    }
    objects.iter_mut().find_map(|o| remove(&mut o.children, id))
}

pub fn delete(editor: Editor, id: u32) {
    delete_all(editor, &[id]);
}

/// Deletes these objects as one edit.
pub fn delete_all(editor: Editor, ids: &[u32]) {
    // A prefab *is* its root object: deleting it would leave nothing to write back.
    if editor.in_prefab() {
        let root = editor
            .scene
            .peek()
            .as_ref()
            .and_then(|doc| doc.objects.first().map(|o| o.id));
        if let Some(root) = root {
            if ids.contains(&root) {
                return editor
                    .error("a prefab is its root object: it cannot be deleted (delete the file)");
            }
        }
    }
    let Some(before) = snapshot(editor) else {
        return;
    };
    let label = match ids {
        [one] => format!("Delete {}", name_of(editor, *one)),
        many => format!("Delete {} objects", many.len()),
    };
    let mut scene = editor.scene;
    let mut removed = false;
    if let Some(doc) = scene.write().as_mut() {
        for id in ids {
            removed |= remove(&mut doc.objects, *id).is_some();
        }
    }
    if removed {
        select_only(editor, None);
        commit(editor, label, None, before);
    }
}

/// Copies an object (and its children, with new ids) right after it; selects the copy.
pub fn duplicate(editor: Editor, id: u32) {
    fn renumber(object: &mut ObjectDoc, next: &mut u32) {
        *next += 1;
        object.id = *next;
        for child in &mut object.children {
            renumber(child, next);
        }
    }
    fn insert_after(objects: &mut Vec<ObjectDoc>, id: u32, copy: &mut Option<ObjectDoc>) {
        if let Some(index) = objects.iter().position(|o| o.id == id) {
            if let Some(copy) = copy.take() {
                objects.insert(index + 1, copy);
            }
            return;
        }
        for object in objects {
            insert_after(&mut object.children, id, copy);
        }
    }
    let Some(before) = snapshot(editor) else {
        return;
    };
    let label = format!("Duplicate {}", name_of(editor, id));
    let mut scene = editor.scene;
    let mut copied = None;
    if let Some(doc) = scene.write().as_mut() {
        let Some(original) = doc.find(id).cloned() else {
            return;
        };
        let mut copy = original;
        let mut next = max_id(doc);
        renumber(&mut copy, &mut next);
        if !copy.name.is_empty() {
            copy.name = format!("{} (1)", copy.name);
        }
        copied = Some(copy.id);
        insert_after(&mut doc.objects, id, &mut Some(copy));
    }
    if let Some(copy) = copied {
        let mut selected = editor.selected;
        selected.set(Some(copy));
        commit(editor, label, None, before);
    }
}

// ---- the document's constants ----------------------------------------------------------

/// The names a document's constants already use.
fn const_names(editor: Editor) -> Vec<String> {
    editor
        .scene
        .peek()
        .as_ref()
        .map(|doc| doc.consts.iter().map(|c| c.name.clone()).collect())
        .unwrap_or_default()
}

/// Adds a constant to the open document, with a name that is not taken yet. Constants are plain
/// Rust (`const PANEL: Color = Color::srgb(0.0, 0.0, 0.0);`), written back into the file on Save.
pub fn add_const(editor: Editor) {
    let Some(before) = snapshot(editor) else {
        return;
    };
    let taken = const_names(editor);
    let mut name = String::from("COLOR");
    let mut n = 1;
    while taken.contains(&name) {
        n += 1;
        name = format!("COLOR_{n}");
    }
    let wanted = ConstDoc::new(&name, "Color", "Color::srgb(0.5, 0.5, 0.5)");
    let mut scene = editor.scene;
    if let Some(doc) = scene.write().as_mut() {
        doc.consts.push(wanted);
    }
    commit(editor, format!("Add {name}"), None, before);
}

/// Changes one constant. The parts that are `None` stay as they are; a change the file would not
/// accept (a value that is not an expression, a name that is not an identifier or is taken) is
/// refused, so the document is always valid Rust.
pub fn set_const(
    editor: Editor,
    index: usize,
    name: Option<String>,
    ty: Option<String>,
    value: Option<String>,
) {
    let Some(before) = snapshot(editor) else {
        return;
    };
    let taken = const_names(editor);
    let Some(old) = editor
        .scene
        .peek()
        .as_ref()
        .and_then(|doc| doc.consts.get(index).cloned())
    else {
        return;
    };
    let wanted = ConstDoc {
        name: name.clone().unwrap_or_else(|| old.name.clone()),
        ty: ty.unwrap_or_else(|| old.ty.clone()),
        value: value.unwrap_or_else(|| old.value.clone()),
    };
    if wanted == old {
        return;
    }
    if wanted.item().is_err() {
        return;
    }
    if name.is_some() && taken.iter().any(|n| *n == wanted.name && *n != old.name) {
        return editor.error(format!("a constant named {} is already there", wanted.name));
    }
    let renamed = (wanted.name != old.name).then(|| (old.name.clone(), wanted.name.clone()));
    let label = format!("Set {}", wanted.name);
    let mut scene = editor.scene;
    if let Some(doc) = scene.write().as_mut() {
        doc.consts[index] = wanted;
        // A renamed constant is renamed where the document uses it, so the file still compiles.
        if let Some((from, to)) = &renamed {
            for object in &mut doc.objects {
                rename_in_object(object, from, to);
            }
            for item in &mut doc.consts {
                item.value = rename_ident(&item.value, from, to);
            }
        }
    }
    commit(editor, label, Some(format!("const-{index}")), before);
}

/// Removes a constant. What used it keeps the name, which would not compile, so a constant in use
/// is refused with the objects that name it.
pub fn delete_const(editor: Editor, index: usize) {
    let Some(before) = snapshot(editor) else {
        return;
    };
    let Some(doc) = editor.scene.peek().clone() else {
        return;
    };
    let Some(item) = doc.consts.get(index).cloned() else {
        return;
    };
    let mut users: Vec<String> = doc
        .all()
        .iter()
        .filter(|object| uses_ident(object, &item.name))
        .map(|object| object.name.clone())
        .collect();
    // A constant can be written in terms of another one, which counts as a use too.
    users.extend(
        doc.consts
            .iter()
            .filter(|other| other.name != item.name && names_ident(&other.value, &item.name))
            .map(|other| other.name.clone()),
    );
    if !users.is_empty() {
        return editor.error(format!(
            "{} is used by {}: change those first",
            item.name,
            users.join(", ")
        ));
    }
    let mut scene = editor.scene;
    if let Some(doc) = scene.write().as_mut() {
        doc.consts.remove(index);
    }
    commit(editor, format!("Delete {}", item.name), None, before);
}

/// Whether an object's own components name this identifier.
fn uses_ident(object: &ObjectDoc, name: &str) -> bool {
    object.components.iter().any(|component| {
        let in_original = component
            .original
            .as_deref()
            .is_some_and(|src| names_ident(src, name));
        in_original || names_value(&component.value, name)
    })
}

fn names_value(value: &Value, name: &str) -> bool {
    match value {
        Value::Struct { fields, .. } => fields.iter().any(|(_, field)| match field {
            Field::Path(p) | Field::Expr(p) => names_ident(p, name),
            _ => false,
        }),
        Value::Unit(src) | Value::Expr(src) => names_ident(src, name),
        Value::Transform(_) => false,
    }
}

fn rename_in_object(object: &mut ObjectDoc, from: &str, to: &str) {
    for component in &mut object.components {
        if let Some(original) = &mut component.original {
            *original = rename_ident(original, from, to);
        }
        match &mut component.value {
            Value::Struct { fields, .. } => {
                for (_, field) in fields.iter_mut() {
                    match field {
                        Field::Path(p) | Field::Expr(p) => *p = rename_ident(p, from, to),
                        _ => {}
                    }
                }
            }
            Value::Unit(src) | Value::Expr(src) => *src = rename_ident(src, from, to),
            Value::Transform(_) => {}
        }
    }
    for child in &mut object.children {
        rename_in_object(child, from, to);
    }
}

/// Whether `text` names the identifier `name` (a whole word, not a part of a longer one).
fn names_ident(text: &str, name: &str) -> bool {
    idents(text).any(|(start, end)| &text[start..end] == name)
}

/// The identifier spans of some source text, as (start, end). Text inside a string literal is
/// not code: a label that happens to read like a constant's name is left alone.
fn idents(text: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    let bytes = text.as_bytes();
    let (mut i, mut in_str, mut escaped) = (0, false, false);
    std::iter::from_fn(move || {
        while i < bytes.len() {
            let c = bytes[i] as char;
            if in_str {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_str = false;
                }
                i += 1;
                continue;
            }
            if c == '"' {
                in_str = true;
                i += 1;
                continue;
            }
            if c.is_ascii_alphabetic() || c == '_' {
                let start = i;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if c.is_ascii_alphanumeric() || c == '_' {
                        i += 1;
                    } else {
                        break;
                    }
                }
                return Some((start, i));
            }
            i += 1;
        }
        None
    })
}

/// Renames every whole-word use of `from` in some source text.
fn rename_ident(text: &str, from: &str, to: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    for (start, end) in idents(text).collect::<Vec<_>>() {
        if &text[start..end] == from {
            out.push_str(&text[at..start]);
            out.push_str(to);
            at = end;
        }
    }
    out.push_str(&text[at..]);
    out
}

// ---- prefab overrides ------------------------------------------------------------------

/// Takes an override off a prefab instance: the object's own component of this type goes, so the
/// prefab's shows again (Unity's Revert).
pub fn revert_override(editor: Editor, id: u32, ty: &str) {
    let wanted = base(ty).to_string();
    let name = name_of(editor, id);
    with_object(
        editor,
        id,
        format!("Revert {wanted} on {name}"),
        None,
        move |object| object.components.retain(|c| base(&c.ty) != wanted),
    );
}

/// The component an instance overrides, and the prefab file it belongs to: what Apply needs.
pub fn override_source(editor: Editor, id: u32, ty: &str) -> Option<(String, ComponentDoc)> {
    let doc = editor.scene.peek().clone()?;
    let object = doc.find(id)?;
    let reference = object.prefab.clone()?;
    let component = object
        .components
        .iter()
        .find(|c| base(&c.ty) == base(ty))
        .cloned()?;
    let path = editor
        .files
        .peek()
        .iter()
        .find(|file| {
            !file.dir
                && file.path.ends_with(".prefab.rs")
                && crate::editor::project::prefab_ref(&file.path) == reference
        })
        .map(|file| file.path.clone())?;
    Some((path, component))
}

#[cfg(test)]
mod tests {
    use super::{names_ident, rename_ident};

    /// Renaming a constant rewrites the code that names it, and only that: a longer name that
    /// merely starts with it is a different identifier, and text inside a string is not code.
    #[test]
    fn renaming_a_constant_rewrites_the_code_that_names_it() {
        let source = r#"BackgroundColor(PANEL) PANEL_SOON Text::new("PANEL")"#;
        assert!(names_ident(source, "PANEL"));
        assert!(names_ident(source, "PANEL_SOON"));
        assert!(!names_ident(source, "PANE"));
        assert_eq!(
            rename_ident(source, "PANEL", "BAR"),
            r#"BackgroundColor(BAR) PANEL_SOON Text::new("PANEL")"#
        );
        // A name only inside a string is not a use at all.
        assert!(!names_ident(r#"Text::new("TITLE")"#, "TITLE"));
    }
}
