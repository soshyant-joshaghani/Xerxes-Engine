//! The editor's `.rs` asset codec, on the real template files.

use std::path::Path;

use xerxes_engine::editor::project::codec::{
    Field, ObjectDoc, SceneDoc, TransformData, Value, parse_prefab, parse_scene, write_prefab,
    write_scene,
};

fn read(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(path)).unwrap()
}

#[test]
fn reads_a_real_scene() {
    let doc = parse_scene(&read("tests/engine/fixtures/main.scene.rs")).unwrap();
    let names: Vec<&str> = doc.objects.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Camera",
            "Sun",
            "Ground",
            "",
            "Crate",
            "Bonus Crate",
            "Goal",
            "HUD"
        ]
    );

    // Ids are unique, in document order.
    let ids: Vec<u32> = doc.all().iter().map(|o| o.id).collect();
    assert_eq!(ids, (0..ids.len() as u32).collect::<Vec<_>>());

    // A prefab instance keeps its prefab path and override.
    let player = &doc.objects[3];
    assert_eq!(
        player.prefab.as_deref(),
        Some("crate::assets::prefabs::player_prefab::prefab")
    );
    assert_eq!(player.transform().unwrap().translation, [0.0, 0.5, 0.0]);

    // Struct and enum-variant components with editable fields.
    let crate_ = doc.objects.iter().find(|o| o.name == "Crate").unwrap();
    let mesh = crate_.component("MeshDef").unwrap();
    assert_eq!(mesh.ty, "MeshDef::Cube");
    assert_eq!(
        mesh.value,
        Value::Struct {
            fields: vec![("size".into(), Field::Num(1.0))],
            rest: false
        }
    );
    assert!(mesh.original.is_some(), "read components keep their source");
    let Value::Struct { fields, .. } = &crate_.component("MaterialDef").unwrap().value else {
        panic!()
    };
    assert_eq!(
        fields[0],
        ("color".into(), Field::Color([1.0, 1.0, 1.0, 1.0]))
    );
    assert_eq!(
        fields[1],
        (
            "texture".into(),
            Field::OptStr(Some("art/checker.png".into()))
        )
    );

    // `Camera3d::default()` is a unit component; a vec3 field reads as numbers.
    let camera = &doc.objects[0];
    assert_eq!(
        camera.component("Camera3d").unwrap().value,
        Value::Unit("Camera3d::default()".into())
    );
    let Value::Struct { fields, .. } = &camera.component("CameraController3d").unwrap().value
    else {
        panic!()
    };
    assert_eq!(fields[0], ("offset".into(), Field::Vec3([0.0, 9.0, 12.0])));

    // `from_xyz(..).looking_at(..)` evaluates; `..default()` is kept as `rest`.
    let sun = &doc.objects[1];
    let t = sun.transform().unwrap();
    assert_eq!(t.translation, [4.0, 10.0, 6.0]);
    assert!(t.rotation != [0.0; 3], "looking_at gives a rotation");
    let Value::Struct { fields, rest } = &sun.component("DirectionalLight").unwrap().value else {
        panic!()
    };
    assert_eq!(fields[0], ("shadows_enabled".into(), Field::Bool(true)));
    assert!(*rest);
}

#[test]
fn reads_prefabs_and_engine_scene_templates() {
    let player = parse_prefab(&read("tests/engine/fixtures/player.prefab.rs")).unwrap();
    assert_eq!(player.name, "Player");
    assert!(
        player
            .component("CameraTarget3d")
            .is_some_and(|c| matches!(c.value, Value::Unit(_)))
    );
    let Value::Struct { fields, .. } = &player.component("CharacterController3d").unwrap().value
    else {
        panic!()
    };
    assert_eq!(
        fields,
        &[
            ("speed".into(), Field::Num(6.0)),
            ("bounds".into(), Field::Num(9.5))
        ]
    );

    for path in [
        "engine/src/editor/project/starters/3d.scene.rs",
        "engine/src/editor/project/starters/2d.scene.rs",
    ] {
        let doc = parse_scene(&read(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert!(!doc.objects.is_empty());
    }
}

#[test]
fn keeps_what_it_cannot_edit_and_reports_what_it_cannot_read() {
    let doc = parse_scene(
        r#"pub fn scene() -> SceneAsset {
            SceneAsset { objects: vec![
                Object::new("A").with(Weird::build(1, |x| x + 1)).child(Object::new("B").with(Speed(-2)))
            ] }
        }"#,
    )
    .unwrap();
    let a = &doc.objects[0];
    assert_eq!(a.components[0].ty, "Weird");
    assert!(matches!(&a.components[0].value, Value::Expr(src) if src.contains("build")));
    assert_eq!(a.children[0].name, "B");

    assert!(parse_scene("pub fn scene() -> SceneAsset { make() }").is_err());
    assert!(parse_scene("fn nope(").is_err());
    assert!(
        parse_scene("pub fn scene() -> SceneAsset { SceneAsset { objects: vec![spawn()] } }")
            .is_err()
    );
}

/// What a document means, without the source text it was read from.
fn meaning(mut doc: SceneDoc) -> SceneDoc {
    fn strip(o: &mut ObjectDoc) {
        for c in &mut o.components {
            c.original = None;
        }
        o.children.iter_mut().for_each(strip);
    }
    doc.objects.iter_mut().for_each(strip);
    doc
}

fn scenes() -> Vec<&'static str> {
    vec![
        "tests/engine/fixtures/main.scene.rs",
        "tests/engine/fixtures/sandbox.scene.rs",
        "engine/src/editor/project/starters/3d.scene.rs",
        "engine/src/editor/project/starters/2d.scene.rs",
    ]
}

#[test]
fn writing_a_scene_keeps_its_meaning() {
    for path in scenes() {
        let src = read(path);
        let doc = parse_scene(&src).unwrap();
        let written = write_scene(&src, &doc).unwrap();
        assert_eq!(
            meaning(parse_scene(&written).unwrap()),
            meaning(doc.clone()),
            "{path}"
        );

        // Also when every component is regenerated from the document (as after edits).
        let regenerated = meaning(doc.clone());
        let written = write_scene(&src, &regenerated).unwrap();
        assert_eq!(
            meaning(parse_scene(&written).unwrap()),
            regenerated,
            "{path} (regenerated)
{written}"
        );
        assert!(
            written.contains("use xerxes_engine::prelude::*;"),
            "keeps the use lines"
        );
    }
    let prefab_path = "tests/engine/fixtures/player.prefab.rs";
    let src = read(prefab_path);
    let root = parse_prefab(&src).unwrap();
    assert_eq!(
        parse_prefab(&write_prefab(&src, &root).unwrap())
            .unwrap()
            .name,
        "Player"
    );
}

#[test]
fn an_edit_changes_only_what_was_edited() {
    let src = read("tests/engine/fixtures/main.scene.rs");
    let mut doc = parse_scene(&src).unwrap();
    let crate_ = doc.objects.iter_mut().find(|o| o.name == "Crate").unwrap();
    let transform = crate_
        .components
        .iter_mut()
        .find(|c| c.ty == "Transform")
        .unwrap();
    transform.value = Value::Transform(TransformData {
        translation: [5.0, 0.5, -2.0],
        rotation: [0.0, 45.0, 0.0],
        scale: [2.0, 2.0, 2.0],
    });
    transform.original = None;

    let written = write_scene(&src, &doc).unwrap();
    let back = parse_scene(&written).unwrap();
    let t = back
        .objects
        .iter()
        .find(|o| o.name == "Crate")
        .unwrap()
        .transform()
        .unwrap();
    assert_eq!(t.translation, [5.0, 0.5, -2.0]);
    assert_eq!(
        t.rotation,
        [0.0, 45.0, 0.0],
        "rotation is written and read in degrees"
    );
    assert_eq!(t.scale, [2.0, 2.0, 2.0]);
    // Untouched components come back exactly as they were read.
    let sun = |d: &SceneDoc| {
        d.objects
            .iter()
            .find(|o| o.name == "Sun")
            .unwrap()
            .transform()
            .unwrap()
    };
    assert_eq!(sun(&back), sun(&doc));
    assert!(written.contains("BundleTexture"), "logic modules survive");
}

#[test]
fn integers_stay_integers_and_unknown_code_survives() {
    let src = r#"use x::*;
pub fn scene() -> SceneAsset {
    SceneAsset { objects: vec![Object::new("A").with(Spawner { count: 3, rate: 1.5 }).with(Weird::build(1, |x| x + 1))] }
}
"#;
    let doc = meaning(parse_scene(src).unwrap());
    let written = write_scene(src, &doc).unwrap();
    assert!(written.contains("count: 3"), "{written}");
    assert!(written.contains("rate: 1.5"), "{written}");
    assert!(written.contains("Weird::build"), "{written}");
    assert!(syn::parse_file(&written).is_ok());
}

#[test]
fn scenes_are_2d_or_3d() {
    use xerxes_engine::editor::project::codec::Dimension;
    let flat = read("engine/src/editor/project/starters/2d.scene.rs");
    let doc = parse_scene(&flat).unwrap();
    assert_eq!(doc.dimension, Dimension::D2);
    assert_eq!(
        parse_scene(&write_scene(&flat, &doc).unwrap())
            .unwrap()
            .dimension,
        Dimension::D2,
        "kept on save"
    );
    assert_eq!(
        parse_scene(&read("engine/src/editor/project/starters/3d.scene.rs"))
            .unwrap()
            .dimension,
        Dimension::D3
    );
    // Older scenes without `dimension` are 3D.
    let old = "pub fn scene() -> SceneAsset { SceneAsset { objects: vec![] } }";
    assert_eq!(parse_scene(old).unwrap().dimension, Dimension::D3);
}

#[test]
fn new_scenes_start_from_the_engine_templates() {
    use xerxes_engine::editor::project::codec::Dimension;
    use xerxes_engine::editor::project::store::new_scene_source;
    for dimension in [Dimension::D2, Dimension::D3] {
        let src = new_scene_source("level_2", dimension);
        assert!(
            src.starts_with("//! Scene `level_2`, started as an empty "),
            "{src}"
        );
        assert_eq!(parse_scene(&src).unwrap().dimension, dimension);
    }
}

#[test]
fn the_editor_starts_projects_from_its_own_starter() {
    use xerxes_engine::editor::project::store::starters;
    let starters = starters();
    // The engine's built-in starter is always first, with no template game needed.
    let starter = starters.first().expect("a starter");
    assert_eq!(
        (starter.name.as_str(), starter.title.as_str()),
        ("starter", "Starter")
    );
    assert!(
        starter
            .description
            .as_deref()
            .unwrap()
            .starts_with("An empty game")
    );
}

#[test]
fn project_and_scene_names_are_checked_as_typed() {
    use xerxes_engine::editor::project::names::{
        normalize, normalize_scene, project_problem, scene_problem, unique,
    };
    assert_eq!(normalize("My Game_2"), "my-game-2");
    assert_eq!(normalize_scene("Boss Room-1"), "boss_room_1");
    assert_eq!(
        unique("my-game", '-', &["my-game", "my-game-2"]),
        "my-game-3"
    );
    assert_eq!(unique("level", '_', &[]), "level");
    assert!(project_problem("my-game", &["my-game"], &[]).is_some());
    assert!(project_problem("my-game", &[], &[]).is_none());
    assert!(scene_problem("main", &["main"]).is_some());
    assert!(scene_problem("level-2", &[]).is_some());
    assert!(scene_problem("level_2", &["main"]).is_none());
}
