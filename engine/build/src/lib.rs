//! Build-time half of the Xerxes asset model. A game's `build.rs` is one line:
//!
//! ```ignore
//! fn main() { xerxes_build::generate(); }
//! ```
//!
//! From the authored `assets/` folder it
//! - generates `$OUT_DIR/xerxes_assets.rs`: a `pub mod assets` mirroring the folder (every `.rs`
//!   file becomes a module: `scenes/main.scene.rs` → `assets::scenes::main_scene`), the logic
//!   plugins (`assets/logic/*.rs`), the metas and the bundle manifest, and `assets::project()`
//!   (built from the one `assets/<name>.project.rs`, the game's project settings);
//! - packs every external file (anything not `.rs`) into the bundle its meta names
//!   (`.bundle("id")`, default `core`): `public/bundles/<id>.zip`. Only bundles ship.
//!
//! Assets are typed by name, `Name.Type.rs` (see [`types`]): an external file `art/crate.png`
//! has its meta beside it, `art/crate.image.rs`.
//!
//! It runs inside every build (cargo, dx, cargo-apk), so every platform gets the same output.

pub mod addons;
pub mod scaffold;
pub mod types;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

/// The bundle an external file goes to when its meta names none (or it has no meta yet).
pub const DEFAULT_BUNDLE: &str = "core";

/// Runs the whole step for the crate being built. Panics with a readable message on error,
/// which fails the build.
pub fn generate() {
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    generate_from(&crate_dir.join("assets"));
}

/// Like [`generate`], from an assets folder somewhere else (the engine's `__templates__`,
/// compiled by its check crate). Bundles still go to this crate's `public/`.
pub fn generate_from(assets: &Path) {
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    // `#[path]` in the generated file must be absolute (it is read from OUT_DIR).
    let assets = crate_dir.join(assets);
    println!("cargo:rerun-if-changed={}", assets.display());

    if let Err(err) = run(
        &assets,
        &crate_dir.join("public"),
        &out_dir.join("xerxes_assets.rs"),
    ) {
        panic!("xerxes_build: {err}");
    }
}

fn run(assets: &Path, public: &Path, generated: &Path) -> io::Result<()> {
    let layout = scan(assets)?;
    for file in layout.externals.iter().filter(|f| !f.has_meta) {
        // Files of a type without a meta (fonts, data) ship in the default bundle quietly.
        if let Some(meta) = types::meta_path(&file.path) {
            println!(
                "cargo:warning=assets/{} has no meta (assets/{meta}); it goes to bundle `{DEFAULT_BUNDLE}`",
                file.path
            );
        }
    }
    for meta in layout.orphan_metas() {
        println!("cargo:warning=assets/{meta} has no file beside it; it is ignored");
    }
    let bundles = write_bundles(assets, &public.join("bundles"), &layout)?;
    write_if_changed(generated, render(assets, &layout, &bundles)?.as_bytes())
}

/// An authored `.rs` file, by its path under `assets/` (forward slashes).
#[derive(Debug, Clone, PartialEq)]
pub struct Source {
    pub path: String,
}

/// An external file (image, model, audio, video, ...) and the bundle it goes to.
#[derive(Debug, Clone, PartialEq)]
pub struct External {
    pub path: String,
    pub bundle: String,
    pub has_meta: bool,
}

#[derive(Debug, Default, PartialEq)]
pub struct Layout {
    pub sources: Vec<Source>,
    pub externals: Vec<External>,
}

impl Layout {
    /// The path of the project settings file (`<name>.project.rs`, in the root of `assets/`).
    pub fn project_file(&self) -> io::Result<&str> {
        types::project_file(self.sources.iter().map(|s| s.path.as_str())).map_err(|e| invalid(&e))
    }

    /// Metas (`Name.image.rs`, ...) whose external file is missing.
    pub fn orphan_metas(&self) -> Vec<&str> {
        let externals: Vec<&str> = self.externals.iter().map(|e| e.path.as_str()).collect();
        self.sources
            .iter()
            .map(|s| s.path.as_str())
            .filter(|p| {
                types::is_meta(p) && types::external_of(p, externals.iter().copied()).is_none()
            })
            .collect()
    }
}

/// One packed bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    pub id: String,
    pub size: u64,
    pub files: Vec<String>,
}

/// Walks `assets/` (sorted, so output is stable) and classifies every file.
pub fn scan(assets: &Path) -> io::Result<Layout> {
    let mut files = Vec::new();
    walk(assets, assets, &mut files)?;
    files.sort();

    let mut layout = Layout::default();
    let mut metas: BTreeMap<String, String> = BTreeMap::new();
    for path in &files {
        if path.ends_with(".rs") {
            layout.sources.push(Source { path: path.clone() });
        } else {
            // `art/crate.png` is configured by `art/crate.image.rs`; two files of one type and
            // name (`crate.png`, `crate.jpg`) would share it.
            let meta = types::meta_path(path);
            if let Some(meta) = &meta {
                if let Some(other) = metas.insert(meta.clone(), path.clone()) {
                    return Err(invalid(&format!(
                        "assets/{other} and assets/{path} share one meta (assets/{meta}); rename one"
                    )));
                }
            }
            let (has_meta, bundle) = match meta.map(|m| fs::read_to_string(assets.join(m))) {
                Some(Ok(text)) => (
                    true,
                    bundle_of(&text)?.unwrap_or_else(|| DEFAULT_BUNDLE.into()),
                ),
                _ => (false, DEFAULT_BUNDLE.into()),
            };
            layout.externals.push(External {
                path: path.clone(),
                bundle,
                has_meta,
            });
        }
    }
    Ok(layout)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        // Dotfiles (`.gitkeep`) and a folder's README are for people, not for the build.
        if name.starts_with('.') || name.eq_ignore_ascii_case("README.md") {
            continue;
        }
        if path.is_dir() {
            walk(root, &path, out)?;
        } else {
            let rel = path.strip_prefix(root).expect("under root");
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

/// The bundle id a meta names: the string literal in `.bundle("id")`. Kept a literal so the
/// build can read it without compiling the game.
pub fn bundle_of(meta: &str) -> io::Result<Option<String>> {
    let Some(start) = meta.find(".bundle(") else {
        return Ok(None);
    };
    let rest = meta[start + ".bundle(".len()..].trim_start();
    let id = rest
        .strip_prefix('"')
        .and_then(|r| r.split_once('"'))
        .map(|(id, _)| id.to_string())
        .ok_or_else(|| invalid("a meta's bundle must be a string literal: .bundle(\"id\")"))?;
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err(invalid(&format!("bundle id `{id}`: use a-z, 0-9, - and _")));
    }
    Ok(Some(id))
}

/// A Rust module name for a file or folder name: `main.scene` → `main_scene`, `crate.image` →
/// `crate_image`, `2d` → `_2d`.
pub fn module_ident(name: &str) -> String {
    let mut ident: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    if ident.starts_with(|c: char| c.is_ascii_digit()) || ident.is_empty() {
        ident.insert(0, '_');
    }
    if matches!(
        ident.as_str(),
        "mod" | "crate" | "self" | "super" | "type" | "use" | "fn" | "impl" | "struct" | "enum"
    ) {
        ident.insert(0, '_');
    }
    ident
}

/// The module path (`self::art::crate_image`) of an authored `.rs` path (`art/crate.image.rs`).
pub fn module_path(rs_path: &str) -> String {
    let stem = rs_path.strip_suffix(".rs").unwrap_or(rs_path);
    let mut path = String::from("self");
    for part in stem.split('/') {
        path.push_str("::");
        path.push_str(&module_ident(part));
    }
    path
}

/// Packs the externals into one zip per bundle id under `bundles_dir`, and removes zips of
/// bundles that no longer exist. Zips are deterministic and only rewritten when they change.
pub fn write_bundles(
    assets: &Path,
    bundles_dir: &Path,
    layout: &Layout,
) -> io::Result<Vec<Bundle>> {
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for file in &layout.externals {
        groups.entry(&file.bundle).or_default().push(&file.path);
    }

    fs::create_dir_all(bundles_dir)?;
    for entry in fs::read_dir(bundles_dir)? {
        let path = entry?.path();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if path.extension().is_some_and(|e| e == "zip") && !groups.contains_key(stem) {
            fs::remove_file(path)?;
        }
    }

    let mut bundles = Vec::new();
    for (id, files) in groups {
        let bytes = zip_files(assets, &files)?;
        write_if_changed(&bundles_dir.join(format!("{id}.zip")), &bytes)?;
        bundles.push(Bundle {
            id: id.to_string(),
            size: bytes.len() as u64,
            files: files.iter().map(|f| f.to_string()).collect(),
        });
    }
    Ok(bundles)
}

fn zip_files(assets: &Path, files: &[&str]) -> io::Result<Vec<u8>> {
    use zip::write::SimpleFileOptions;
    let mut zip = zip::ZipWriter::new(io::Cursor::new(Vec::new()));
    // Fixed timestamps: the same files always give the same bytes.
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());
    for file in files {
        zip.start_file(*file, options).map_err(io::Error::other)?;
        zip.write_all(&fs::read(assets.join(file))?)?;
    }
    Ok(zip.finish().map_err(io::Error::other)?.into_inner())
}

/// The generated `pub mod assets`.
pub fn render(assets: &Path, layout: &Layout, bundles: &[Bundle]) -> io::Result<String> {
    // The game's ProjectSettings live in the one `<name>.project.rs`.
    let project_file = layout.project_file()?;

    // Folder tree of the `.rs` files.
    #[derive(Default)]
    struct Dir {
        dirs: BTreeMap<String, Dir>,
        files: Vec<(String, String)>,
    }
    let mut root = Dir::default();
    for source in &layout.sources {
        let mut parts: Vec<&str> = source.path.split('/').collect();
        let file = parts.pop().expect("a file name");
        let mut dir = &mut root;
        for part in parts {
            dir = dir.dirs.entry(module_ident(part)).or_default();
        }
        let ident = module_ident(file.strip_suffix(".rs").unwrap_or(file));
        if dir.files.iter().any(|(i, _)| *i == ident) || dir.dirs.contains_key(&ident) {
            return Err(invalid(&format!(
                "assets/{}: another file maps to module `{ident}`",
                source.path
            )));
        }
        let absolute = assets
            .join(&source.path)
            .to_string_lossy()
            .replace('\\', "/");
        dir.files.push((ident, absolute));
    }
    fn emit(dir: &Dir, out: &mut String, depth: usize) {
        let pad = "    ".repeat(depth);
        for (ident, path) in &dir.files {
            let _ = writeln!(out, "{pad}#[path = {path:?}]\n{pad}pub mod {ident};");
        }
        for (ident, sub) in &dir.dirs {
            let _ = writeln!(out, "{pad}pub mod {ident} {{");
            emit(sub, out, depth + 1);
            let _ = writeln!(out, "{pad}}}");
        }
    }

    let engine = "::xerxes_engine::modules::assets";
    let mut out = String::from("// Generated by xerxes_build from assets/. Do not edit.\n");
    out.push_str("#[allow(dead_code, unused_imports)]\npub mod assets {\n");
    emit(&root, &mut out, 1);

    out.push_str(&format!(
        "\n    pub const BUNDLES: &[::xerxes_engine::modules::bundles::BundleInfo] = &[\n"
    ));
    for b in bundles {
        let _ = writeln!(
            out,
            "        ::xerxes_engine::modules::bundles::BundleInfo {{ id: {:?}, size: {}, files: &{:?} }},",
            b.id, b.size, b.files
        );
    }
    out.push_str("    ];\n");

    out.push_str(&format!(
        "    pub const METAS: &[(&str, fn() -> {engine}::AssetMeta)] = &[\n"
    ));
    for file in layout.externals.iter().filter(|f| f.has_meta) {
        let meta = types::meta_path(&file.path).expect("an external with a meta has a meta path");
        let _ = writeln!(
            out,
            "        ({:?}, {}::meta),",
            file.path,
            module_path(&meta)
        );
    }
    out.push_str("    ];\n");

    out.push_str("    pub const LOGIC: &[fn(&mut ::bevy::app::App)] = &[\n");
    for source in layout
        .sources
        .iter()
        .filter(|s| s.path.starts_with("logic/"))
    {
        let _ = writeln!(out, "        {}::plugin,", module_path(&source.path));
    }
    out.push_str("    ];\n");

    out.push_str(&format!(
        "\n    /// The project: its settings, bundles, metas and logic modules.\n    pub fn project() -> ::xerxes_engine::modules::project::Project {{\n        ::xerxes_engine::modules::project::Project {{ settings: {project}::settings(), bundles: BUNDLES, metas: METAS, logic: LOGIC }}\n    }}\n}}\n",
        project = module_path(project_file)
    ));
    Ok(out)
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if fs::read(path).is_ok_and(|old| old == bytes) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xerxes_build_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn put(root: &Path, path: &str, body: &[u8]) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    #[test]
    fn idents_are_valid_rust() {
        assert_eq!(module_ident("main.scene"), "main_scene");
        assert_eq!(module_ident("crate.image"), "crate_image");
        assert_eq!(module_ident("2d-art"), "_2d_art");
        assert_eq!(module_ident("mod"), "_mod");
        assert_eq!(module_path("art/crate.image.rs"), "self::art::crate_image");
        assert_eq!(module_path("my-game.project.rs"), "self::my_game_project");
    }

    #[test]
    fn reads_the_bundle_literal() {
        assert_eq!(
            bundle_of("AssetMeta::image(x).bundle(\"level-2\")").unwrap(),
            Some("level-2".into())
        );
        assert_eq!(bundle_of("AssetMeta::image(x)").unwrap(), None);
        assert!(bundle_of(".bundle(NAME)").is_err());
        assert!(bundle_of(".bundle(\"Bad Id\")").is_err());
    }

    #[test]
    fn groups_files_by_bundle_and_ships_no_source() {
        let root = temp("bundles");
        let assets = root.join("assets");
        put(&assets, "game.project.rs", b"");
        put(&assets, "scenes/main.scene.rs", b"");
        put(&assets, "art/a.png", b"aaaa");
        put(
            &assets,
            "art/a.image.rs",
            b"AssetMeta::image(ImageMeta::default()).bundle(\"core\")",
        );
        put(&assets, "art/b.png", b"bbbb");
        put(
            &assets,
            "art/b.image.rs",
            b"AssetMeta::image(ImageMeta::default()).bundle(\"extra\")",
        );
        put(&assets, "art/c.ogg", b"cccc"); // no meta: core

        let layout = scan(&assets).unwrap();
        assert_eq!(layout.sources.len(), 4);
        assert!(
            layout
                .externals
                .iter()
                .any(|e| e.path == "art/c.ogg" && !e.has_meta && e.bundle == "core")
        );

        let dir = root.join("public/bundles");
        let bundles = write_bundles(&assets, &dir, &layout).unwrap();
        let ids: Vec<_> = bundles.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, ["core", "extra"]);
        assert_eq!(bundles[0].files, ["art/a.png", "art/c.ogg"]);

        // Only zips ship, and the core zip holds exactly its files.
        let shipped: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(shipped.len(), 2);
        let mut zip = zip::ZipArchive::new(fs::File::open(dir.join("core.zip")).unwrap()).unwrap();
        let mut names: Vec<_> = (0..zip.len())
            .map(|i| zip.by_index(i).unwrap().name().to_string())
            .collect();
        names.sort();
        assert_eq!(names, ["art/a.png", "art/c.ogg"]);

        // Same input, same bytes; a bundle that disappears loses its zip.
        let again = write_bundles(&assets, &dir, &layout).unwrap();
        assert_eq!(again, bundles);
        fs::remove_file(assets.join("art/b.png")).unwrap();
        write_bundles(&assets, &dir, &scan(&assets).unwrap()).unwrap();
        assert!(!dir.join("extra.zip").exists());
    }

    #[test]
    fn renders_modules_metas_logic_and_project() {
        let root = temp("render");
        let assets = root.join("assets");
        put(&assets, "game.project.rs", b"");
        put(&assets, "logic/mover.rs", b"");
        put(&assets, "art/a.png", b"a");
        put(&assets, "art/a.image.rs", b"");
        let layout = scan(&assets).unwrap();
        let code = render(&assets, &layout, &[]).unwrap();
        assert!(code.contains("pub mod logic {"));
        assert!(code.contains("pub mod mover;"));
        assert!(
            code.contains("(\"art/a.png\", self::art::a_image::meta)"),
            "{code}"
        );
        assert!(code.contains("self::logic::mover::plugin,"));
        assert!(code.contains("pub fn project()"));
        assert!(code.contains("settings: self::game_project::settings()"));

        fs::remove_file(assets.join("game.project.rs")).unwrap();
        assert!(render(&assets, &scan(&assets).unwrap(), &[]).is_err());
    }

    #[test]
    fn metas_are_named_by_type_and_collisions_are_refused() {
        let root = temp("typed");
        let assets = root.join("assets");
        put(&assets, "game.project.rs", b"");
        put(&assets, "art/a.png", b"a");
        put(&assets, "art/a.image.rs", b"");
        put(&assets, "art/gone.image.rs", b"");
        let layout = scan(&assets).unwrap();
        assert!(layout.externals.iter().all(|e| e.has_meta));
        assert_eq!(layout.orphan_metas(), ["art/gone.image.rs"]);
        assert_eq!(layout.project_file().unwrap(), "game.project.rs");

        // `a.png` and `a.jpg` would both be configured by `a.image.rs`.
        put(&assets, "art/a.jpg", b"b");
        let err = scan(&assets).unwrap_err().to_string();
        assert!(err.contains("share one meta"), "{err}");

        // Two project files are refused too.
        fs::remove_file(assets.join("art/a.jpg")).unwrap();
        put(&assets, "other.project.rs", b"");
        assert!(scan(&assets).unwrap().project_file().is_err());
    }
}
