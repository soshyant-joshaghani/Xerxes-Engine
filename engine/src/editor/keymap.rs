//! The keymap: which keys run which editor action. Data, not code: the defaults are a table, a
//! user's changes are kept in the preferences (`keymap`, a JSON object of action id to a list
//! of chords) and replace the defaults of that action only, so a new default shipped in a
//! later version still reaches everyone who did not change that action (Renzora does the
//! same). The stage matches the keys; the menus, the command palette and the Preferences
//! editor show and change the same data. A change applies at once ([`revision`] tells the
//! stage to read the bindings again).
//!
//! Tool keys (W E R X, F) and camera keys are the Scene view's own and are not here yet.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{OnceLock, RwLock};

use super::protocol::Action;

/// A key with its modifiers, like `Ctrl+Shift+Z`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    /// The key's name: `A`..`Z`, `0`..`9`, `F1`..`F12`, `Delete`, `Space`, `Escape`, `Enter`,
    /// `Tab`, `Backspace`.
    pub key: String,
}

impl fmt::Display for Chord {
    /// The spelling the preferences keep and the menus show: `Ctrl+Shift+Alt+Key`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ctrl {
            write!(f, "Ctrl+")?;
        }
        if self.shift {
            write!(f, "Shift+")?;
        }
        if self.alt {
            write!(f, "Alt+")?;
        }
        write!(f, "{}", self.key)
    }
}

impl Chord {
    /// Reads `Ctrl+Shift+Z`. Ctrl also means Cmd on a Mac (the stage treats them alike).
    pub fn parse(text: &str) -> Option<Chord> {
        let mut chord = Chord {
            ctrl: false,
            shift: false,
            alt: false,
            key: String::new(),
        };
        for part in text.split('+').map(str::trim) {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "cmd" | "control" => chord.ctrl = true,
                "shift" => chord.shift = true,
                "alt" | "option" => chord.alt = true,
                "" => return None,
                _ => {
                    if !chord.key.is_empty() {
                        return None;
                    }
                    chord.key = canonical_key(part)?;
                }
            }
        }
        (!chord.key.is_empty()).then_some(chord)
    }
}

/// The spelling of a key name, or `None` for a key this keymap does not know.
fn canonical_key(name: &str) -> Option<String> {
    let upper = name.to_ascii_uppercase();
    let single = upper.len() == 1 && upper.chars().all(|c| c.is_ascii_alphanumeric());
    let function = upper
        .strip_prefix('F')
        .and_then(|n| n.parse::<u8>().ok())
        .is_some_and(|n| (1..=12).contains(&n));
    if single || function {
        return Some(upper);
    }
    match upper.as_str() {
        "DELETE" => Some("Delete".into()),
        "SPACE" => Some("Space".into()),
        "ESCAPE" | "ESC" => Some("Escape".into()),
        "ENTER" | "RETURN" => Some("Enter".into()),
        "TAB" => Some("Tab".into()),
        "BACKSPACE" => Some("Backspace".into()),
        _ => None,
    }
}

/// The shipped bindings. An action may have several.
pub const DEFAULTS: &[(Action, &str)] = &[
    (Action::Undo, "Ctrl+Z"),
    (Action::Redo, "Ctrl+Shift+Z"),
    (Action::Redo, "Ctrl+Y"),
    (Action::Save, "Ctrl+S"),
    (Action::Duplicate, "Shift+D"),
    (Action::Delete, "Delete"),
    (Action::Palette, "F3"),
    (Action::ToggleMaximize, "Ctrl+Space"),
];

/// The bindings in effect: the defaults, with an action the user changed taking theirs.
pub fn bindings_with(overrides: &HashMap<String, Vec<String>>) -> Vec<(Action, Chord)> {
    let mut out = Vec::new();
    for action in Action::ALL {
        let texts = effective_with(action, overrides);
        out.extend(
            texts
                .iter()
                .filter_map(|t| Chord::parse(t))
                .map(|chord| (action, chord)),
        );
    }
    out
}

/// The chords of an action as text: the user's, else the defaults.
pub fn effective_with(action: Action, overrides: &HashMap<String, Vec<String>>) -> Vec<String> {
    match overrides.get(action.id()) {
        Some(list) => list.clone(),
        None => DEFAULTS
            .iter()
            .filter(|(a, _)| *a == action)
            .map(|(_, c)| (*c).to_string())
            .collect(),
    }
}

/// Reads a list of chords as a person types it (`Ctrl+Z, Ctrl+Y`). Every chord must be a key
/// this keymap knows; the result is spelled the way the preferences keep it. An empty text is
/// an empty list (the action has no key).
pub fn parse_list(text: &str) -> Result<Vec<String>, String> {
    text.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            Chord::parse(part)
                .map(|c| c.to_string())
                .ok_or_else(|| format!("`{part}` is not a key (like Ctrl+Shift+Z, F3, Delete)"))
        })
        .collect()
}

/// Pairs of actions that share a chord: `(first, second, chord)`.
pub fn conflicts_with(overrides: &HashMap<String, Vec<String>>) -> Vec<(Action, Action, String)> {
    let all = bindings_with(overrides);
    let mut out = Vec::new();
    for (i, (a, chord)) in all.iter().enumerate() {
        for (b, other) in &all[i + 1..] {
            if a != b && chord == other {
                out.push((*a, *b, chord.to_string()));
            }
        }
    }
    out
}

fn store() -> &'static RwLock<HashMap<String, Vec<String>>> {
    static STORE: OnceLock<RwLock<HashMap<String, Vec<String>>>> = OnceLock::new();
    STORE.get_or_init(|| {
        RwLock::new(
            super::prefs::load("keymap")
                .and_then(|text| serde_json::from_str(&text).ok())
                .unwrap_or_default(),
        )
    })
}

static REVISION: AtomicU64 = AtomicU64::new(1);

/// Changes whenever the keymap does: whoever caches the bindings reads them again.
pub fn revision() -> u64 {
    REVISION.load(Ordering::Relaxed)
}

/// The user's changes now.
pub fn overrides() -> HashMap<String, Vec<String>> {
    store().read().map(|m| m.clone()).unwrap_or_default()
}

/// Replaces the user's changes: kept in the preferences and in effect at once.
pub fn set_overrides(overrides: HashMap<String, Vec<String>>) {
    super::prefs::save(
        "keymap",
        &serde_json::to_string(&overrides).unwrap_or_default(),
    );
    if let Ok(mut current) = store().write() {
        *current = overrides;
    }
    REVISION.fetch_add(1, Ordering::Relaxed);
}

/// The bindings in effect now.
pub fn bindings() -> Vec<(Action, Chord)> {
    bindings_with(&overrides())
}

/// How an action's first binding reads in a menu (`Ctrl+Z`), if it has one.
pub fn label(action: Action) -> Option<String> {
    effective_with(action, &overrides()).into_iter().next()
}
