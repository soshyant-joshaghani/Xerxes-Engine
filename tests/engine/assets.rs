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
fn actions_follow_their_bindings_from_every_device() {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyboardInput};
    use xerxes_engine::modules::input::ActionsPlugin;
    use xerxes_engine::modules::ui::TouchButton;

    let map = InputMap::new()
        .vector2("move", [Binding::wasd(), Binding::PadStick(PadStick::Left)])
        .button(
            "fire",
            [
                Binding::Mouse(MouseButton::Left),
                Binding::TouchButton("fire"),
            ],
        )
        .action("pause", [Binding::Key(KeyCode::Escape)]);
    let mut app = App::new();
    // The action manager reads the clock (a swipe is a distance over a time), so the app needs it.
    app.add_plugins((bevy::time::TimePlugin, bevy::input::InputPlugin))
        .add_plugins(ActionsPlugin(map));
    let window = app.world_mut().spawn_empty().id();

    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::KeyA,
        logical_key: Key::Character("a".into()),
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window,
    });
    app.update();
    let actions = app.world().resource::<Actions>();
    assert_eq!(actions.vector("move"), Vec2::NEG_X);
    assert!(actions.just_pressed("move"));
    assert!(!actions.pressed("fire"));
    assert_eq!(actions.device(), Device::Keyboard);

    // An on-screen button held by a finger or the mouse drives the same kind of action.
    app.world_mut()
        .spawn((TouchButton("fire"), Interaction::Pressed));
    app.update();
    let actions = app.world().resource::<Actions>();
    assert!(actions.just_pressed("fire"));
    assert_eq!(actions.vector("move"), Vec2::NEG_X, "the key is still held");
}

/// A **prefab variant** is a prefab whose root is an instance of another prefab, so three layers
/// meet on one object: the base prefab, the variant's changes, and the scene instance's. The last
/// one wins, by component type, and children add up.
#[test]
fn a_prefab_variant_layers_over_its_base() {
    fn wide_wheel() -> PrefabAsset {
        PrefabAsset {
            // The variant: the wheel, faster, with a hub of its own.
            root: Object::prefab(wheel)
                .with(Speed(5.0))
                .child(Object::new("Hub").with(Tag("hub"))),
        }
    }

    let mut world = world();
    let scene = SceneAsset {
        dimension: Dimension::D3,
        objects: vec![
            Object::prefab(wide_wheel).named("Front"),
            // The scene's own change beats the variant's.
            Object::prefab(wide_wheel).named("Rear").with(Speed(9.0)),
        ],
    };
    spawn_scene(&mut world, &scene);

    let front = named(&mut world, "Front");
    assert_eq!(front.get::<Tag>(), Some(&Tag("wheel")), "from the base");
    assert_eq!(front.get::<Speed>(), Some(&Speed(5.0)), "from the variant");
    let front = front.id();
    let rear = named(&mut world, "Rear");
    assert_eq!(rear.get::<Speed>(), Some(&Speed(9.0)), "from the instance");
    assert_eq!(rear.get::<Tag>(), Some(&Tag("wheel")), "still the base's");

    // The variant's children are there, one per instance.
    let mut hubs = world.query::<(&Tag, &ChildOf)>();
    let under: Vec<Entity> = hubs
        .iter(&world)
        .filter(|(tag, _)| **tag == Tag("hub"))
        .map(|(_, parent)| parent.parent())
        .collect();
    assert_eq!(under.len(), 2, "a hub under each instance");
    assert!(under.contains(&front));
}

/// A game that makes objects while it plays spawns the same prefabs its scene uses, through
/// `Commands`: the entity comes back at once (so the game can put its own components on it) and
/// the prefab's components and children arrive when the command runs.
#[test]
fn a_game_spawns_a_prefab_from_a_system() {
    use xerxes_engine::modules::assets::spawn;

    #[derive(Component, Debug, Clone, Copy, PartialEq)]
    struct Spawned(u32);

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_asset::<Image>();
    let road = app.world_mut().spawn(Name::new("Road")).id();

    app.add_systems(Update, move |mut commands: Commands| {
        let wheel = Object::prefab(wheel).with(Speed(4.0));
        let entity = spawn(&mut commands, wheel, Some(road));
        commands.entity(entity).insert(Spawned(7));
    });
    app.update();

    let world = app.world_mut();
    let spawned = named(world, "Wheel");
    assert_eq!(
        spawned.get::<Spawned>(),
        Some(&Spawned(7)),
        "the game's own"
    );
    assert_eq!(spawned.get::<Tag>(), Some(&Tag("wheel")), "the prefab's");
    assert_eq!(spawned.get::<Speed>(), Some(&Speed(4.0)), "the override");
    assert_eq!(spawned.get::<ChildOf>().unwrap().parent(), road);
    assert!(
        !spawned.contains::<SceneRootObject>(),
        "it belongs to the game, not to the scene's roots"
    );
}

/// A model is loaded from the bundle it ships in, by its authored path and the glTF scene's
/// label, and becomes a `SceneRoot` — so an instance's own `ModelDef` replaces the prefab's, the
/// way every other component does.
#[test]
fn a_model_def_loads_a_glb_from_its_bundle_as_a_scene() {
    use std::any::TypeId;
    use xerxes_engine::modules::assets::ComponentDef;
    use xerxes_engine::modules::bundles::asset_path;

    assert_eq!(
        asset_path("meshes/chr/knight.glb", Some("Scene0")),
        "bundle://meshes/chr/knight.glb#Scene0"
    );
    assert_eq!(asset_path("art/crate.png", None), "bundle://art/crate.png");

    let model = ModelDef::new("meshes/chr/knight.glb");
    assert_eq!(model.scene, 0, "the first glTF scene");
    assert_eq!(model.component(), TypeId::of::<SceneRoot>());
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
