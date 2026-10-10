//! The engine's add-ons: optional gameplay (`engine/addons/<feature>/<name>.rs`:
//! camera and character controllers, HUD, pause, ...) that a game copies into its own `logic/`
//! and then owns. It is compiled into this crate, so the editor, the backend and the CLI offer
//! the same add-ons with or without the repo.
//!
//! What every game has (scenes, bundles, flow, input, project settings) is engine code, not
//! an add-on.

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

/// Where an add-on lands in a project's `assets/`: always `logic/<file>.rs`.
pub fn destination(path: &str) -> String {
    format!("logic/{}", path.rsplit('/').next().unwrap_or(path))
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
        // They all land in `logic/` under unique names.
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
        assert_eq!(paths, ["logic/hud.rs", "logic/pause.rs"]);
        assert!(files_for("hud/nope.rs").unwrap_err().contains("no add-on"));
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
