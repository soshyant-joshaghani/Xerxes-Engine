//! The Spreadsheet's rows: the scene flattened, what each row says, the filter, the numbers.

use std::collections::HashMap;

use xerxes_engine::editor::project::codec::{
    ComponentDoc, ObjectDoc, SceneDoc, TransformData, Value, parse_scene,
};
use xerxes_engine::editor::project::table::{filter, number, rows};

fn starter_scene() -> SceneDoc {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("starter/assets/scenes/main.scene.rs"),
    )
    .unwrap();
    parse_scene(&source).unwrap()
}

fn object(id: u32, name: &str, at: [f32; 3]) -> ObjectDoc {
    ObjectDoc {
        id,
        name: name.into(),
        components: vec![ComponentDoc::new(
            "Transform",
            Value::Transform(TransformData {
                translation: at,
                ..Default::default()
            }),
        )],
        ..Default::default()
    }
}

#[test]
fn the_starter_scene_is_three_rows() {
    let scene = starter_scene();
    let all = rows(&scene, &HashMap::new());
    let names: Vec<_> = all.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["Camera", "Sun", "Ground"]);
    assert!(all.iter().all(|r| r.depth == 0));
    // The camera is not at the origin; the ground says what it is.
    assert_ne!(all[0].position, [0.0; 3]);
    assert!(all[2].kind.contains("Plane"), "{:?}", all[2].kind);
    assert!(all.iter().all(|r| r.components >= 2));
    // Ids are the scene's own, so a click selects the right object.
    for row in &all {
        assert!(scene.find(row.id).is_some());
    }
}

#[test]
fn children_follow_their_parent_with_a_depth_and_prefabs_lend_what_they_lack() {
    let mut parent = object(1, "Parent", [1.0, 2.0, 3.0]);
    let mut child = object(2, "", [0.0; 3]);
    child.components.clear();
    child.prefab = Some("crate::assets::prefabs::coin_prefab::prefab".into());
    parent.children.push(child);
    let scene = SceneDoc {
        objects: vec![parent, object(3, "Other", [9.0, 0.0, 0.0])],
        ..Default::default()
    };
    // The prefab has a Transform and a mesh; the instance has neither of its own.
    let mut coin = object(0, "Coin", [5.0, 5.0, 5.0]);
    coin.components.push(ComponentDoc::new(
        "MeshDef::Cube",
        Value::Unit("MeshDef::Cube".into()),
    ));
    let prefabs = HashMap::from([(
        "crate::assets::prefabs::coin_prefab::prefab".to_string(),
        coin,
    )]);

    let all = rows(&scene, &prefabs);
    let order: Vec<_> = all.iter().map(|r| (r.id, r.depth)).collect();
    assert_eq!(order, [(1, 0), (2, 1), (3, 0)]);
    // The instance has no name of its own: its prefab's; no Transform: the prefab's.
    assert_eq!(all[1].name, "coin");
    assert_eq!(all[1].kind, "prefab coin");
    assert_eq!(all[1].position, [5.0, 5.0, 5.0]);
    assert_eq!(all[1].components, 2, "both come from the prefab");
}

#[test]
fn the_filter_matches_every_word_in_the_name_or_the_kind() {
    let scene = starter_scene();
    let all = rows(&scene, &HashMap::new());
    assert_eq!(filter(all.clone(), "").len(), 3);
    assert_eq!(filter(all.clone(), "  ").len(), 3);
    let sun = filter(all.clone(), "SUN");
    assert_eq!(sun.len(), 1);
    assert_eq!(sun[0].name, "Sun");
    assert!(
        filter(all.clone(), "plane")
            .iter()
            .any(|r| r.name == "Ground")
    );
    assert!(
        filter(all.clone(), "sun plane").is_empty(),
        "every word must match"
    );
    assert!(filter(all, "nothing like this").is_empty());
}

#[test]
fn numbers_are_short() {
    assert_eq!(number(0.0), "0");
    assert_eq!(number(-0.0), "0");
    assert_eq!(number(1.0), "1");
    assert_eq!(number(1.5), "1.5");
    assert_eq!(number(0.1234567), "0.123");
    assert_eq!(number(-2.5), "-2.5");
    assert_eq!(number(10.0), "10");
    assert_eq!(number(100.0), "100");
    assert_eq!(number(-0.0004), "0");
}
