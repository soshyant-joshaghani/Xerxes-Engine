//! Asset types: every known asset is named `Name.Type.rs` (or, for an external file, sits beside
//! its `Name.Type.rs` meta). One rule, shared by the build, the backend and the editor.
//!
//! | File | Type |
//! |------|------|
//! | `main.scene.rs` | scene |
//! | `coin.prefab.rs` | prefab |
//! | `my-game.project.rs` | project settings (one, in the root of `assets/`) |
//! | `crate.png` + `crate.image.rs` | image (the meta is its config) |
//! | `hero.glb` + `hero.mesh.rs` | mesh |
//! | `theme.ogg` + `theme.audio.rs` | audio |
//! | `intro.mp4` + `intro.video.rs` | video |
//! | `logic/mover.rs` | logic module |
//!
//! Like Unity, the browser shows an external file and its meta once: the meta is hidden and moves,
//! renames and deletes with its file.

/// What an asset is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Project,
    Scene,
    Prefab,
    Image,
    Mesh,
    Audio,
    Video,
    /// A logic module (`logic/*.rs`).
    Logic,
    /// Any other `.rs` file.
    Source,
    /// An external file of a type that has no meta (a font, a data file, ...).
    Other,
}

pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tga", "ktx2"];
pub const MESH_EXTENSIONS: &[&str] = &["glb", "gltf"];
pub const AUDIO_EXTENSIONS: &[&str] = &["ogg", "wav", "mp3", "flac"];
pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "webm", "mov"];

impl AssetType {
    /// The `Type` in `Name.Type.rs`.
    pub fn tag(self) -> &'static str {
        match self {
            AssetType::Project => "project",
            AssetType::Scene => "scene",
            AssetType::Prefab => "prefab",
            AssetType::Image => "image",
            AssetType::Mesh => "mesh",
            AssetType::Audio => "audio",
            AssetType::Video => "video",
            AssetType::Logic => "logic",
            AssetType::Source => "rust",
            AssetType::Other => "file",
        }
    }

    /// The name the browser shows.
    pub fn label(self) -> &'static str {
        match self {
            AssetType::Project => "Project settings",
            AssetType::Scene => "Scene",
            AssetType::Prefab => "Prefab",
            AssetType::Image => "Image",
            AssetType::Mesh => "Mesh",
            AssetType::Audio => "Audio",
            AssetType::Video => "Video",
            AssetType::Logic => "Logic",
            AssetType::Source => "Rust",
            AssetType::Other => "File",
        }
    }

    /// `true` for the types an external file has (its meta is `Name.<tag>.rs`).
    pub fn is_external(self) -> bool {
        matches!(
            self,
            AssetType::Image | AssetType::Mesh | AssetType::Audio | AssetType::Video
        )
    }

    fn from_tag(tag: &str) -> Option<AssetType> {
        Some(match tag {
            "project" => AssetType::Project,
            "scene" => AssetType::Scene,
            "prefab" => AssetType::Prefab,
            "image" => AssetType::Image,
            "mesh" => AssetType::Mesh,
            "audio" => AssetType::Audio,
            "video" => AssetType::Video,
            _ => return None,
        })
    }

    fn extensions(self) -> &'static [&'static str] {
        match self {
            AssetType::Image => IMAGE_EXTENSIONS,
            AssetType::Mesh => MESH_EXTENSIONS,
            AssetType::Audio => AUDIO_EXTENSIONS,
            AssetType::Video => VIDEO_EXTENSIONS,
            _ => &[],
        }
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn parent(path: &str) -> &str {
    path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
}

fn join(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    }
}

/// The type of an external file by its extension (`None`: no known type, so no meta).
pub fn external_type(path: &str) -> Option<AssetType> {
    let ext = file_name(path).rsplit_once('.')?.1.to_ascii_lowercase();
    [
        AssetType::Image,
        AssetType::Mesh,
        AssetType::Audio,
        AssetType::Video,
    ]
    .into_iter()
    .find(|t| t.extensions().contains(&ext.as_str()))
}

/// `(name, type)` for a typed `.rs` file: `main.scene.rs` → `("main", Scene)`. Metas count
/// (`crate.image.rs` → `("crate", Image)`); a plain `.rs` has no type here.
pub fn typed_name(path: &str) -> Option<(&str, AssetType)> {
    let stem = file_name(path).strip_suffix(".rs")?;
    let (name, tag) = stem.rsplit_once('.')?;
    if name.is_empty() {
        return None;
    }
    Some((name, AssetType::from_tag(tag)?))
}

/// Where the meta of an external file lives: `art/crate.png` → `art/crate.image.rs`.
pub fn meta_path(external: &str) -> Option<String> {
    let kind = external_type(external)?;
    let name = file_name(external);
    let stem = name.rsplit_once('.')?.0;
    Some(join(parent(external), &format!("{stem}.{}.rs", kind.tag())))
}

/// `true` when `path` is a meta (`Name.image|mesh|audio|video.rs`).
pub fn is_meta(path: &str) -> bool {
    typed_name(path).is_some_and(|(_, t)| t.is_external())
}

/// The external file a meta belongs to, among `files` (every path of the project):
/// `art/crate.image.rs` → `art/crate.png`, when that file exists.
pub fn external_of<'a, I>(meta: &str, files: I) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let (name, kind) = typed_name(meta)?;
    if !kind.is_external() {
        return None;
    }
    let dir = parent(meta);
    files
        .into_iter()
        .find(|f| {
            parent(f) == dir
                && file_name(f).rsplit_once('.').is_some_and(|(stem, ext)| {
                    stem == name
                        && kind
                            .extensions()
                            .contains(&ext.to_ascii_lowercase().as_str())
                })
        })
        .map(str::to_string)
}

/// The type of any path under `assets/`. A meta reports its file's type.
pub fn type_of(path: &str) -> AssetType {
    if path.ends_with(".rs") {
        if let Some((_, kind)) = typed_name(path) {
            return kind;
        }
        if path.starts_with("logic/") {
            return AssetType::Logic;
        }
        return AssetType::Source;
    }
    external_type(path).unwrap_or(AssetType::Other)
}

/// The project settings file: the one `Name.project.rs` in the root of `assets/`.
pub fn project_file<'a, I>(files: I) -> Result<&'a str, String>
where
    I: IntoIterator<Item = &'a str>,
{
    let found: Vec<&str> = files
        .into_iter()
        .filter(|f| !f.contains('/') && typed_name(f).is_some_and(|(_, t)| t == AssetType::Project))
        .collect();
    match found.as_slice() {
        [one] => Ok(one),
        [] => Err("assets/ has no project settings file (`<name>.project.rs`)".into()),
        many => Err(format!(
            "assets/ has {} project settings files ({}); keep one",
            many.len(),
            many.join(", ")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_carry_their_type() {
        assert_eq!(
            typed_name("scenes/main.scene.rs"),
            Some(("main", AssetType::Scene))
        );
        assert_eq!(
            typed_name("my-game.project.rs"),
            Some(("my-game", AssetType::Project))
        );
        assert_eq!(
            typed_name("art/crate.image.rs"),
            Some(("crate", AssetType::Image))
        );
        assert_eq!(typed_name("logic/mover.rs"), None);
        assert_eq!(typed_name(".scene.rs"), None);
        assert_eq!(type_of("logic/mover.rs"), AssetType::Logic);
        assert_eq!(type_of("util.rs"), AssetType::Source);
        assert_eq!(type_of("art/crate.PNG"), AssetType::Image);
        assert_eq!(type_of("fonts/ui.ttf"), AssetType::Other);
    }

    #[test]
    fn a_meta_sits_beside_its_file() {
        assert_eq!(
            meta_path("art/crate.png").as_deref(),
            Some("art/crate.image.rs")
        );
        assert_eq!(meta_path("hero.glb").as_deref(), Some("hero.mesh.rs"));
        assert_eq!(
            meta_path("a/b/theme.final.ogg").as_deref(),
            Some("a/b/theme.final.audio.rs")
        );
        assert_eq!(meta_path("intro.mp4").as_deref(), Some("intro.video.rs"));
        assert_eq!(meta_path("fonts/ui.ttf"), None);
        assert!(is_meta("art/crate.image.rs"));
        assert!(!is_meta("scenes/main.scene.rs"));

        let files = ["art/crate.png", "art/crate.image.rs", "art/other.jpg"];
        assert_eq!(
            external_of("art/crate.image.rs", files).as_deref(),
            Some("art/crate.png")
        );
        assert_eq!(external_of("art/gone.image.rs", files), None);
        // The type must match: an audio meta never claims an image.
        assert_eq!(external_of("art/crate.audio.rs", files), None);
    }

    #[test]
    fn there_is_exactly_one_project_file() {
        assert_eq!(
            project_file(["a.project.rs", "scenes/b.project.rs"]),
            Ok("a.project.rs")
        );
        assert!(project_file(["scenes/main.scene.rs"]).is_err());
        assert!(
            project_file(["a.project.rs", "b.project.rs"])
                .unwrap_err()
                .contains("2")
        );
    }
}
