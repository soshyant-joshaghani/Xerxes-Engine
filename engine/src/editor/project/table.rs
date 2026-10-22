//! The Spreadsheet's rows: every object of the open scene, flattened (parents before their
//! children), with what a table wants of it. Plain data, tested.

use std::collections::HashMap;

use super::codec::{ObjectDoc, SceneDoc, TransformData, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub id: u32,
    /// How deep it sits in the scene (0 at the root).
    pub depth: usize,
    pub name: String,
    /// What it is: its prefab, else its first component that is not the Transform.
    pub kind: String,
    pub position: [f32; 3],
    /// Degrees about X, Y, Z.
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    /// Its own components and the ones it takes from its prefab.
    pub components: usize,
}

fn base(ty: &str) -> &str {
    ty.split("::").next().unwrap_or(ty)
}

fn transform(object: &ObjectDoc) -> Option<TransformData> {
    object.components.iter().find_map(|c| match &c.value {
        Value::Transform(t) if c.ty == "Transform" => Some(*t),
        _ => None,
    })
}

/// A prefab path's short name: `crate::assets::prefabs::coin_prefab::prefab` is `coin`.
fn prefab_name(path: &str) -> String {
    path.rsplit("::")
        .nth(1)
        .unwrap_or(path)
        .trim_end_matches("_prefab")
        .to_string()
}

/// The rows of a scene, in the order of the file (parents first).
pub fn rows(scene: &SceneDoc, prefabs: &HashMap<String, ObjectDoc>) -> Vec<Row> {
    fn walk(
        object: &ObjectDoc,
        depth: usize,
        prefabs: &HashMap<String, ObjectDoc>,
        out: &mut Vec<Row>,
    ) {
        let root = object.prefab.as_ref().and_then(|p| prefabs.get(p));
        // A prefab instance without a Transform of its own takes the prefab's.
        let t = transform(object)
            .or_else(|| root.and_then(transform))
            .unwrap_or_default();
        let inherited = root.map_or(0, |r| {
            r.components
                .iter()
                .filter(|c| !object.components.iter().any(|o| base(&o.ty) == base(&c.ty)))
                .count()
        });
        let name = if !object.name.is_empty() {
            object.name.clone()
        } else if let Some(prefab) = &object.prefab {
            prefab_name(prefab)
        } else {
            "Object".to_string()
        };
        let kind = match &object.prefab {
            Some(prefab) => format!("prefab {}", prefab_name(prefab)),
            None => object
                .components
                .iter()
                .find(|c| c.ty != "Transform")
                .map_or(String::new(), |c| c.ty.clone()),
        };
        out.push(Row {
            id: object.id,
            depth,
            name,
            kind,
            position: t.translation,
            rotation: t.rotation,
            scale: t.scale,
            components: object.components.len() + inherited,
        });
        for child in &object.children {
            walk(child, depth + 1, prefabs, out);
        }
    }
    let mut out = Vec::new();
    for object in &scene.objects {
        walk(object, 0, prefabs, &mut out);
    }
    out
}

/// The rows whose name or kind contains every word of `text` (any case); all of them when
/// `text` is empty.
pub fn filter(rows: Vec<Row>, text: &str) -> Vec<Row> {
    let words: Vec<String> = text.split_whitespace().map(str::to_lowercase).collect();
    if words.is_empty() {
        return rows;
    }
    rows.into_iter()
        .filter(|row| {
            let hay = format!("{} {}", row.name, row.kind).to_lowercase();
            words.iter().all(|w| hay.contains(w))
        })
        .collect()
}

/// A number as a cell shows it: no trailing zeros, at most three decimals (`-0` is `0`).
pub fn number(value: f32) -> String {
    let text = format!("{value:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    match text {
        "" | "-0" | "-" => "0".to_string(),
        other => other.to_string(),
    }
}
