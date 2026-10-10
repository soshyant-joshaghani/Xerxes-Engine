//! Metas: the typed `Name.Type.rs` beside every external file (like Unity's `.meta`):
//! `crate.image.rs` for `crate.png`, `hero.mesh.rs` for `hero.glb`, `theme.audio.rs`,
//! `intro.video.rs`. It names the file's kind, its import settings and the bundle it ships in.
//!
//! ```ignore
//! // assets/art/crate.image.rs
//! pub fn meta() -> AssetMeta {
//!     AssetMeta::image(ImageMeta { filter: Filter::Nearest }).bundle("core")
//! }
//! ```
//!
//! The bundle id must be a string literal: `xerxes_build` reads it at build time to pack the
//! file into `bundles/<id>.zip`.

/// The bundle a file ships in when its meta names none.
pub const DEFAULT_BUNDLE: &str = "core";

#[derive(Debug, Clone, PartialEq)]
pub struct AssetMeta {
    pub kind: AssetKind,
    pub bundle: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssetKind {
    Image(ImageMeta),
    Mesh,
    Audio,
    Video,
    Other,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImageMeta {
    pub filter: Filter,
}

/// Texture sampling: `Linear` for smooth art, `Nearest` for pixel art.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Filter {
    #[default]
    Linear,
    Nearest,
}

impl AssetMeta {
    fn new(kind: AssetKind) -> Self {
        Self {
            kind,
            bundle: DEFAULT_BUNDLE,
        }
    }

    pub fn image(meta: ImageMeta) -> Self {
        Self::new(AssetKind::Image(meta))
    }

    pub fn mesh() -> Self {
        Self::new(AssetKind::Mesh)
    }

    pub fn audio() -> Self {
        Self::new(AssetKind::Audio)
    }

    pub fn video() -> Self {
        Self::new(AssetKind::Video)
    }

    pub fn other() -> Self {
        Self::new(AssetKind::Other)
    }

    /// The bundle this file ships in (`bundles/<id>.zip`).
    pub fn bundle(mut self, id: &'static str) -> Self {
        self.bundle = id;
        self
    }
}
