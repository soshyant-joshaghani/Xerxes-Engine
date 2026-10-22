//! Scenes, prefabs and objects: the typed data in `*.scene.rs` and `*.prefab.rs`.
//!
//! A scene is 2D or 3D ([`Dimension`]); a game can have both kinds.
//!
//! An object is an entity: a name, components and children. Any Bevy component that is
//! `Clone` can be attached as it is (`Transform`, `Camera3d`, `DirectionalLight`, a game's
//! logic module). Components that need assets are written as defs (`MeshDef`, `MaterialDef`,
//! `ModelDef`, `SpriteDef`) and become real components when spawned.
//!
//! ```ignore
//! pub fn scene() -> SceneAsset {
//!     SceneAsset {
//!         dimension: Dimension::D3,
//!         objects: vec![
//!             Object::new("Light").with(DirectionalLight::default()),
//!             Object::prefab(crate::assets::prefabs::player_prefab::prefab)
//!                 .with(Transform::from_xyz(0.0, 0.5, 0.0)),
//!         ],
//!     }
//! }
//! ```

use std::any::TypeId;

use bevy::prelude::*;

/// A file a component loads, by the authored path that names it (`art/crate.png`,
/// `meshes/chr/knight.glb`). The flow has these ready **before** it spawns a scene, so nothing
/// pops into a level that has already started, and a game can ask for them ahead of time
/// (`flow::PrefetchLevel`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetNeed {
    pub path: &'static str,
    pub kind: NeedKind,
}

/// What kind of file a need is, which says how it is loaded and when it counts as ready.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeedKind {
    /// An image, with its meta's import settings.
    Image,
    /// One glTF scene of a model file: ready only once its meshes, materials and textures are.
    Model { scene: usize },
}

impl AssetNeed {
    pub fn image(path: &'static str) -> Self {
        Self {
            path,
            kind: NeedKind::Image,
        }
    }

    pub fn model(path: &'static str, scene: usize) -> Self {
        Self {
            path,
            kind: NeedKind::Model { scene },
        }
    }
}

/// Something an object carries: inserted into its entity when spawned. A component of the
/// same type inserted later replaces an earlier one, which is how prefab overrides work.
pub trait ComponentDef: Send + Sync + 'static {
    fn insert(&self, entity: &mut EntityWorldMut);
    /// The component type this inserts (for the editor and for overrides).
    fn component(&self) -> TypeId;
    /// The files this component loads. Almost no component loads one; those that do (a sprite's
    /// image, a material's texture, a model) say so here, and the flow waits for them.
    fn needs(&self) -> Vec<AssetNeed> {
        Vec::new()
    }
}

impl<T: Component + Clone> ComponentDef for T {
    fn insert(&self, entity: &mut EntityWorldMut) {
        entity.insert(self.clone());
    }

    fn component(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

/// An object in a scene or prefab.
pub struct Object {
    /// Empty on a prefab instance keeps the prefab's name.
    pub name: String,
    /// When set, the object is an instance of this prefab and `components` are its overrides.
    pub prefab: Option<fn() -> PrefabAsset>,
    pub components: Vec<Box<dyn ComponentDef>>,
    pub children: Vec<Object>,
}

impl Object {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prefab: None,
            components: Vec::new(),
            children: Vec::new(),
        }
    }

    /// An instance of a prefab. Components added with [`with`](Self::with) override the prefab's.
    pub fn prefab(prefab: fn() -> PrefabAsset) -> Self {
        Self {
            prefab: Some(prefab),
            ..Self::new("")
        }
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with(mut self, component: impl ComponentDef) -> Self {
        self.components.push(Box::new(component));
        self
    }

    pub fn child(mut self, child: Object) -> Self {
        self.children.push(child);
        self
    }
}

/// Whether a scene is 2D or 3D. Games are not: one game can have 2D and 3D scenes (a 2D
/// menu level and a 3D world, say). The editor shows a 2D scene flat (orthographic, XY).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dimension {
    D2,
    #[default]
    D3,
}

/// A scene (`assets/scenes/*.scene.rs`): 2D or 3D, and its objects.
#[derive(Default)]
pub struct SceneAsset {
    pub dimension: Dimension,
    pub objects: Vec<Object>,
}

/// A reusable object tree (`assets/prefabs/*.prefab.rs`); it may contain other prefabs.
pub struct PrefabAsset {
    pub root: Object,
}

/// A mesh generated when spawned (3D).
#[derive(Debug, Clone, PartialEq)]
pub enum MeshDef {
    Cube {
        size: f32,
    },
    /// A box of its own width, height and depth (a cube is the same on every axis).
    Cuboid {
        size: Vec3,
    },
    Sphere {
        radius: f32,
    },
    Plane {
        size: f32,
    },
}

impl ComponentDef for MeshDef {
    fn insert(&self, entity: &mut EntityWorldMut) {
        let mesh: Mesh = match *self {
            MeshDef::Cube { size } => Cuboid::from_length(size).into(),
            MeshDef::Cuboid { size } => Cuboid::from_size(size).into(),
            MeshDef::Sphere { radius } => Sphere::new(radius).into(),
            MeshDef::Plane { size } => Plane3d::default().mesh().size(size, size).into(),
        };
        let handle = entity.world_scope(|world| world.resource_mut::<Assets<Mesh>>().add(mesh));
        entity.insert(Mesh3d(handle));
    }

    fn component(&self) -> TypeId {
        TypeId::of::<Mesh3d>()
    }
}

/// A model from a bundle (3D): a `.glb` or `.gltf` file, spawned whole — its meshes, materials
/// and textures with it. The 3D counterpart of [`SpriteDef`]; use it instead of [`MeshDef`] when
/// the shape is authored art rather than a primitive.
///
/// ```ignore
/// Object::new("Knight")
///     .with(ModelDef::new("meshes/chr/knight.glb"))
///     .with(Transform::from_xyz(0.0, 0.0, 0.0))
/// ```
///
/// A glTF model faces **+Z** with **+Y** up, so an object that keeps the identity rotation faces
/// the same way as the game's forward axis: nothing has to be turned around.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDef {
    /// Authored path of the model (`meshes/chr/knight.glb`), loaded from its bundle.
    pub model: &'static str,
    /// The glTF scene's index: `0` unless the file holds more than one scene.
    pub scene: usize,
}

impl ModelDef {
    /// The model's first (usually only) glTF scene.
    pub fn new(model: &'static str) -> Self {
        Self { model, scene: 0 }
    }
}

impl ComponentDef for ModelDef {
    fn insert(&self, entity: &mut EntityWorldMut) {
        let handle = entity.world_scope(|world| {
            crate::modules::bundles::load_model(world, self.model, self.scene)
        });
        entity.insert(SceneRoot(handle));
    }

    fn component(&self) -> TypeId {
        TypeId::of::<SceneRoot>()
    }

    fn needs(&self) -> Vec<AssetNeed> {
        vec![AssetNeed::model(self.model, self.scene)]
    }
}

/// A standard material (3D): a color, optionally a texture from a bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialDef {
    pub color: Color,
    /// Authored path of an image (`art/crate.png`), loaded from its bundle.
    pub texture: Option<&'static str>,
}

impl ComponentDef for MaterialDef {
    fn insert(&self, entity: &mut EntityWorldMut) {
        let handle = entity.world_scope(|world| {
            let texture = self
                .texture
                .map(|path| crate::modules::bundles::load_image(world, path));
            world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color: self.color,
                    base_color_texture: texture,
                    ..default()
                })
        });
        entity.insert(MeshMaterial3d(handle));
    }

    fn component(&self) -> TypeId {
        TypeId::of::<MeshMaterial3d<StandardMaterial>>()
    }

    fn needs(&self) -> Vec<AssetNeed> {
        self.texture.map(AssetNeed::image).into_iter().collect()
    }
}

/// A sprite (2D): an image from a bundle and/or a color, optionally a size.
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteDef {
    pub image: Option<&'static str>,
    pub color: Color,
    pub size: Option<Vec2>,
}

impl ComponentDef for SpriteDef {
    fn insert(&self, entity: &mut EntityWorldMut) {
        let image = self
            .image
            .map(|path| {
                entity.world_scope(|world| crate::modules::bundles::load_image(world, path))
            })
            .unwrap_or_default();
        entity.insert(Sprite {
            image,
            color: self.color,
            custom_size: self.size,
            ..default()
        });
    }

    fn component(&self) -> TypeId {
        TypeId::of::<Sprite>()
    }

    fn needs(&self) -> Vec<AssetNeed> {
        self.image.map(AssetNeed::image).into_iter().collect()
    }
}
