//! The Inspector: the selected object's name and components, editable. A prefab instance
//! lists its own components and those it inherits from the prefab (marked); editing an
//! inherited one adds an override to the instance. Fields the editor does not understand
//! show their source, read-only.
//!
//! Inputs apply as soon as what is typed parses (a half-typed `1.` or `-` waits), so the
//! Scene view follows the typing; nothing is written to disk until Save.

use dioxus::prelude::*;

use super::hierarchy::display_name;
use super::{Editor, edit};
use crate::editor::project::codec::{ComponentDoc, Field, TransformData, Value};

#[component]
pub fn Inspector() -> Element {
    let editor = use_context::<Editor>();
    let scene = editor.scene.read().clone();
    let selected = *editor.selected.read();
    let object = match (scene.as_ref(), selected) {
        (Some(scene), Some(id)) => scene.find(id).cloned(),
        _ => None,
    };
    let Some(object) = object else {
        return rsx! {
            div { class: "body", div { class: "hint", "Select an object in the Hierarchy or the Scene view." } }
        };
    };

    let prefabs = editor.prefabs.read();
    let shown_name = display_name(&object, &prefabs);
    let base = |ty: &str| ty.split("::").next().unwrap_or(ty).to_string();
    let inherited: Vec<ComponentDoc> = object
        .prefab
        .as_ref()
        .and_then(|p| prefabs.get(p))
        .map(|root| {
            root.components
                .iter()
                .filter(|c| {
                    !object
                        .components
                        .iter()
                        .any(|own| base(&own.ty) == base(&c.ty))
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let prefab = object.prefab.as_ref().map(|p| {
        p.rsplit("::")
            .nth(1)
            .unwrap_or(p)
            .trim_end_matches("_prefab")
            .to_string()
    });
    let id = object.id;
    // A prefab instance without a Transform of its own still has a position: the prefab's (or
    // the origin). Show it, and editing it adds the override.
    let has_transform = object.components.iter().any(|c| c.ty == "Transform")
        || inherited.iter().any(|c| c.ty == "Transform");

    rsx! {
        div { class: "body",
            div { class: "title",
                input {
                    class: "num name-input",
                    value: "{shown_name}",
                    oninput: move |e| edit::rename(editor, id, e.value()),
                }
                if let Some(prefab) = prefab {
                    span { class: "prefab", "Prefab: {prefab}" }
                }
            }
            if !has_transform {
                ComponentView {
                    key: "{id}-add-transform",
                    id,
                    component: ComponentDoc::new("Transform", Value::Transform(TransformData::default())),
                    inherited: true,
                }
            }
            for (i, component) in object.components.iter().cloned().enumerate() {
                ComponentView { key: "own-{i}-{component.ty}", id, component, inherited: false }
            }
            for (i, component) in inherited.into_iter().enumerate() {
                ComponentView { key: "prefab-{i}-{component.ty}", id, component, inherited: true }
            }
        }
    }
}

#[component]
fn ComponentView(id: u32, component: ComponentDoc, inherited: bool) -> Element {
    let title = if inherited {
        format!("{}  (prefab)", component.ty)
    } else {
        component.ty.clone()
    };
    let ty = component.ty.clone();
    let base_component = inherited.then(|| component.clone());
    rsx! {
        div { class: "section",
            div { class: "section-head", "{title}" }
            match component.value {
                Value::Transform(t) => rsx! { TransformFields { id, t } },
                Value::Struct { fields, rest } => rsx! {
                    for (name, value) in fields {
                        FieldRow { key: "{name}", id, ty: ty.clone(), inherited: base_component.clone(), name, value }
                    }
                    if rest {
                        div { class: "field muted", "other fields: defaults" }
                    }
                },
                Value::Unit(_) => rsx! { div { class: "field muted", "no fields" } },
                Value::Expr(src) => rsx! { div { class: "field", span { class: "code", "{src}" } } },
            }
        }
    }
}

pub fn num(n: f32) -> String {
    let s = format!("{n:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

/// A number input: applies `apply(value)` whenever the text parses.
#[component]
fn NumInput(value: f32, on_value: EventHandler<f32>) -> Element {
    rsx! {
        input {
            class: "num",
            value: "{num(value)}",
            oninput: move |e| {
                if let Ok(v) = e.value().trim().parse::<f32>() {
                    if v.is_finite() {
                        on_value.call(v);
                    }
                }
            },
        }
    }
}

#[component]
fn TransformFields(id: u32, t: TransformData) -> Element {
    let editor = use_context::<Editor>();
    let row = move |label: &'static str, v: [f32; 3], set: fn(&mut TransformData, usize, f32)| {
        rsx! {
            div { class: "field",
                span { class: "label", "{label}" }
                div { class: "value",
                    for (i, axis) in ["X", "Y", "Z"].into_iter().enumerate() {
                        span { key: "{axis}", class: "axis", "{axis}" }
                        NumInput {
                            value: v[i],
                            on_value: move |n| {
                                let mut next = t;
                                set(&mut next, i, n);
                                edit::set_transform(editor, id, next);
                            },
                        }
                    }
                }
            }
        }
    };
    rsx! {
        {row("Position", t.translation, |t, i, n| t.translation[i] = n)}
        {row("Rotation", t.rotation, |t, i, n| t.rotation[i] = n)}
        {row("Scale", t.scale, |t, i, n| t.scale[i] = n)}
    }
}

#[component]
fn FieldRow(
    id: u32,
    ty: String,
    inherited: Option<ComponentDoc>,
    name: String,
    value: Field,
) -> Element {
    let editor = use_context::<Editor>();
    let set = {
        let (ty, inherited, name) = (ty.clone(), inherited.clone(), name.clone());
        move |field: Field| edit::set_field(editor, id, &ty, inherited.clone(), &name, field)
    };
    let shown = match value.clone() {
        Field::Num(n) => {
            let set = set.clone();
            rsx! { NumInput { value: n, on_value: move |v| set(Field::Num(v)) } }
        }
        Field::Int(i) => {
            let set = set.clone();
            rsx! {
                input {
                    class: "num",
                    value: "{i}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().trim().parse::<i64>() {
                            set(Field::Int(v));
                        }
                    },
                }
            }
        }
        Field::Bool(b) => {
            let set = set.clone();
            rsx! {
                button {
                    class: if b { "on" } else { "" },
                    onclick: move |_| set(Field::Bool(!b)),
                    if b { "true" } else { "false" }
                }
            }
        }
        Field::Str(s) => {
            let set = set.clone();
            rsx! { input { class: "num", value: "{s}", oninput: move |e| set(Field::Str(e.value())) } }
        }
        Field::OptStr(s) => {
            let set = set.clone();
            let text = s.clone().unwrap_or_default();
            rsx! {
                input {
                    class: "num",
                    value: "{text}",
                    placeholder: "none",
                    oninput: move |e| {
                        let v = e.value();
                        set(Field::OptStr(if v.trim().is_empty() { None } else { Some(v) }));
                    },
                }
            }
        }
        Field::Vec2(v) => {
            let set = set.clone();
            rsx! {
                for (i, axis) in ["X", "Y"].into_iter().enumerate() {
                    span { key: "{axis}", class: "axis", "{axis}" }
                    NumInput {
                        value: v[i],
                        on_value: {
                            let set = set.clone();
                            move |n| {
                                let mut next = v;
                                next[i] = n;
                                set(Field::Vec2(next));
                            }
                        },
                    }
                }
            }
        }
        Field::Vec3(v) => {
            let set = set.clone();
            rsx! {
                for (i, axis) in ["X", "Y", "Z"].into_iter().enumerate() {
                    span { key: "{axis}", class: "axis", "{axis}" }
                    NumInput {
                        value: v[i],
                        on_value: {
                            let set = set.clone();
                            move |n| {
                                let mut next = v;
                                next[i] = n;
                                set(Field::Vec3(next));
                            }
                        },
                    }
                }
            }
        }
        Field::Color(c) => {
            let set = set.clone();
            let [r, g, b, a] = c;
            let css = format!(
                "background: rgba({}, {}, {}, {a});",
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8
            );
            rsx! {
                span { class: "swatch", style: "{css}" }
                for (i, channel) in ["R", "G", "B"].into_iter().enumerate() {
                    span { key: "{channel}", class: "axis", "{channel}" }
                    NumInput {
                        value: c[i],
                        on_value: {
                            let set = set.clone();
                            move |n: f32| {
                                let mut next = c;
                                next[i] = n.clamp(0.0, 1.0);
                                set(Field::Color(next));
                            }
                        },
                    }
                }
            }
        }
        Field::Path(p) => rsx! { span { class: "code", "{p}" } },
        Field::Expr(src) => rsx! { span { class: "code", "{src}" } },
    };
    rsx! {
        div { class: "field",
            span { class: "label", "{name}" }
            div { class: "value", {shown} }
        }
    }
}
