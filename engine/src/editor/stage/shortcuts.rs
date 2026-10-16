//! The keymap's keys (`editor::keymap`), matched against the keyboard. An action that fires is
//! published to the UI (with the area under the pointer, for the actions that apply to one),
//! which does the work: the UI owns the document. Keys go to the UI's own text fields while
//! one has focus (`UiFocus`), so typing never fires an action.

use bevy::prelude::*;

use super::camera::{ActiveLayout, ActiveWorkspace, SceneInput};
use crate::editor::host::UiFocus;
use crate::editor::keymap::{self, Chord};
use crate::editor::layout::{self, AreaId};
use crate::editor::protocol::Action;

/// The bindings in effect, and the last action that fired (numbered, so pressing the same key
/// twice reaches the UI twice).
#[derive(Resource)]
pub struct Shortcuts {
    bindings: Vec<(Action, Chord)>,
    /// The keymap's revision these bindings were read at.
    revision: u64,
    pub fired: Option<(Action, u64, Option<AreaId>)>,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            bindings: keymap::bindings(),
            revision: keymap::revision(),
            fired: None,
        }
    }
}

const LETTERS: [KeyCode; 26] = [
    KeyCode::KeyA,
    KeyCode::KeyB,
    KeyCode::KeyC,
    KeyCode::KeyD,
    KeyCode::KeyE,
    KeyCode::KeyF,
    KeyCode::KeyG,
    KeyCode::KeyH,
    KeyCode::KeyI,
    KeyCode::KeyJ,
    KeyCode::KeyK,
    KeyCode::KeyL,
    KeyCode::KeyM,
    KeyCode::KeyN,
    KeyCode::KeyO,
    KeyCode::KeyP,
    KeyCode::KeyQ,
    KeyCode::KeyR,
    KeyCode::KeyS,
    KeyCode::KeyT,
    KeyCode::KeyU,
    KeyCode::KeyV,
    KeyCode::KeyW,
    KeyCode::KeyX,
    KeyCode::KeyY,
    KeyCode::KeyZ,
];

const DIGITS: [KeyCode; 10] = [
    KeyCode::Digit0,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

const FUNCTION: [KeyCode; 12] = [
    KeyCode::F1,
    KeyCode::F2,
    KeyCode::F3,
    KeyCode::F4,
    KeyCode::F5,
    KeyCode::F6,
    KeyCode::F7,
    KeyCode::F8,
    KeyCode::F9,
    KeyCode::F10,
    KeyCode::F11,
    KeyCode::F12,
];

/// The key a chord's key name stands for (names as `keymap::Chord` spells them).
fn key_code(name: &str) -> Option<KeyCode> {
    let first = name.chars().next()?;
    if name.len() == 1 && first.is_ascii_uppercase() {
        return Some(LETTERS[(first as u8 - b'A') as usize]);
    }
    if name.len() == 1 && first.is_ascii_digit() {
        return Some(DIGITS[(first as u8 - b'0') as usize]);
    }
    if let Some(n) = name.strip_prefix('F').and_then(|n| n.parse::<usize>().ok()) {
        return FUNCTION.get(n.checked_sub(1)?).copied();
    }
    match name {
        "Delete" => Some(KeyCode::Delete),
        "Space" => Some(KeyCode::Space),
        "Escape" => Some(KeyCode::Escape),
        "Enter" => Some(KeyCode::Enter),
        "Tab" => Some(KeyCode::Tab),
        "Backspace" => Some(KeyCode::Backspace),
        _ => None,
    }
}

pub fn shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<UiFocus>,
    input: Res<SceneInput>,
    layout: Res<ActiveLayout>,
    workspace: Res<ActiveWorkspace>,
    mut shortcuts: ResMut<Shortcuts>,
) {
    if shortcuts.revision != keymap::revision() {
        shortcuts.bindings = keymap::bindings();
        shortcuts.revision = keymap::revision();
    }
    if focus.0 {
        return;
    }
    let ctrl = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    let hit = shortcuts
        .bindings
        .iter()
        .find(|(_, chord)| {
            chord.ctrl == ctrl
                && chord.shift == shift
                && chord.alt == alt
                && key_code(&chord.key).is_some_and(|key| keys.just_pressed(key))
        })
        .map(|(action, _)| *action);
    let Some(action) = hit else { return };

    // The area under the pointer, for the actions that apply to an area.
    let area = input.window_cursor.and_then(|cursor| {
        let content = layout.0.content();
        let solved = layout::solve(
            &workspace.0,
            layout::Rect::new(0.0, 0.0, content.w, content.h),
        );
        solved
            .areas
            .iter()
            .find(|p| p.rect.contains(cursor.x - content.x, cursor.y - content.y))
            .map(|p| p.area.id)
    });
    let seq = shortcuts.fired.map_or(1, |(_, n, _)| n + 1);
    shortcuts.fired = Some((action, seq, area));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_default_binding_has_a_key() {
        for (action, chord) in keymap::bindings() {
            assert!(
                key_code(&chord.key).is_some(),
                "{action:?} is bound to {chord:?}, which no key matches"
            );
        }
    }
}
