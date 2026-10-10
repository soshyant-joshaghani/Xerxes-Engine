//! What the Project browser shows, Unity style: one entry per asset. An external file and its
//! typed meta (`crate.png` + `crate.image.rs`) appear once, as the file; the meta is hidden and
//! moves with it. The type of every entry comes from its name (`Name.Type.rs`, see
//! `xerxes_build::types`).

use xerxes_build::types::{self, AssetType};

use super::store::FileEntry;

/// One row of the browser.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The path under `assets/` (what opens, moves and deletes).
    pub path: String,
    /// The name shown: `main` for `main.scene.rs`, `crate.png` for an image.
    pub name: String,
    /// `None` for a folder.
    pub kind: Option<AssetType>,
    pub size: u64,
    /// A meta whose file is missing: shown (so it can be cleaned up), marked in the label.
    pub orphan: bool,
}

impl Entry {
    pub fn is_dir(&self) -> bool {
        self.kind.is_none()
    }

    /// The small text beside the name: the type, like Unity's browser tooltip.
    pub fn label(&self) -> String {
        match (self.kind, self.orphan) {
            (None, _) => "Folder".into(),
            (Some(kind), true) => format!("{} meta, file missing", kind.label()),
            (Some(kind), false) => kind.label().into(),
        }
    }

    /// The one-letter badge drawn on the preview tile.
    pub fn badge(&self) -> &'static str {
        match self.kind {
            None => "D",
            Some(AssetType::Project) => "*",
            Some(AssetType::Scene) => "S",
            Some(AssetType::Prefab) => "P",
            Some(AssetType::Image) => "I",
            Some(AssetType::Mesh) => "M",
            Some(AssetType::Audio) => "A",
            Some(AssetType::Video) => "V",
            Some(AssetType::Logic) => "L",
            Some(AssetType::Source) => "R",
            Some(AssetType::Other) => "?",
        }
    }
}

fn parent(path: &str) -> &str {
    path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// The shown name: typed `.rs` assets drop `.<type>.rs` (`coin.prefab.rs` → `coin`).
pub fn display_name(path: &str) -> String {
    match types::typed_name(path) {
        Some((name, _)) => name.to_string(),
        None => file_name(path).to_string(),
    }
}

/// The entries of `folder` (`""` is the root of `assets/`): metas with a file are hidden.
pub fn entries(files: &[FileEntry], folder: &str) -> Vec<Entry> {
    let paths: Vec<&str> = files
        .iter()
        .filter(|f| !f.dir)
        .map(|f| f.path.as_str())
        .collect();
    files
        .iter()
        .filter(|f| parent(&f.path) == folder)
        .filter_map(|f| {
            if f.dir {
                return Some(Entry {
                    path: f.path.clone(),
                    name: file_name(&f.path).to_string(),
                    kind: None,
                    size: 0,
                    orphan: false,
                });
            }
            let is_meta = types::is_meta(&f.path);
            let orphan = is_meta && types::external_of(&f.path, paths.iter().copied()).is_none();
            if is_meta && !orphan {
                return None;
            }
            Some(Entry {
                path: f.path.clone(),
                name: display_name(&f.path),
                kind: Some(types::type_of(&f.path)),
                size: f.size,
                orphan,
            })
        })
        .collect()
}

/// `1.2 MB`, `34 KB`, `12 B`.
/// The size a PNG file's header gives.
pub fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.len() < 24 || bytes[..8] != SIGNATURE || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let read =
        |at: usize| u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    Some((read(16), read(20)))
}

pub fn size_label(size: u64) -> String {
    match size {
        s if s >= 1024 * 1024 => format!("{:.1} MB", s as f64 / (1024.0 * 1024.0)),
        s if s >= 1024 => format!("{} KB", s / 1024),
        s => format!("{s} B"),
    }
}

/// The biggest image the browser decodes for a thumbnail.
pub const PREVIEW_LIMIT: u64 = 1024 * 1024;

/// `data:` URI of an image file for its preview (`None` for a format the browser cannot show).
pub fn image_data_uri(path: &str, bytes: &[u8]) -> Option<String> {
    let ext = file_name(path).rsplit_once('.')?.1.to_ascii_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => return None,
    };
    Some(format!("data:{mime};base64,{}", base64(bytes)))
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16
            | (*chunk.get(1).unwrap_or(&0) as u32) << 8
            | *chunk.get(2).unwrap_or(&0) as u32;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// A scene's dimension badge for its preview, read from its source (`2D` / `3D`).
pub fn scene_dimension(source: &str) -> &'static str {
    if source.contains("Dimension::D2") {
        "2D"
    } else {
        "3D"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str) -> FileEntry {
        FileEntry {
            path: path.into(),
            dir: false,
            size: 10,
        }
    }

    fn dir(path: &str) -> FileEntry {
        FileEntry {
            path: path.into(),
            dir: true,
            size: 0,
        }
    }

    #[test]
    fn a_file_and_its_meta_show_once() {
        let files = [
            dir("art"),
            file("art/crate.png"),
            file("art/crate.image.rs"),
            file("art/hero.glb"),
            file("art/hero.mesh.rs"),
            file("art/gone.audio.rs"),
            file("game.project.rs"),
            dir("scenes"),
            file("scenes/main.scene.rs"),
        ];
        let shown: Vec<_> = entries(&files, "art")
            .into_iter()
            .map(|e| (e.name, e.kind, e.orphan))
            .collect();
        assert_eq!(
            shown,
            [
                ("crate.png".to_string(), Some(AssetType::Image), false),
                ("hero.glb".to_string(), Some(AssetType::Mesh), false),
                ("gone".to_string(), Some(AssetType::Audio), true),
            ]
        );
        let root: Vec<_> = entries(&files, "")
            .into_iter()
            .map(|e| {
                let label = e.label();
                (e.name, label)
            })
            .collect();
        assert_eq!(
            root,
            [
                ("art".to_string(), "Folder".to_string()),
                ("game".to_string(), "Project settings".to_string()),
                ("scenes".to_string(), "Folder".to_string()),
            ]
        );
    }

    #[test]
    fn typed_assets_drop_their_suffix_and_say_their_type() {
        assert_eq!(display_name("scenes/main.scene.rs"), "main");
        assert_eq!(display_name("prefabs/coin.prefab.rs"), "coin");
        assert_eq!(display_name("logic/mover.rs"), "mover.rs");
        let scene = &entries(&[file("scenes/main.scene.rs")], "scenes")[0];
        assert_eq!((scene.label().as_str(), scene.badge()), ("Scene", "S"));
        assert_eq!(
            entries(&[file("logic/mover.rs")], "logic")[0].label(),
            "Logic"
        );
        assert_eq!(entries(&[file("fonts/ui.ttf")], "fonts")[0].label(), "File");
        assert_eq!(entries(&[file("a.glb"), file("a.mesh.rs")], "").len(), 1);
    }

    #[test]
    fn previews_are_data_uris() {
        assert_eq!(base64(b"Man"), "TWFu");
        assert_eq!(base64(b"Ma"), "TWE=");
        assert_eq!(base64(b"M"), "TQ==");
        assert_eq!(
            image_data_uri("art/a.PNG", b"Man").as_deref(),
            Some("data:image/png;base64,TWFu")
        );
        assert_eq!(image_data_uri("art/a.ktx2", b"x"), None);
        assert_eq!(scene_dimension("dimension: Dimension::D2,"), "2D");
        assert_eq!(scene_dimension("dimension: Dimension::D3,"), "3D");
        assert_eq!(size_label(2048), "2 KB");
        assert_eq!(size_label(12), "12 B");
    }
}
