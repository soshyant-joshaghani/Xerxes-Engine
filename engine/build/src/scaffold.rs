//! New projects: the engine's own starter project (or a finished template game's files), made
//! into a new game. The one implementation of "new project": the native editor, the backend
//! (for the web editor) and `xerxes-ctrl game new` (through the `xerxes-new` binary) all call it.

use std::io::{self, Cursor, Read, Write};
use std::path::Path;

use crate::types;

/// Not copied from a template: build output, the lock file, Git, generated bundles.
pub const EXCLUDED: &[&str] = &[".git", "target", "dist", "public", "Cargo.lock"];

/// Names a new project cannot take: `dist/<name>/` and `<name>.play.localhost` are shared by
/// the engine app, games and templates.
pub const RESERVED: &[&str] = &["engine", STARTER];

/// The engine's built-in starter project: an empty game with one 3D scene. It is not a folder
/// anywhere: it is compiled into this crate ([`starter_files`]), so every way of creating a
/// project has it, with or without the repo.
pub const STARTER: &str = "starter";

/// The template a new project starts from when none is named.
pub const DEFAULT_TEMPLATE: &str = STARTER;

macro_rules! starter_text {
    ($($path:literal),* $(,)?) => {
        vec![$(($path.to_string(), include_str!(concat!("../../starter/", $path)).as_bytes().to_vec())),*]
    };
}

macro_rules! starter_binary {
    ($($path:literal),* $(,)?) => {
        vec![$(($path.to_string(), include_bytes!(concat!("../../starter/", $path)).to_vec())),*]
    };
}

/// The files of the built-in starter project (`engine/starter/`), as a template
/// for [`customize`] under the name [`STARTER`].
pub fn starter_files() -> Vec<File> {
    let mut files = starter_text![
        "Cargo.toml",
        "Dioxus.toml",
        "README.md",
        "index.html",
        "build.rs",
        "src/lib.rs",
        "src/main.rs",
        "src/android.rs",
        "assets/starter.project.rs",
        "assets/scenes/main.scene.rs",
    ];
    files.extend(starter_binary![
        "res/mipmap-hdpi/ic_launcher.png",
        "res/mipmap-mdpi/ic_launcher.png",
        "res/mipmap-xhdpi/ic_launcher.png",
        "res/mipmap-xxhdpi/ic_launcher.png",
        "res/mipmap-xxxhdpi/ic_launcher.png",
    ]);
    files
}

/// Where the new game finds the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineSource {
    /// A relative path to the engine crate (inside the Xerxes repo: `../../engine`).
    Path(String),
    /// The engine's Git repository (a project anywhere else).
    Git { url: String, branch: String },
}

/// The engine's public repository: projects outside the repo depend on it from here.
pub const ENGINE_GIT: &str = "https://github.com/soshyant-joshaghani/Xerxes-Engine.git";

/// A file of a project: its path relative to the project folder (`/`-separated) and bytes.
pub type File = (String, Vec<u8>);

/// Why `name` cannot be a new project, or `None` when it can. `games` and `templates` are
/// the folder names already taken.
pub fn name_problem(name: &str, games: &[&str], templates: &[&str]) -> Option<String> {
    if name.is_empty() {
        return Some("Type a name.".into());
    }
    if !name.starts_with(|c: char| c.is_ascii_lowercase()) {
        return Some("Start with a letter (a-z).".into());
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Some("Use only a-z, 0-9 and -.".into());
    }
    if RESERVED.contains(&name) || templates.contains(&name) {
        return Some(format!(
            "`{name}` is reserved (the engine app or a template has that name)."
        ));
    }
    if games.contains(&name) {
        return Some(format!("`{name}` already exists."));
    }
    None
}

fn excluded(path: &str) -> bool {
    path.split('/').any(|part| EXCLUDED.contains(&part))
}

/// A template game folder's files (without [`EXCLUDED`]), sorted by path.
pub fn read_dir(dir: &Path) -> io::Result<Vec<File>> {
    let mut paths = Vec::new();
    collect(dir, dir, &mut paths)?;
    paths.sort();
    paths
        .into_iter()
        .map(|rel| std::fs::read(dir.join(&rel)).map(|bytes| (rel, bytes)))
        .collect()
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<String>) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map_err(|_| io::Error::other("path outside the template"))?
            .to_string_lossy()
            .replace('\\', "/");
        if excluded(&rel) {
            continue;
        }
        if entry.file_type()?.is_dir() {
            collect(root, &path, out)?;
        } else {
            out.push(rel);
        }
    }
    Ok(())
}

/// Templates packed into one zip (`<template>/<path>`), for embedding in the editor.
/// Deterministic: same files, same bytes.
pub fn pack(templates: &[(String, Vec<File>)]) -> io::Result<Vec<u8>> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());
    for (name, files) in templates {
        for (path, bytes) in files {
            zip.start_file(format!("{name}/{path}"), options)
                .map_err(io::Error::other)?;
            zip.write_all(bytes)?;
        }
    }
    Ok(zip.finish().map_err(io::Error::other)?.into_inner())
}

/// The templates in a [`pack`]: (template name, its files).
pub fn unpack(bytes: &[u8]) -> io::Result<Vec<(String, Vec<File>)>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(io::Error::other)?;
    let mut out: Vec<(String, Vec<File>)> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(io::Error::other)?;
        if entry.is_dir() {
            continue;
        }
        let full = entry.name().to_string();
        let Some((template, path)) = full.split_once('/') else {
            continue;
        };
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        match out.iter_mut().find(|(name, _)| name == template) {
            Some((_, files)) => files.push((path.to_string(), data)),
            None => out.push((template.to_string(), vec![(path.to_string(), data)])),
        }
    }
    Ok(out)
}

/// A TOML `key = "value"` line's value (first match).
fn toml_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let (k, v) = line.split_once('=')?;
        (k.trim() == key).then(|| v.trim().trim_matches('"').to_string())
    })
}

fn text_of<'a>(files: &'a [File], path: &str) -> Option<&'a str> {
    files
        .iter()
        .find(|(p, _)| p == path)
        .and_then(|(_, b)| std::str::from_utf8(b).ok())
}

/// A project's description: the first sentence of its README's first paragraph, without
/// Markdown links or code marks.
pub fn readme_summary(readme: &str) -> Option<String> {
    let text = readme.replace("\r\n", "\n");
    let paragraph = text.split("\n\n").map(str::trim).find(|p| {
        !p.is_empty() && !p.starts_with('#') && !p.starts_with('[') && !p.starts_with("```")
    })?;
    // [label](url) → label
    let mut plain = String::new();
    let mut rest = paragraph.replace('\n', " ");
    while let Some(open) = rest.find('[') {
        let Some(close) = rest[open..].find("](").map(|i| open + i) else {
            break;
        };
        let Some(end) = rest[close..].find(')').map(|i| close + i) else {
            break;
        };
        plain.push_str(&rest[..open]);
        plain.push_str(&rest[open + 1..close]);
        rest = rest[end + 1..].to_string();
    }
    plain.push_str(&rest);
    // Asides that are only code (" (`../../engine`)") are for readers of the file, not a summary.
    while let Some(open) = plain.find(" (`") {
        let Some(close) = plain[open..].find("`)").map(|i| open + i) else {
            break;
        };
        plain.replace_range(open..close + 2, "");
    }
    let plain = plain.replace('`', "");
    let sentence = match plain.find(". ") {
        Some(i) => &plain[..=i],
        None => plain.as_str(),
    };
    Some(sentence.trim().to_string()).filter(|s| !s.is_empty())
}

/// `[web.app] title` from a Dioxus.toml's text.
pub fn dioxus_title(text: &str) -> Option<String> {
    toml_value(text, "title")
}

/// The template's display title (`[web.app] title` in Dioxus.toml), e.g. "Cyrus".
pub fn template_title(files: &[File]) -> Option<String> {
    text_of(files, "Dioxus.toml").and_then(|t| toml_value(t, "title"))
}

/// The engine dependency, as Cargo.toml writes it.
fn engine_dep(source: &EngineSource, sub: &str) -> String {
    match source {
        EngineSource::Path(base) => {
            let base = base.trim_end_matches('/');
            format!("path = \"{base}{sub}\"")
        }
        EngineSource::Git { url, branch } => format!("git = \"{url}\", branch = \"{branch}\""),
    }
}

/// Every `path = "../../../engine[/sub]"` (the template's view of the engine) pointed at
/// `engine`: a path for a game inside the repo, the engine's repository elsewhere.
fn rewrite_engine_paths(text: &str, engine: &EngineSource) -> String {
    const FROM: &str = "path = \"../../../engine";
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find(FROM) {
        let after = &rest[i + FROM.len()..];
        let Some(end) = after.find('"') else { break };
        let sub = &after[..end];
        out.push_str(&rest[..i]);
        if sub.is_empty() || sub.starts_with('/') {
            out.push_str(&engine_dep(engine, sub));
        } else {
            // Not the engine folder (e.g. `../../../engine-extras`): keep it.
            out.push_str(&rest[i..i + FROM.len() + end + 1]);
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// The template's files made into the game `name`: package, binary, APK and Android
/// package names, titles, the README, and where the engine comes from.
pub fn customize(
    files: Vec<File>,
    template: &str,
    name: &str,
    engine: &EngineSource,
) -> io::Result<Vec<File>> {
    let cargo = text_of(&files, "Cargo.toml")
        .ok_or_else(|| io::Error::other(format!("template `{template}` has no Cargo.toml")))?;
    let old_pkg = toml_value(cargo, "name")
        .ok_or_else(|| io::Error::other("template Cargo.toml has no package name"))?;
    let old_title = template_title(&files).unwrap_or_else(|| old_pkg.clone());
    let snake = |s: &str| s.replace('-', "_");

    let ignore = gitignore(text_of(&files, ".gitignore"));
    let mut files: Vec<File> = files
        .into_iter()
        .filter(|(p, _)| p != ".gitignore")
        .collect();
    files.push((".gitignore".into(), ignore.into_bytes()));
    files
        .into_iter()
        .map(|(path, bytes)| {
            let Ok(text) = String::from_utf8(bytes.clone()) else { return Ok((path, bytes)) };
            let text = match path.as_str() {
                "Cargo.toml" => {
                    // The template's engine paths (it lives in games/__templates__/<t>): the
                    // engine, build, the engine's patched winit (winit/).
                    // The Android package id first: it takes the underscore name, and a
                    // one-word template (`starter`) would otherwise get the hyphenated one.
                    let mut text = rewrite_engine_paths(&text, engine)
                        .replace(
                            &format!("dev.xerxes.{}", snake(&old_pkg)),
                            &format!("dev.xerxes.{}", snake(name)),
                        )
                        .replace(&old_pkg, name)
                        .replace(&format!("label = \"{old_title}\""), &format!("label = \"{name}\""));
                    if text.starts_with('#') {
                        let rest = text.split_once('\n').map(|(_, r)| r.to_string()).unwrap_or_default();
                        text = format!("# {name}, on the Xerxes engine (created from {template}).\n{rest}");
                    }
                    text
                }
                "Dioxus.toml" => text
                    .replace(&format!("name = \"{old_pkg}\""), &format!("name = \"{name}\""))
                    .replace(&format!("title = \"{old_title}\""), &format!("title = \"{name}\"")),
                "README.md" => {
                    // The template's intro is about creating games from it; keep its layout.
                    let layout = text.find("## Layout").map(|i| text[i..].to_string()).unwrap_or_default();
                    let mut readme = format!(
                        "# {name}\n\nA game made with the Xerxes engine, created from `{template}`. \
                         Open it in the Xerxes editor (Project Manager).\n"
                    );
                    if !layout.is_empty() {
                        readme.push('\n');
                        readme.push_str(&layout);
                    }
                    readme
                }
                p if p.ends_with(".rs") && (p.starts_with("src/") || p.starts_with("assets/")) => {
                    text.replace(&format!("\"{old_title}\""), &format!("\"{name}\""))
                }
                _ => return Ok((path, bytes)),
            };
            Ok((path, text.into_bytes()))
        })
        .map(|file| {
            // The project settings are named for the game: `assets/<name>.project.rs`.
            file.map(|(path, bytes)| match project_settings_path(&path) {
                true => (format!("assets/{name}.project.rs"), bytes),
                false => (path, bytes),
            })
        })
        .collect()
}

/// `assets/<anything>.project.rs`, in the root of `assets/`.
fn project_settings_path(path: &str) -> bool {
    path.strip_prefix("assets/").is_some_and(|rest| {
        !rest.contains('/')
            && types::typed_name(rest).is_some_and(|(_, t)| t == types::AssetType::Project)
    })
}

/// What every game's `.gitignore` keeps out: build output, release builds, generated bundles.
pub const GITIGNORE: &[&str] = &["/target/", "/dist/", "/public/"];

/// The project's `.gitignore` with at least [`GITIGNORE`] (the template's own lines kept).
fn gitignore(existing: Option<&str>) -> String {
    let mut lines: Vec<String> = existing.unwrap_or("").lines().map(str::to_string).collect();
    for needed in GITIGNORE {
        if !lines.iter().any(|l| l.trim() == *needed) {
            lines.push(needed.to_string());
        }
    }
    lines.join("\n") + "\n"
}

/// Writes a new project into `dest`, which must not exist yet.
pub fn write(dest: &Path, files: &[File]) -> io::Result<()> {
    if dest.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} already exists", dest.display()),
        ));
    }
    for (path, bytes) in files {
        if path
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
        {
            return Err(io::Error::other(format!("bad path in template: {path}")));
        }
        let target = dest.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(target, bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template() -> Vec<File> {
        let file = |p: &str, t: &str| (p.to_string(), t.as_bytes().to_vec());
        vec![
            file(
                "Cargo.toml",
                "# Cyrus, a template.\n[package]\nname = \"cyrus\"\n[lib]\nname = \"game\"\n\
                 [package.metadata.android]\npackage = \"dev.xerxes.cyrus\"\n\
                 [package.metadata.android.application]\nlabel = \"Cyrus\"\n\
                 [dependencies]\nxerxes_engine = { path = \"../../../engine\", default-features = false }\n\
                 [build-dependencies]\nxerxes_build = { path = \"../../../engine/build\" }\n\
                 [patch.crates-io]\nwinit = { path = \"../../../engine/winit\" }\n",
            ),
            file(
                "Dioxus.toml",
                "[application]\nname = \"cyrus\"\n[web.app]\ntitle = \"Cyrus\"\n",
            ),
            file(
                "README.md",
                "# Cyrus\n\nHow to make games from it.\n\n## Layout\n\nsrc/\n",
            ),
            file("assets/cyrus.project.rs", "title: \"Cyrus\","),
            file("assets/art/checker.png", "\u{0}png"),
        ]
    }

    fn get<'a>(files: &'a [File], path: &str) -> &'a str {
        text_of(files, path).unwrap()
    }

    #[test]
    fn readmes_summarize_to_their_first_sentence() {
        let readme = "# Zeta\r\n\r\nA [space](x.md) shooter with `ships`. It has more.\r\n";
        assert_eq!(
            readme_summary(readme).as_deref(),
            Some("A space shooter with ships.")
        );
        assert_eq!(readme_summary("# Only a title\n"), None);
        let template = "# T\n\nThe starting point for games made with the [Xerxes engine](../README.md) (`../../../engine`). More.\n";
        assert_eq!(
            readme_summary(template).as_deref(),
            Some("The starting point for games made with the Xerxes engine.")
        );
    }

    #[test]
    fn names_follow_one_rule() {
        assert_eq!(
            name_problem("my-game", &["darius"], &["cyrus"]),
            None
        );
        assert!(name_problem("", &[], &[]).is_some());
        assert!(name_problem("2d", &[], &[]).is_some());
        assert!(name_problem("My Game", &[], &[]).is_some());
        assert!(
            name_problem("engine", &[], &[])
                .unwrap()
                .contains("reserved")
        );
        assert!(
            name_problem("cyrus", &[], &["cyrus"])
                .unwrap()
                .contains("reserved")
        );
        assert!(
            name_problem("darius", &["darius"], &[])
                .unwrap()
                .contains("exists")
        );
    }

    #[test]
    fn a_new_game_inside_the_repo_uses_the_engine_path() {
        let files = customize(
            template(),
            "cyrus",
            "space-rush",
            &EngineSource::Path("../../engine".into()),
        )
        .unwrap();
        let cargo = get(&files, "Cargo.toml");
        assert!(
            cargo.starts_with("# space-rush, on the Xerxes engine (created from cyrus).\n")
        );
        assert!(cargo.contains("name = \"space-rush\""));
        assert!(
            cargo.contains("name = \"game\""),
            "the lib name stays `game`"
        );
        assert!(cargo.contains("package = \"dev.xerxes.space_rush\""));
        assert!(cargo.contains("label = \"space-rush\""));
        assert!(
            cargo.contains("xerxes_engine = { path = \"../../engine\", default-features = false }")
        );
        assert!(cargo.contains("xerxes_build = { path = \"../../engine/build\" }"));
        assert!(cargo.contains("winit = { path = \"../../engine/winit\" }"));
        assert!(get(&files, "Dioxus.toml").contains("title = \"space-rush\""));
        assert_eq!(
            get(&files, "assets/space-rush.project.rs"),
            "title: \"space-rush\","
        );
        let readme = get(&files, "README.md");
        assert!(
            readme.starts_with("# space-rush\n")
                && readme.contains("## Layout")
                && !readme.contains("How to make")
        );
        assert_eq!(
            files
                .iter()
                .find(|(p, _)| p == "assets/art/checker.png")
                .unwrap()
                .1,
            b"\0png"
        );
    }

    #[test]
    fn every_new_game_ignores_its_build_output() {
        let files = customize(
            template(),
            "cyrus",
            "g",
            &EngineSource::Path("../../engine".into()),
        )
        .unwrap();
        assert_eq!(get(&files, ".gitignore"), "/target/\n/dist/\n/public/\n");
        let mut with_own = template();
        with_own.push((".gitignore".into(), b"/target/\n*.log\n".to_vec()));
        let files = customize(
            with_own,
            "cyrus",
            "g",
            &EngineSource::Path("../../engine".into()),
        )
        .unwrap();
        assert_eq!(
            get(&files, ".gitignore"),
            "/target/\n*.log\n/dist/\n/public/\n"
        );
    }

    #[test]
    fn a_new_game_elsewhere_uses_the_engine_repository() {
        let git = EngineSource::Git {
            url: ENGINE_GIT.into(),
            branch: "main".into(),
        };
        let files = customize(template(), "cyrus", "solo", &git).unwrap();
        let cargo = get(&files, "Cargo.toml");
        assert!(cargo.contains(&format!("xerxes_engine = {{ git = \"{ENGINE_GIT}\", branch = \"main\", default-features = false }}")));
        assert!(cargo.contains(&format!(
            "xerxes_build = {{ git = \"{ENGINE_GIT}\", branch = \"main\" }}"
        )));
        assert!(cargo.contains(&format!(
            "winit = {{ git = \"{ENGINE_GIT}\", branch = \"main\" }}"
        )));
        assert!(!cargo.contains("path ="));
    }

    #[test]
    fn templates_pack_and_unpack() {
        let packed = pack(&[("cyrus".into(), template())]).unwrap();
        assert_eq!(
            packed,
            pack(&[("cyrus".into(), template())]).unwrap(),
            "deterministic"
        );
        let unpacked = unpack(&packed).unwrap();
        assert_eq!(unpacked, vec![("cyrus".to_string(), template())]);
    }

    #[test]
    fn writing_refuses_an_existing_folder_and_bad_paths() {
        let dir = std::env::temp_dir().join(format!("xerxes-scaffold-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        write(&dir, &template()).unwrap();
        assert!(dir.join("assets/art/checker.png").is_file());
        assert_eq!(
            write(&dir, &template()).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        let bad = vec![("../x".to_string(), Vec::new())];
        assert!(write(&dir.join("other"), &bad).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_starter_is_built_in_and_becomes_a_named_game() {
        let files = customize(
            starter_files(),
            STARTER,
            "coin-rush",
            &EngineSource::Path("../../engine".into()),
        )
        .unwrap();
        let paths: Vec<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
        assert!(paths.contains(&"assets/coin-rush.project.rs"));
        assert!(!paths.contains(&"assets/starter.project.rs"));
        assert!(paths.contains(&"assets/scenes/main.scene.rs"));
        let cargo = get(&files, "Cargo.toml");
        assert!(cargo.contains("name = \"coin-rush\""));
        assert!(cargo.contains("package = \"dev.xerxes.coin_rush\""));
        assert!(
            cargo.contains("xerxes_engine = { path = \"../../engine\", default-features = false }")
        );
        assert!(
            !cargo.contains("Starter") && !cargo.contains("\"starter\""),
            "{cargo}"
        );
        assert!(get(&files, "assets/coin-rush.project.rs").contains("title: \"coin-rush\""));
        assert!(get(&files, "Dioxus.toml").contains("title = \"coin-rush\""));
        // The build step accepts what a new game is made of.
        let readme = get(&files, "README.md");
        assert!(
            readme.starts_with(
                "# coin-rush
"
            ),
            "{readme}"
        );
        // `starter` cannot be a project name: it is the built-in template's.
        assert!(
            name_problem("starter", &[], &[])
                .unwrap()
                .contains("reserved")
        );
        assert_eq!(template_title(&starter_files()).as_deref(), Some("Starter"));
        assert!(
            readme_summary(get(&starter_files(), "README.md"))
                .unwrap()
                .starts_with("An empty game")
        );
    }
}
