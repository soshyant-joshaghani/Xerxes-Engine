//! Reading and changing the simple fields of a project settings file (`<name>.project.rs`)
//! for the Project Settings editor, by looking for the field names in the text. The file is
//! Rust that people also edit by hand, so a change replaces only the value it names and
//! leaves everything else (comments, layout, the level list, the input) exactly as it was. A
//! field the text does not have in the expected shape (`WindowSettings::default()` instead
//! of a struct) is reported as missing, never guessed.

/// What the editor shows of a settings file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Parsed {
    /// The project title (the first `title:`, which is the settings' own).
    pub title: Option<String>,
    /// `None` when the file uses `WindowSettings::default()`.
    pub window: Option<WindowOpts>,
    /// `None` when the file uses `BackendSettings::default()`.
    pub backend: Option<BackendOpts>,
    pub levels: Vec<LevelInfo>,
    pub actions: Vec<ActionInfo>,
    pub preload: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowOpts {
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackendOpts {
    pub dev: String,
    pub publish: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelInfo {
    pub name: String,
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionInfo {
    pub name: String,
    /// The bindings as written, shortened (`KeyA, ArrowLeft`).
    pub bindings: String,
}

/// Just after `key:` (a whole word) at or after `from`.
fn find_key(src: &str, from: usize, key: &str) -> Option<usize> {
    let pattern = format!("{key}:");
    let mut at = from;
    while let Some(found) = src.get(at..)?.find(&pattern) {
        let index = at + found;
        let before = src[..index].chars().next_back();
        if !before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            return Some(index + pattern.len());
        }
        at = index + pattern.len();
    }
    None
}

fn skip_space(src: &str, mut at: usize) -> usize {
    while src[at..].starts_with(|c: char| c.is_whitespace()) {
        at += src[at..].chars().next().map_or(1, char::len_utf8);
    }
    at
}

/// The range of the text inside the quotes of the string after `key:`.
fn string_value(src: &str, from: usize, key: &str) -> Option<(usize, usize)> {
    let at = skip_space(src, find_key(src, from, key)?);
    if !src[at..].starts_with('"') {
        return None;
    }
    let start = at + 1;
    let end = start + src[start..].find('"')?;
    // Escapes are not read: such a value is left alone.
    (!src[start..end].contains('\\')).then_some((start, end))
}

/// The range of the digits after `key:`.
fn number_value(src: &str, from: usize, key: &str) -> Option<(usize, usize)> {
    let start = skip_space(src, find_key(src, from, key)?);
    let len = src[start..].find(|c: char| !c.is_ascii_digit())?;
    (len > 0).then_some((start, start + len))
}

/// The range of `true` or `false` after `key:`.
fn bool_value(src: &str, from: usize, key: &str) -> Option<(usize, usize)> {
    let start = skip_space(src, find_key(src, from, key)?);
    ["true", "false"]
        .iter()
        .find(|word| src[start..].starts_with(**word))
        .map(|word| (start, start + word.len()))
}

/// The range between the braces that follow `marker` (`WindowSettings {`).
fn block(src: &str, marker: &str) -> Option<(usize, usize)> {
    let start = src.find(marker)? + marker.len();
    block_from(src, start)
}

/// The range from `start` (just inside an opening bracket) to its closing bracket.
fn block_from(src: &str, start: usize) -> Option<(usize, usize)> {
    let mut depth = 1usize;
    let mut in_string = false;
    for (offset, c) in src[start..].char_indices() {
        match c {
            '"' => in_string = !in_string,
            '{' | '[' | '(' if !in_string => depth += 1,
            '}' | ']' | ')' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some((start, start + offset));
                }
            }
            _ => {}
        }
    }
    None
}

/// The expression that starts at `start` and ends at the next comma or closing bracket that is
/// not inside brackets of its own.
fn expression(src: &str, start: usize) -> &str {
    let mut depth = 0usize;
    let mut in_string = false;
    for (offset, c) in src[start..].char_indices() {
        match c {
            '"' => in_string = !in_string,
            '{' | '[' | '(' if !in_string => depth += 1,
            '}' | ']' | ')' if !in_string => {
                if depth == 0 {
                    return src[start..start + offset].trim();
                }
                depth -= 1;
            }
            ',' if !in_string && depth == 0 => return src[start..start + offset].trim(),
            _ => {}
        }
    }
    src[start..].trim()
}

fn text(src: &str, range: (usize, usize)) -> String {
    src[range.0..range.1].to_string()
}

pub fn parse(src: &str) -> Parsed {
    let root = src.find("ProjectSettings {").map_or(0, |i| i + 17);
    let title = string_value(src, root, "title").map(|r| text(src, r));

    let window = block(src, "WindowSettings {").and_then(|(start, end)| {
        let inside = &src[..end];
        Some(WindowOpts {
            width: text(src, number_value(inside, start, "width")?)
                .parse()
                .ok()?,
            height: text(src, number_value(inside, start, "height")?)
                .parse()
                .ok()?,
            resizable: text(src, bool_value(inside, start, "resizable")?) == "true",
        })
    });
    let backend = block(src, "BackendSettings {").and_then(|(start, end)| {
        let inside = &src[..end];
        Some(BackendOpts {
            dev: text(src, string_value(inside, start, "dev")?),
            publish: text(src, string_value(inside, start, "publish")?),
        })
    });

    let mut levels = Vec::new();
    for (index, marker) in src.match_indices("LevelDef {") {
        let start = index + marker.len();
        let Some((_, end)) = block_from(src, start) else {
            continue;
        };
        let inside = &src[..end];
        let name = string_value(inside, start, "name")
            .map(|r| text(src, r))
            .unwrap_or_default();
        let mode = find_key(inside, start, "mode")
            .map(|at| expression(inside, skip_space(inside, at)).to_string())
            .unwrap_or_default();
        levels.push(LevelInfo { name, mode });
    }

    let mut actions = Vec::new();
    for (index, marker) in src.match_indices(".action(") {
        let start = index + marker.len();
        let Some(open) = src[start..].find('"').map(|i| start + i + 1) else {
            continue;
        };
        let Some(close) = src[open..].find('"').map(|i| open + i) else {
            continue;
        };
        let bindings = src[close..]
            .find('[')
            .and_then(|i| block_from(src, close + i + 1))
            .map(|(s, e)| {
                src[s..e]
                    .replace("Binding::Key(KeyCode::", "")
                    .replace(')', "")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim_end_matches(',')
                    .to_string()
            })
            .unwrap_or_default();
        actions.push(ActionInfo {
            name: src[open..close].to_string(),
            bindings,
        });
    }

    let preload = find_key(src, 0, "preload_bundles")
        .and_then(|at| {
            let open = at + src[at..].find('[')? + 1;
            block_from(src, open)
        })
        .map(|(s, e)| {
            src[s..e]
                .split('"')
                .skip(1)
                .step_by(2)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    Parsed {
        title,
        window,
        backend,
        levels,
        actions,
        preload,
    }
}

/// A value that can be written between quotes unchanged.
fn plain(value: &str) -> bool {
    !value.contains('"') && !value.contains('\\') && !value.contains('\n')
}

/// Replaces the given ranges (in any order) with their new text.
fn replace_all(src: &str, mut edits: Vec<((usize, usize), String)>) -> String {
    edits.sort_by_key(|(range, _)| std::cmp::Reverse(range.0));
    let mut out = src.to_string();
    for ((start, end), with) in edits {
        out.replace_range(start..end, &with);
    }
    out
}

pub fn set_title(src: &str, title: &str) -> Option<String> {
    if !plain(title) {
        return None;
    }
    let root = src.find("ProjectSettings {").map_or(0, |i| i + 17);
    let range = string_value(src, root, "title")?;
    Some(replace_all(src, vec![(range, title.to_string())]))
}

pub fn set_window(src: &str, window: &WindowOpts) -> Option<String> {
    let (start, end) = block(src, "WindowSettings {")?;
    let inside = &src[..end];
    Some(replace_all(
        src,
        vec![
            (
                number_value(inside, start, "width")?,
                window.width.to_string(),
            ),
            (
                number_value(inside, start, "height")?,
                window.height.to_string(),
            ),
            (
                bool_value(inside, start, "resizable")?,
                window.resizable.to_string(),
            ),
        ],
    ))
}

pub fn set_backend(src: &str, backend: &BackendOpts) -> Option<String> {
    if !plain(&backend.dev) || !plain(&backend.publish) {
        return None;
    }
    let (start, end) = block(src, "BackendSettings {")?;
    let inside = &src[..end];
    Some(replace_all(
        src,
        vec![
            (string_value(inside, start, "dev")?, backend.dev.clone()),
            (
                string_value(inside, start, "publish")?,
                backend.publish.clone(),
            ),
        ],
    ))
}

/// Writes `window: WindowSettings::default()` out as a struct, so its fields can be edited.
pub fn make_window_explicit(src: &str) -> Option<String> {
    let marker = "WindowSettings::default()";
    let at = src.find(marker)?;
    let mut out = src.to_string();
    out.replace_range(
        at..at + marker.len(),
        "WindowSettings {\n            width: 1280,\n            height: 720,\n            resizable: true,\n        }",
    );
    Some(out)
}

/// Writes `backend: BackendSettings::default()` out as a struct, so its fields can be edited.
pub fn make_backend_explicit(src: &str) -> Option<String> {
    let marker = "BackendSettings::default()";
    let at = src.find(marker)?;
    let mut out = src.to_string();
    out.replace_range(
        at..at + marker.len(),
        "BackendSettings {\n            dev: \"http://127.0.0.1:8000\",\n            publish: \"\",\n        }",
    );
    Some(out)
}
