//! The asset model, headless: scenes and prefabs spawn as entities, prefab overrides replace
//! components by type, nested prefabs and children keep their structure, input actions map
//! bindings, and a bundle zip extracts into the in-memory source.

use std::io::{Cursor, Write};
use std::path::Path;

use bevy::asset::io::memory::Dir;
use bevy::prelude::*;
use xerxes_engine::modules::bundles::extract;
use xerxes_engine::prelude::*;

#[derive(Component, Clone, Debug, PartialEq)]
struct Speed(f32);

#[derive(Component, Clone, Debug, PartialEq)]
struct Tag(&'static str);

fn wheel() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Wheel").with(Tag("wheel")).with(Speed(1.0)),
    }
}

fn car() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Car")
            .with(Speed(10.0))
            .child(Object::prefab(wheel))
            .child(Object::prefab(wheel).named("Spare").with(Speed(0.0))),
    }
}

fn world() -> World {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_asset::<Image>();
    std::mem::take(app.world_mut())
}

fn named<'w>(world: &'w mut World, name: &str) -> EntityRef<'w> {
    let mut query = world.query::<(Entity, &Name)>();
    let entity = query
        .iter(world)
        .find(|(_, n)| n.as_str() == name)
        .unwrap_or_else(|| panic!("no object named {name}"))
        .0;
    world.entity(entity)
}

#[test]
fn spawns_objects_with_components_and_children() {
    let mut world = world();
    let scene = SceneAsset {
        dimension: Dimension::D3,
        objects: vec![
            Object::new("Ground")
                .with(Transform::from_xyz(0.0, -1.0, 0.0))
                .with(MeshDef::Plane { size: 4.0 }),
            Object::new("Empty").child(Object::new("Child").with(Tag("c"))),
        ],
    };
    let roots = spawn_scene(&mut world, &scene);
    assert_eq!(roots.len(), 2);

    let ground = named(&mut world, "Ground");
    assert_eq!(ground.get::<Transform>().unwrap().translation.y, -1.0);
    assert!(ground.contains::<Mesh3d>(), "MeshDef becomes a Mesh3d");
    assert!(ground.contains::<SceneRootObject>());

    let empty = named(&mut world, "Empty").id();
    let child = named(&mut world, "Child");
    assert_eq!(child.get::<ChildOf>().unwrap().parent(), empty);
    assert_eq!(child.get::<Tag>(), Some(&Tag("c")));
    assert!(!child.contains::<SceneRootObject>());
}

#[test]
fn prefab_instances_override_by_component_type() {
    let mut world = world();
    let scene = SceneAsset {
        dimension: Dimension::D3,
        objects: vec![Object::prefab(car).named("Red Car").with(Speed(25.0))],
    };
    spawn_scene(&mut world, &scene);

    // The instance's own component replaces the prefab's; the name override applies.
    let car = named(&mut world, "Red Car");
    assert_eq!(car.get::<Speed>(), Some(&Speed(25.0)));
    let car = car.id();

    // Nested prefabs keep the prefab's name unless renamed, and their own overrides.
    let wheel = named(&mut world, "Wheel");
    assert_eq!(wheel.get::<Speed>(), Some(&Speed(1.0)));
    assert_eq!(wheel.get::<ChildOf>().unwrap().parent(), car);
    let spare = named(&mut world, "Spare");
    assert_eq!(spare.get::<Speed>(), Some(&Speed(0.0)));
    assert_eq!(spare.get::<Tag>(), Some(&Tag("wheel")));
}

#[test]
fn actions_follow_their_bindings() {
    let map = InputMap::new()
        .action(
            "left",
            [
                Binding::Key(KeyCode::KeyA),
                Binding::Key(KeyCode::ArrowLeft),
            ],
        )
        .action("right", [Binding::Key(KeyCode::KeyD)])
        .action("fire", [Binding::Mouse(MouseButton::Left)]);
    let mut actions = Actions::new(map);
    let down = [
        Binding::Key(KeyCode::ArrowLeft),
        Binding::Mouse(MouseButton::Left),
    ];
    actions.update(
        |b| down.contains(&b),
        |b| b == Binding::Mouse(MouseButton::Left),
        |_| false,
    );

    assert!(actions.pressed("left"));
    assert!(!actions.pressed("right"));
    assert_eq!(actions.axis("left", "right"), -1.0);
    assert!(actions.just_pressed("fire"));
    assert!(!actions.just_pressed("left"));
}

#[test]
fn metas_name_their_bundle() {
    let meta = AssetMeta::image(ImageMeta {
        filter: Filter::Nearest,
    })
    .bundle("level-2");
    assert_eq!(meta.bundle, "level-2");
    assert_eq!(AssetMeta::audio().bundle, "core");
    assert_eq!(AssetMeta::mesh().kind, AssetKind::Mesh);
}

#[test]
fn bundle_zips_extract_into_the_memory_source() {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (path, body) in [
        ("art/a.png", b"png bytes".as_slice()),
        ("audio/b.ogg", b"ogg".as_slice()),
    ] {
        zip.start_file(path, options).unwrap();
        zip.write_all(body).unwrap();
    }
    let bytes = zip.finish().unwrap().into_inner();

    let dir = Dir::default();
    assert_eq!(extract(&dir, &bytes).unwrap(), 2);
    let asset = dir.get_asset(Path::new("art/a.png")).expect("extracted");
    assert_eq!(&*asset.value(), b"png bytes");
    assert!(dir.get_asset(Path::new("audio/b.ogg")).is_some());
    assert!(extract(&dir, b"not a zip").is_err());
}
