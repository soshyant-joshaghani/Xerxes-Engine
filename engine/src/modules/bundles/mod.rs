//! Bundles: external files ship zipped by the bundle id in their meta (`bundles/<id>.zip`,
//! packed by `xerxes_build`). At runtime a bundle is fetched through the normal
//! `AssetServer` (HTTP on the web, a file on desktop, the APK on Android) and extracted in
//! memory into the `bundle://` asset source. Game data names assets by their authored path
//! (`art/crate.png`); [`load_image`] and friends resolve it inside the bundles.
//!
//! Ask for a bundle with [`LoadBundle`]; [`BundleReady`] says when its files can be loaded.

pub mod texture;

pub use texture::BundleTexture;

use std::collections::{HashMap, HashSet};
use std::io::{self, Cursor, Read};
use std::path::Path;

use bevy::asset::io::memory::{Dir, MemoryAssetReader};
use bevy::asset::io::{AssetSourceBuilder, Reader};
use bevy::asset::{AssetLoader, LoadContext};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

use crate::modules::assets::meta::{AssetKind, AssetMeta, Filter};

/// The asset source the bundles are extracted into.
pub const SOURCE: &str = "bundle";

/// One bundle, as listed in the generated manifest.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BundleInfo {
    pub id: &'static str,
    /// Size of the zip in bytes (for progress and download prompts).
    pub size: u64,
    pub files: &'static [&'static str],
}

/// Asks for a bundle to be downloaded and extracted (no-op if it already is or is on its way).
#[derive(Message, Debug, Clone, PartialEq)]
pub struct LoadBundle(pub &'static str);

/// A bundle's files can now be loaded.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct BundleReady(pub &'static str);

/// The bundles of the project, their state and the metas of every external file.
#[derive(Resource)]
pub struct Bundles {
    dir: Dir,
    known: &'static [BundleInfo],
    metas: HashMap<&'static str, AssetMeta>,
    loading: HashMap<&'static str, Handle<BundleZip>>,
    ready: HashSet<&'static str>,
}

impl Bundles {
    pub fn new(
        dir: Dir,
        known: &'static [BundleInfo],
        metas: HashMap<&'static str, AssetMeta>,
    ) -> Self {
        Self {
            dir,
            known,
            metas,
            loading: HashMap::new(),
            ready: HashSet::new(),
        }
    }

    pub fn is_ready(&self, id: &str) -> bool {
        self.ready.contains(id)
    }

    pub fn info(&self, id: &str) -> Option<&BundleInfo> {
        self.known.iter().find(|b| b.id == id)
    }

    /// The meta of an external file, by its authored path.
    pub fn meta(&self, path: &str) -> Option<&AssetMeta> {
        self.metas.get(path)
    }
}

/// A downloaded bundle, before extraction.
#[derive(Asset, TypePath, Debug)]
pub struct BundleZip(pub Vec<u8>);

#[derive(Default, TypePath)]
pub(crate) struct BundleZipLoader;

impl AssetLoader for BundleZipLoader {
    type Asset = BundleZip;
    type Settings = ();
    type Error = io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _: &mut LoadContext<'_>,
    ) -> Result<BundleZip, io::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(BundleZip(bytes))
    }

    fn extensions(&self) -> &[&str] {
        &["zip"]
    }
}

/// Registers `bundle://` over `dir`. Must run before the `AssetPlugin` is added.
pub(crate) fn register_source(app: &mut App, dir: &Dir) {
    let dir = dir.clone();
    app.register_asset_source(
        SOURCE,
        AssetSourceBuilder::new(move || Box::new(MemoryAssetReader { root: dir.clone() })),
    );
}

/// Unpacks a bundle zip into `dir`; returns how many files it held.
pub fn extract(dir: &Dir, zip_bytes: &[u8]) -> io::Result<usize> {
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes)).map_err(io::Error::other)?;
    let mut count = 0;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(io::Error::other)?;
        if file.is_dir() {
            continue;
        }
        let mut bytes = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut bytes)?;
        dir.insert_asset(Path::new(file.name()), bytes);
        count += 1;
    }
    Ok(count)
}

pub(crate) fn start_loads(
    mut requests: MessageReader<LoadBundle>,
    mut bundles: ResMut<Bundles>,
    server: Res<AssetServer>,
) {
    for LoadBundle(id) in requests.read() {
        if bundles.ready.contains(id) || bundles.loading.contains_key(id) {
            continue;
        }
        if bundles.info(id).is_none() {
            error!("bundle `{id}` is not in this build (no asset names it in its meta)");
            continue;
        }
        let handle = server.load(format!("bundles/{id}.zip"));
        bundles.loading.insert(id, handle);
    }
}

pub(crate) fn finish_loads(
    mut bundles: ResMut<Bundles>,
    mut zips: ResMut<Assets<BundleZip>>,
    server: Res<AssetServer>,
    mut ready: MessageWriter<BundleReady>,
) {
    let arrived: Vec<&'static str> = bundles
        .loading
        .iter()
        .filter(|(id, handle)| {
            if server.load_state(handle.id()).is_failed() {
                error!("bundle `{id}` could not be downloaded (bundles/{id}.zip)");
            }
            zips.contains(handle.id())
        })
        .map(|(id, _)| *id)
        .collect();
    for id in arrived {
        let handle = bundles.loading.remove(id).expect("loading");
        let Some(zip) = zips.remove(handle.id()) else {
            continue;
        };
        match extract(&bundles.dir, &zip.0) {
            Ok(files) => {
                info!("bundle `{id}` ready ({files} files)");
                bundles.ready.insert(id);
                ready.write(BundleReady(id));
            }
            Err(err) => error!("bundle `{id}` is not a valid zip: {err}"),
        }
    }
}

/// Loads an image by its authored path from the bundles, with its meta's import settings.
pub fn load_image(world: &World, path: &str) -> Handle<Image> {
    let filter = world
        .get_resource::<Bundles>()
        .and_then(|bundles| bundles.meta(path))
        .and_then(|meta| match &meta.kind {
            AssetKind::Image(image) => Some(image.filter),
            _ => None,
        })
        .unwrap_or_default();
    let server = world.resource::<AssetServer>();
    let path = format!("{SOURCE}://{path}");
    match filter {
        Filter::Linear => server.load(path),
        Filter::Nearest => server.load_with_settings(path, |s: &mut ImageLoaderSettings| {
            s.sampler = ImageSampler::nearest();
        }),
    }
}
