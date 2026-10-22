//! The editor's `.rs` asset codec, on the real template files.

use std::path::Path;

use xerxes_engine::editor::project::codec::{
    ConstDoc, Dimension, Field, ObjectDoc, SceneDoc, TransformData, Value, parse_prefab,
    parse_prefab_doc, parse_scene, prefab_label, write_prefab, write_prefab_doc, write_scene,
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

    // `GameCamera3D` is a unit component; a vec3 field reads as numbers.
    let camera = &doc.objects[0];
    assert_eq!(
        camera.component("GameCamera3D").unwrap().value,
        Value::Unit("GameCamera3D".into())
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

/// A prefab opens in the editor as a document of one object: the Outliner, the Properties and the
/// Scene view then work on it the way they work on a scene. Its dimension is not in the file — it
/// is read off what the prefab holds — and its palette constants come with it, so the Scene view
/// resolves the colours a UI prefab uses.
#[test]
fn a_prefab_opens_as_a_document_of_one_object() {
    // A UI prefab: flat, with the constants its colours come from.
    let src = read("games/__templates__/darius/assets/prefabs/header.prefab.rs");
    let doc = parse_prefab_doc(&src).unwrap();
    assert_eq!(doc.objects.len(), 1, "a prefab is its root object");
    let root = &doc.objects[0];
    assert_eq!(root.name, "Header");
    assert_eq!(doc.dimension, Dimension::D2, "UI is shown flat");
    assert!(root.component("HeaderBand").is_some());
    assert!(
        doc.consts.iter().any(|c| c.name == "PANEL"),
        "the file's constants come with it"
    );
    // Its toggle is a child, so entering the prefab shows the whole tree.
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.children[0].name, "Header Toggle");

    // A prefab of a model is 3D, and the model hangs on a child of its own: the game writes the
    // root's transform every frame, so the art's stays the author's to set.
    let model =
        read("games/__templates__/cyrus/assets/prefabs/games/solo/siavash/knight.prefab.rs");
    let doc = parse_prefab_doc(&model).unwrap();
    assert_eq!(
        doc.dimension,
        Dimension::D3,
        "a model is seen in perspective"
    );
    let root = &doc.objects[0];
    assert_eq!(root.name, "Knight");
    assert!(root.component("ModelDef").is_none(), "not on the root");
    let art = root
        .children
        .iter()
        .find(|child| child.name == "Model")
        .expect("the model is a child");
    assert!(art.component("ModelDef").is_some());

    // A prefab made of another prefab (the narrow menu tile) keeps the reference it is made of.
    let nested = read("games/__templates__/darius/assets/prefabs/menu_tile_narrow.prefab.rs");
    let doc = parse_prefab_doc(&nested).unwrap();
    assert_eq!(
        doc.objects[0].prefab.as_deref(),
        Some("crate::assets::prefabs::menu_tile_prefab::prefab")
    );

    // Saving a prefab document writes its root back, and nothing else changes.
    let written = write_prefab(&src, &doc_root(&parse_prefab_doc(&src).unwrap())).unwrap();
    assert_eq!(parse_prefab(&written).unwrap(), parse_prefab(&src).unwrap());
    assert!(written.contains("use xerxes_engine::prelude::*;"));
}

fn doc_root(doc: &SceneDoc) -> ObjectDoc {
    doc.objects[0].clone()
}

/// A document's constants are plain Rust the editor reads **and writes**: they can be added,
/// renamed, changed and removed, and everything else in the file stays as it was. `pub` items
/// (the project file's `LOGIC`) are not the document's constants and are never touched.
#[test]
fn a_documents_constants_are_read_and_written_back() {
    let src = read("games/__templates__/darius/assets/scenes/games/solo/qanat.scene.rs");
    let mut doc = parse_scene(&src).unwrap();
    let named = |doc: &SceneDoc, name: &str| {
        doc.consts
            .iter()
            .find(|c| c.name == name)
            .cloned()
            .unwrap_or_else(|| panic!("no constant {name}"))
    };
    let grass = named(&doc, "GRASS");
    assert_eq!(grass.ty, "Color");
    assert!(grass.value.starts_with("Color :: srgb") || grass.value.starts_with("Color::srgb"));

    // Change one, add one, and drop one.
    let before = doc.consts.len();
    let index = doc.consts.iter().position(|c| c.name == "GRASS").unwrap();
    doc.consts[index].value = "Color::srgb(0.0, 1.0, 0.0)".into();
    doc.consts.push(ConstDoc::new("EXTRA", "f32", "12.5"));
    let dropped = doc.consts.iter().position(|c| c.name == "CURSOR");
    if let Some(dropped) = dropped {
        doc.consts.remove(dropped);
    }

    let written = write_scene(&src, &doc).unwrap();
    let again = parse_scene(&written).unwrap();
    assert_eq!(
        named(&again, "GRASS").value.replace(' ', ""),
        "Color::srgb(0.0,1.0,0.0)"
    );
    let extra = named(&again, "EXTRA");
    assert_eq!((extra.ty.as_str(), extra.value.as_str()), ("f32", "12.5"));
    assert_eq!(again.consts.len(), before + 1 - dropped.map_or(0, |_| 1));
    assert!(!again.consts.iter().any(|c| c.name == "CURSOR"));
    // The rest of the file is untouched.
    assert!(written.contains("use xerxes_engine::prelude::*;"));
    assert_eq!(again.objects.len(), doc.objects.len());
    assert!(written.contains("const EXTRA: f32 = 12.5;"), "{written}");

    // A `pub const` is the file's interface, not its palette: it is left alone.
    let with_pub = "pub const LOGIC: &[u8] = &[];\nconst A: f32 = 1.0;\n".to_string()
        + "pub fn prefab() -> PrefabAsset { PrefabAsset { root: Object::new(\"X\") } }";
    let doc = parse_prefab_doc(&with_pub).unwrap();
    assert_eq!(doc.consts.len(), 1, "only the file's own");
    assert_eq!(doc.consts[0].name, "A");
    let written = write_prefab_doc(&with_pub, &doc.objects[0], &doc.consts).unwrap();
    assert!(written.contains("pub const LOGIC"), "{written}");
}

/// A **prefab variant** is a prefab whose root is an instance of another prefab: that prefab with
/// changes of its own, in plain Rust. The editor reads the base it varies, and writing it back
/// keeps it.
#[test]
fn a_prefab_variant_names_the_prefab_it_varies() {
    let src = read("games/__templates__/darius/assets/prefabs/menu_tile_narrow.prefab.rs");
    let doc = parse_prefab_doc(&src).unwrap();
    let root = &doc.objects[0];
    let base = root.prefab.as_deref().expect("a variant has a base");
    assert_eq!(base, "crate::assets::prefabs::menu_tile_prefab::prefab");
    assert_eq!(prefab_label(base), "menu_tile");
    // Its own change: a narrower box than the tile it varies.
    assert!(root.component("Node").is_some());

    let written = write_prefab(&src, root).unwrap();
    assert_eq!(parse_prefab(&written).unwrap(), *root);
}

/// The editor starts a prefab, and a variant, from its own templates, and both parse as prefabs.
#[test]
fn new_prefabs_and_variants_start_from_the_editor_templates() {
    use xerxes_engine::editor::project::store::{new_prefab_source, new_variant_source};

    let source = new_prefab_source("menu_tile");
    let root = parse_prefab(&source).unwrap_or_else(|e| panic!("{e}\n{source}"));
    assert_eq!(root.name, "Menu Tile", "named after its file");
    assert!(root.prefab.is_none());
    assert!(source.contains("//! Prefab `menu_tile`"));

    let base = "crate::assets::prefabs::menu_tile_prefab::prefab";
    let source = new_variant_source("menu_tile_wide", base, "menu_tile");
    let root = parse_prefab(&source).unwrap_or_else(|e| panic!("{e}\n{source}"));
    assert_eq!(root.prefab.as_deref(), Some(base), "a variant of it");
    assert!(source.contains("**variant** of `menu_tile`"));
}
