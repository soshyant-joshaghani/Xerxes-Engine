//! The engine's add-ons: optional gameplay (`engine/addons/<feature>/<name>.rs`:
//! camera and character controllers, HUD, pause, ...) that a game copies into its own
//! `src/addons/` and then owns. It is compiled into this crate, so the editor, the backend and
//! the CLI offer the same add-ons with or without the repo.
//!
//! What every game has (scenes, bundles, flow, input, project settings) is engine code, not
//! an add-on.

use std::fs;
use std::path::Path;

use crate::scaffold::File;

include!(concat!(env!("OUT_DIR"), "/addons_library.rs"));

/// One add-on of the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addon {
    /// Its place in the library: `hud/hud.rs`.
    pub path: String,
    /// The first paragraph of its header docs, without the `Template:` / `Add-on:` label.
    pub summary: String,
    /// The library paths of the add-ons it uses (`//! needs: pause`), resolved by file name.
    pub needs: Vec<String>,
}

/// Where an add-on lands in a game: always `src/addons/<file>.rs`.
pub fn destination(path: &str) -> String {
    format!("src/addons/{}", path.rsplit('/').next().unwrap_or(path))
}

/// Every add-on in the library.
pub fn all() -> Vec<Addon> {
    let known: Vec<String> = LIBRARY.iter().map(|(p, _)| p.to_string()).collect();
    LIBRARY
        .iter()
        .map(|(path, source)| describe(path, source, &known))
        .collect()
}

/// Summary and needs from a module's header doc lines. `known` is every library path, to
/// resolve `needs:` names against; an unknown name is kept as written.
pub fn describe(path: &str, source: &str, known: &[String]) -> Addon {
    let docs: Vec<&str> = source
        .lines()
        .take_while(|l| l.starts_with("//!"))
        .map(|l| l.trim_start_matches("//!").trim())
        .collect();
    // The first paragraph of the header: its lines up to a blank `//!` or `needs:`.
    let summary = docs
        .iter()
        .take_while(|l| !l.is_empty() && !l.starts_with("needs:"))
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_start_matches("Template:")
        .trim_start_matches("Add-on:")
        .trim()
        .to_string();
    let needs = docs
        .iter()
        .find_map(|l| l.strip_prefix("needs:"))
        .map(|list| {
            list.split(',')
                .map(str::trim)
                .filter(|n| !n.is_empty())
                .map(|name| resolve(known, name))
                .collect()
        })
        .unwrap_or_default();
    Addon {
        path: path.to_string(),
        summary,
        needs,
    }
}

/// The library path a `needs:` name stands for: the one file with that name, in any feature
/// folder (`pause` → `game/pause.rs`).
fn resolve(known: &[String], name: &str) -> String {
    known
        .iter()
        .find(|p| {
            p.rsplit('/')
                .next()
                .and_then(|f| f.strip_suffix(".rs"))
                .is_some_and(|stem| stem == name)
        })
        .cloned()
        .unwrap_or_else(|| name.to_string())
}

/// The files to add for `path` and, transitively, what it needs: `(destination, bytes)`, each
/// once, sorted by destination. `Err` names a module the library does not have.
pub fn files_for(path: &str) -> Result<Vec<File>, String> {
    let known: Vec<String> = LIBRARY.iter().map(|(p, _)| p.to_string()).collect();
    let mut wanted: Vec<String> = Vec::new();
    let mut queue = vec![path.to_string()];
    while let Some(next) = queue.pop() {
        if wanted.contains(&next) {
            continue;
        }
        let (_, source) = LIBRARY
            .iter()
            .find(|(p, _)| *p == next)
            .ok_or_else(|| format!("no add-on `{next}` in the library"))?;
        queue.extend(describe(&next, source, &known).needs);
        wanted.push(next);
    }
    let mut files: Vec<File> = wanted
        .iter()
        .map(|p| {
            let source = LIBRARY.iter().find(|(l, _)| l == p).unwrap().1;
            (destination(p), source.as_bytes().to_vec())
        })
        .collect();
    files.sort();
    Ok(files)
}

/// What [`install`] did: the files it wrote and the ones the game already had.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Install {
    /// Files written, relative to the game's root.
    pub added: Vec<String>,
    /// Files the game already has (its own copies are never overwritten).
    pub kept: Vec<String>,
}

/// Installs add-on `path` and, transitively, what it needs into the game at `root`. Each file
/// goes to `src/addons/<file>.rs` and is wired in: `pub mod <file>;` in `src/addons/mod.rs` (and
/// `pub mod addons;` in `src/lib.rs`), and its `plugin` in the project's `LOGIC` list. The game
/// owns its copies: a file it already has is kept, and wiring is only added where it is missing.
pub fn install(root: &Path, path: &str) -> Result<Install, String> {
    let files = files_for(path)?;
    let mut out = Install::default();
    for (landing, bytes) in &files {
        let dest = root.join(landing);
        if dest.exists() {
            out.kept.push(landing.clone());
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&dest, bytes).map_err(|e| e.to_string())?;
        out.added.push(landing.clone());
    }
    let stems: Vec<String> = files.iter().map(|(p, _)| module_stem(p)).collect();
    wire(root, &stems)?;
    Ok(out)
}

/// `src/addons/hud.rs` → `hud`.
fn module_stem(landing: &str) -> String {
    let file = landing.rsplit('/').next().unwrap_or(landing);
    file.strip_suffix(".rs").unwrap_or(file).to_string()
}

const ADDONS_MOD: &str = "//! The game's add-ons, copied from the engine (`engine/addons`). Each one is a module here, and\n//! its `plugin` is listed in the project's `LOGIC`. The game owns them.\n";

/// Wires add-on modules into a game: `src/addons/mod.rs`, `src/lib.rs` and the project's `LOGIC`.
fn wire(root: &Path, stems: &[String]) -> Result<(), String> {
    let lib_path = root.join("src/lib.rs");
    let lib = fs::read_to_string(&lib_path).map_err(|e| format!("src/lib.rs: {e}"))?;
    let mod_path = root.join("src/addons/mod.rs");
    let mods = fs::read_to_string(&mod_path).unwrap_or_else(|_| ADDONS_MOD.to_string());
    let project_path = project_settings_file(&root.join("assets"))?;
    let project = fs::read_to_string(&project_path).map_err(|e| e.to_string())?;

    let new_lib = wire_lib(&lib);
    let new_mods = wire_mod(&mods, stems);
    let new_project = wire_project(&project, stems)?;
    for (path, old, new) in [
        (lib_path, lib, new_lib),
        (mod_path, mods, new_mods),
        (project_path, project, new_project),
    ] {
        if new != old {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::write(&path, new).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// The project settings file of a game's `assets/` (`<name>.project.rs`).
fn project_settings_file(assets: &Path) -> Result<std::path::PathBuf, String> {
    let names: Vec<String> = fs::read_dir(assets)
        .map_err(|e| format!("assets/: {e}"))?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .collect();
    let found = crate::types::project_file(names.iter().map(String::as_str))?;
    Ok(assets.join(found))
}

/// `src/lib.rs` with `pub mod addons;` (before the generated assets, if it has them).
pub fn wire_lib(text: &str) -> String {
    if text.lines().any(|l| l.trim() == "pub mod addons;") {
        return text.to_string();
    }
    let line = "pub mod addons;\n";
    match text.find("xerxes_engine::include_assets!") {
        Some(at) => format!("{}{line}{}", &text[..at], &text[at..]),
        None => {
            let mut out = text.to_string();
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(line);
            out
        }
    }
}

/// `src/addons/mod.rs` with `pub mod <stem>;` for each add-on, once.
pub fn wire_mod(text: &str, stems: &[String]) -> String {
    let mut out = text.to_string();
    for stem in stems {
        let line = format!("pub mod {stem};");
        if !out.lines().any(|l| l.trim() == line) {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}

/// The project file with each add-on's `plugin` appended to its `LOGIC` list, once. `Err` when
/// the file has no `pub const LOGIC` list.
pub fn wire_project(text: &str, stems: &[String]) -> Result<String, String> {
    const NO_LIST: &str = "the project file has no `pub const LOGIC` list";
    let at = text.find("pub const LOGIC").ok_or(NO_LIST)?;
    let open = at + text[at..].find("&[").ok_or(NO_LIST)? + 2;
    let close = open + text[open..].find("];").ok_or(NO_LIST)?;
    let mut body = text[open..close].trim_end().to_string();
    let mut changed = false;
    for stem in stems {
        let entry = format!("crate::addons::{stem}::plugin");
        if !body.contains(&entry) {
            body.push_str(&format!("\n    {entry},"));
            changed = true;
        }
    }
    if !changed {
        return Ok(text.to_string());
    }
    body.push('\n');
    Ok(format!("{}{}{}", &text[..open], body, &text[close..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_library_is_embedded_and_described() {
        let all = all();
        let hud = all.iter().find(|l| l.path == "hud/hud.rs").expect("hud");
        assert_eq!(hud.needs, ["game/pause.rs"]);
        assert!(hud.summary.starts_with("HUD"), "{}", hud.summary);
        // A summary is a paragraph (every header line up to the blank one), not just line one.
        assert!(
            hud.summary.contains("leaves with the level"),
            "{}",
            hud.summary
        );
        // Every module says what it does, and every `needs:` resolves inside the library.
        for addon in &all {
            assert!(!addon.summary.is_empty(), "{} has no summary", addon.path);
            for need in &addon.needs {
                assert!(
                    all.iter().any(|l| &l.path == need),
                    "{} needs {need}",
                    addon.path
                );
            }
        }
        // They all land in `src/addons/` under unique names.
        let mut names: Vec<_> = all.iter().map(|l| destination(&l.path)).collect();
        let before = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), before, "two add-ons share a file name");
    }

    #[test]
    fn adding_an_addon_brings_what_it_needs() {
        let files = files_for("hud/hud.rs").unwrap();
        let paths: Vec<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(paths, ["src/addons/hud.rs", "src/addons/pause.rs"]);
        assert!(files_for("hud/nope.rs").unwrap_err().contains("no add-on"));
    }

    #[test]
    fn wiring_adds_each_line_once() {
        let stems = ["hud".to_string(), "pause".to_string()];
        let project = "pub const LOGIC: &[fn(&mut ::bevy::app::App)] = &[];\n";
        let wired = wire_project(project, &stems).unwrap();
        assert_eq!(
            wired,
            "pub const LOGIC: &[fn(&mut ::bevy::app::App)] = &[\n    crate::addons::hud::plugin,\n    crate::addons::pause::plugin,\n];\n"
        );
        assert_eq!(wire_project(&wired, &stems).unwrap(), wired, "idempotent");
        assert!(wire_project("pub fn x() {}", &stems).is_err());

        let lib = "pub mod chapar;\n\nxerxes_engine::include_assets!();\n";
        let lib = wire_lib(lib);
        assert_eq!(
            lib,
            "pub mod chapar;\n\npub mod addons;\nxerxes_engine::include_assets!();\n"
        );
        assert_eq!(wire_lib(&lib), lib);

        let modules = wire_mod("//! doc\n", &stems);
        assert_eq!(modules, "//! doc\npub mod hud;\npub mod pause;\n");
        assert_eq!(wire_mod(&modules, &stems), modules);
    }

    #[test]
    fn install_writes_files_and_wires_them_once() {
        let root = std::env::temp_dir().join(format!("xerxes_addons_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "xerxes_engine::include_assets!();\n",
        )
        .unwrap();
        fs::write(
            root.join("assets/game.project.rs"),
            "pub const LOGIC: &[fn(&mut ::bevy::app::App)] = &[];\n",
        )
        .unwrap();

        let first = install(&root, "hud/hud.rs").unwrap();
        assert_eq!(first.added, ["src/addons/hud.rs", "src/addons/pause.rs"]);
        assert!(first.kept.is_empty());
        assert!(root.join("src/addons/pause.rs").is_file());
        let project = fs::read_to_string(root.join("assets/game.project.rs")).unwrap();
        assert!(project.contains("crate::addons::hud::plugin,"));
        assert!(project.contains("crate::addons::pause::plugin,"));
        assert!(
            fs::read_to_string(root.join("src/lib.rs"))
                .unwrap()
                .contains("pub mod addons;")
        );

        // The game's copy is its own: a second install keeps it and wires nothing twice.
        fs::write(root.join("src/addons/hud.rs"), "// changed by the game\n").unwrap();
        let second = install(&root, "hud/hud.rs").unwrap();
        assert_eq!(second.kept, ["src/addons/hud.rs", "src/addons/pause.rs"]);
        assert!(second.added.is_empty());
        let project = fs::read_to_string(root.join("assets/game.project.rs")).unwrap();
        assert_eq!(project.matches("crate::addons::hud::plugin").count(), 1);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn headers_give_summary_and_needs() {
        let known = ["game/pause.rs".to_string(), "hud/hud.rs".to_string()];
        let addon = describe(
            "chr-ctrl/a.rs",
            "//! Template: thing.\n//! More.\n//! needs: pause, hud, nope\nuse x;\n",
            &known,
        );
        assert_eq!(addon.summary, "thing. More.");
        assert_eq!(addon.needs, ["game/pause.rs", "hud/hud.rs", "nope"]);
        assert!(describe("x.rs", "pub fn f() {}", &known).needs.is_empty());
    }
}
