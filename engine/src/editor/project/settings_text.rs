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
    /// `None` when the file uses `PhysicsSettings::default()`.
    pub physics: Option<PhysicsOpts>,
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
pub struct PhysicsOpts {
    pub gravity_x: f32,
    pub gravity_y: f32,
    /// Layer names as written (`Default, Toy`), empty when the array is empty.
    pub layers: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelInfo {
    pub name: String,
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionInfo {
    pub name: String,
    /// `button`, `axis` or `vector2` (`action` is a button).
    pub kind: String,
    /// The bindings as written, shortened (`Key(KeyA), Key(ArrowLeft), PadStick(Left)`).
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
    let physics = block(src, "PhysicsSettings {").and_then(|(start, end)| {
        let inside = &src[..end];
        let gravity_at = find_key(inside, start, "gravity")?;
        let g_start = skip_space(inside, gravity_at);
        if !inside[g_start..].starts_with('[') {
            return None;
        }
        let (g_inner_start, g_inner_end) = block_from(inside, g_start + 1)?;
        let parts: Vec<&str> = inside[g_inner_start..g_inner_end]
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() != 2 {
            return None;
        }
        let gravity_x = parts[0].parse().ok()?;
        let gravity_y = parts[1].parse().ok()?;
        let layers_at = find_key(inside, start, "layers")?;
        let layers_start = skip_space(inside, layers_at);
        let layers = if inside[layers_start..].starts_with("&[") {
            let (inner_start, inner_end) = block_from(inside, layers_start + 2)?;
            inside[inner_start..inner_end]
                .split(',')
                .filter_map(|s| {
                    let s = s.trim();
                    s.strip_prefix('"')
                        .and_then(|s| s.strip_suffix('"'))
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::new()
        };
        Some(PhysicsOpts {
            gravity_x,
            gravity_y,
            layers,
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

    // `.vector2("move", [...])`, `.axis(..)`, `.button(..)` and the older `.action(..)`, in the
    // order they are written.
    let mut found: Vec<(usize, &str, &str)> = Vec::new();
    for (marker, kind) in [
        (".vector2(", "vector2"),
        (".axis(", "axis"),
        (".button(", "button"),
        (".action(", "button"),
    ] {
        found.extend(
            src.match_indices(marker)
                .map(|(i, m)| (i + m.len(), kind, m)),
        );
    }
    found.sort_by_key(|(start, _, _)| *start);
    let mut actions = Vec::new();
    for (start, kind, _) in found {
        let Some(open) = src[start..].find('"').map(|i| start + i + 1) else {
            continue;
        };
        let Some(close) = src[open..].find('"').map(|i| open + i) else {
            continue;
        };
        let bindings = src[close..]
            .find('[')
            .and_then(|i| block_from(src, close + i + 1))
            .map(|(s, e)| shorten_bindings(&src[s..e]))
            .unwrap_or_default();
        actions.push(ActionInfo {
            name: src[open..close].to_string(),
            kind: kind.to_string(),
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
        physics,
        levels,
        actions,
        preload,
    }
}

/// A binding list as one line: `Binding::Key(KeyCode::KeyA), Binding::Pad(GamepadButton::South)`
/// becomes `Key(KeyA), Pad(South)`.
fn shorten_bindings(list: &str) -> String {
    // Comments between bindings are not part of them.
    let list: String = list
        .lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join(" ");
    let mut text = list.replace("Binding::", "");
    for prefix in [
        "KeyCode::",
        "GamepadButton::",
        "GamepadAxis::",
        "MouseButton::",
        "PadStick::",
    ] {
        text = text.replace(prefix, "");
    }
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(",)", ")")
        .trim_end_matches(',')
        .to_string()
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

/// Replaces the gravity and layers of a written `PhysicsSettings { ... }` block.
pub fn set_physics(src: &str, physics: &PhysicsOpts) -> Option<String> {
    if !physics
        .layers
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .all(plain)
    {
        return None;
    }
    let (start, end) = block(src, "PhysicsSettings {")?;
    let inside = &src[..end];
    let gravity_at = find_key(inside, start, "gravity")?;
    let g_start = skip_space(inside, gravity_at);
    if !inside[g_start..].starts_with('[') {
        return None;
    }
    let (g_inner_start, g_inner_end) = block_from(inside, g_start + 1)?;
    let layers_at = find_key(inside, start, "layers")?;
    let layers_start = skip_space(inside, layers_at);
    if !inside[layers_start..].starts_with("&[") {
        return None;
    }
    let (layers_inner_start, layers_inner_end) = block_from(inside, layers_start + 2)?;
    let layers = physics
        .layers
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| format!("\"{s}\""))
        .collect::<Vec<_>>()
        .join(", ");
    Some(replace_all(
        src,
        vec![
            (
                (g_inner_start, g_inner_end),
                format!("{}, {}", physics.gravity_x, physics.gravity_y),
            ),
            ((layers_inner_start, layers_inner_end), layers),
        ],
    ))
}

/// Writes `physics: PhysicsSettings::default()` out as a struct, so its fields can be edited.
pub fn make_physics_explicit(src: &str) -> Option<String> {
    let marker = "PhysicsSettings::default()";
    let at = src.find(marker)?;
    let mut out = src.to_string();
    out.replace_range(
        at..at + marker.len(),
        "PhysicsSettings {\n            gravity: [0.0, 0.0],\n            layers: &[\"Default\"],\n        }",
    );
    Some(out)
}
