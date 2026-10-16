//! The asset model: what a game authors as typed `.rs` data in its `assets/` folder, and how it
//! is spawned and shipped. `xerxes_build` turns the folder into modules and bundles; running
//! the project is `modules::project`, shipping them in bundles is `modules::bundles`, the game flow is `modules::flow`.
//!
//! - `defs`      objects, scenes, prefabs, built-in component defs
//! - `spawn`     spawning scenes and prefabs (prefab overrides by component type)
//! - `meta`      metas of external files (`Name.image|mesh|audio|video.rs`: import settings, bundle id)

pub mod defs;
pub mod meta;
pub mod spawn;

pub use defs::{
    ComponentDef, Dimension, MaterialDef, MeshDef, Object, PrefabAsset, SceneAsset, SpriteDef,
};
pub use meta::{AssetKind, AssetMeta, Filter, ImageMeta};
pub use spawn::{SceneRootObject, spawn_object, spawn_scene};
