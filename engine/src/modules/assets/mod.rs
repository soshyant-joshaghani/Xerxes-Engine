//! The asset model: what a game authors as typed `.rs` data in its `assets/` folder, and how it
//! is spawned and shipped. `xerxes_build` turns the folder into modules and bundles; running
//! the project is `modules::project`, shipping them in bundles is `modules::bundles`, the game flow is `modules::flow`.
//!
//! - `defs`      objects, scenes, prefabs, built-in component defs
//! - `spawn`     spawning scenes and prefabs (overrides by component type), from a system too,
//!               and [`scene_needs`](spawn::scene_needs): the files a scene loads
//! - `meta`      metas of external files (`Name.image|mesh|audio|video.rs`: import settings, bundle id)

pub mod defs;
pub mod meta;
pub mod spawn;

pub use defs::{
    AssetNeed, ComponentDef, Dimension, MaterialDef, MeshDef, ModelDef, NeedKind, Object,
    PrefabAsset, SceneAsset, SpriteDef,
};
pub use meta::{AssetKind, AssetMeta, Filter, ImageMeta};
pub use spawn::{SceneRootObject, object_needs, scene_needs, spawn, spawn_object, spawn_scene};
