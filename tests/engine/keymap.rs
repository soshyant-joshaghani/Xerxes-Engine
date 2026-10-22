//! The keymap: chords parse, the defaults do not clash, a user's change replaces one action's
//! keys and leaves the others at their defaults.

use std::collections::HashMap;

use xerxes_engine::editor::keymap::{Chord, DEFAULTS, bindings_with, label};
use xerxes_engine::editor::protocol::Action;

#[test]
fn chords_parse_in_any_case_and_bad_ones_do_not() {
    let z = Chord::parse("Ctrl+Shift+Z").unwrap();
    assert!(z.ctrl && z.shift && !z.alt);
    assert_eq!(z.key, "Z");
    assert_eq!(Chord::parse("ctrl+shift+z"), Some(z));
    assert_eq!(Chord::parse("cmd+s"), Chord::parse("Ctrl+S"));
    assert_eq!(Chord::parse("F3").unwrap().key, "F3");
    assert_eq!(Chord::parse("delete").unwrap().key, "Delete");
    assert_eq!(Chord::parse("Ctrl+Space").unwrap().key, "Space");
    assert_eq!(Chord::parse("Alt+1").unwrap().key, "1");

    for bad in [
        "",
        "Ctrl",
        "Ctrl+",
        "Ctrl+Shift",
        "A+B",
        "F13",
        "Ctrl+Hyper",
        "++",
    ] {
        assert!(Chord::parse(bad).is_none(), "{bad:?} should not parse");
    }
}

#[test]
fn every_default_parses_and_no_two_actions_share_a_chord() {
    let all = bindings_with(&HashMap::new());
    assert_eq!(all.len(), DEFAULTS.len(), "a default failed to parse");
    for action in Action::ALL {
        assert!(
            all.iter().any(|(a, _)| *a == action),
            "{action:?} has no key"
        );
    }
    for (i, (a, chord)) in all.iter().enumerate() {
        for (b, other) in &all[i + 1..] {
            assert!(
                chord != other || a == b,
                "{a:?} and {b:?} are both bound to {chord:?}"
            );
        }
    }
}

#[test]
fn a_change_replaces_one_action_only() {
    let mut changed = HashMap::new();
    changed.insert("undo".to_string(), vec!["Alt+U".to_string()]);
    changed.insert("save".to_string(), vec!["nonsense+".to_string()]);
    let all = bindings_with(&changed);

    let of = |action| -> Vec<&Chord> {
        all.iter()
            .filter(|(a, _)| *a == action)
            .map(|(_, c)| c)
            .collect()
    };
    // Undo has the one new key, not Ctrl+Z.
    assert_eq!(of(Action::Undo), vec![&Chord::parse("Alt+U").unwrap()]);
    // Redo keeps both defaults.
    assert_eq!(of(Action::Redo).len(), 2);
    // A change that cannot be read leaves the action without a key rather than guessing.
    assert!(of(Action::Save).is_empty());
}

#[test]
fn menus_show_the_first_default() {
    assert_eq!(label(Action::Undo).as_deref(), Some("Ctrl+Z"));
    assert_eq!(label(Action::Palette).as_deref(), Some("F3"));
}
