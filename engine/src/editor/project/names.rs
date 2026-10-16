//! Project and scene names, checked as the user types. Project names follow the shared
//! scaffold's rule (`xerxes_build::scaffold::name_problem`), so a name the editor accepts is
//! one every way of creating a project accepts.

pub use xerxes_build::scaffold::name_problem as project_problem;

/// What typing produces: lowercase, spaces and `_` as `-`.
pub fn normalize(typed: &str) -> String {
    typed.trim_start().to_lowercase().replace([' ', '_'], "-")
}

/// What typing a scene name produces: lowercase, spaces and `-` as `_` (a module name).
pub fn normalize_scene(typed: &str) -> String {
    typed.trim_start().to_lowercase().replace([' ', '-'], "_")
}

/// `base`, or `base-2`, `base-3`, ... (`sep` between): the first one `taken` does not have.
pub fn unique(base: &str, sep: char, taken: &[&str]) -> String {
    if !taken.contains(&base) {
        return base.to_string();
    }
    (2..)
        .map(|n| format!("{base}{sep}{n}"))
        .find(|name| !taken.contains(&name.as_str()))
        .unwrap_or_default()
}

/// Why `stem` cannot name a new scene file (`scenes/<stem>.scene.rs`), or `None`. Scene
/// files become Rust modules, so the rule is a module name's.
pub fn scene_problem(stem: &str, existing: &[&str]) -> Option<String> {
    if stem.is_empty() {
        return Some("Type a name.".into());
    }
    if !stem.starts_with(|c: char| c.is_ascii_lowercase()) {
        return Some("Start with a letter (a-z).".into());
    }
    if !stem
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        return Some("Use only a-z, 0-9 and _.".into());
    }
    if existing.contains(&stem) {
        return Some(format!("scenes/{stem}.scene.rs already exists."));
    }
    None
}
