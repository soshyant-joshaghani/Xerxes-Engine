//! Edits to the open scene. Every panel (Inspector, Hierarchy, gizmos, menus, keys) changes the
//! document through these, so they all behave the same: the Scene view follows, the scene is
//! marked dirty, the edit is recorded in the history (so Undo and Redo work everywhere), and
//! Save writes it back to its `.rs` file.

use dioxus::prelude::*;

use super::Editor;
use crate::editor::project::codec::{
    ComponentDoc, Dimension, Field, ObjectDoc, SceneDoc, TransformData, Value,
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

fn restore(editor: Editor, state: Snapshot) {
    let (mut scene, mut selected, mut dirty) = (editor.scene, editor.selected, editor.dirty);
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

/// Makes the open scene 2D or 3D (its `dimension`); the Scene view switches with it.
pub fn set_dimension(editor: Editor, dimension: Dimension) {
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
                        rest: false,
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

/// Adds an empty object (a name and a Transform) at the root, or under `parent`; selects it.
pub fn add_empty(editor: Editor, parent: Option<u32>) {
    let Some(before) = snapshot(editor) else {
        return;
    };
    let mut scene = editor.scene;
    let mut selected = editor.selected;
    let mut new_id = None;
    if let Some(doc) = scene.write().as_mut() {
        let id = max_id(doc) + 1;
        let object = ObjectDoc {
            id,
            name: "New Object".into(),
            components: vec![ComponentDoc::new(
                "Transform",
                Value::Transform(TransformData::default()),
            )],
            ..Default::default()
        };
        match parent.and_then(|p| find_mut(&mut doc.objects, p)) {
            Some(parent) => parent.children.push(object),
            None => doc.objects.push(object),
        }
        new_id = Some(id);
    }
    if let Some(id) = new_id {
        selected.set(Some(id));
        commit(editor, "Add empty object".into(), None, before);
    }
}

fn remove(objects: &mut Vec<ObjectDoc>, id: u32) -> Option<ObjectDoc> {
    if let Some(index) = objects.iter().position(|o| o.id == id) {
        return Some(objects.remove(index));
    }
    objects.iter_mut().find_map(|o| remove(&mut o.children, id))
}

pub fn delete(editor: Editor, id: u32) {
    let Some(before) = snapshot(editor) else {
        return;
    };
    let label = format!("Delete {}", name_of(editor, id));
    let mut scene = editor.scene;
    let removed = scene
        .write()
        .as_mut()
        .and_then(|doc| remove(&mut doc.objects, id))
        .is_some();
    if removed {
        let mut selected = editor.selected;
        selected.set(None);
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
