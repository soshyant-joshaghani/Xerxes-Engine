//! Changing the keymap at run time: kept in the preferences, in effect at once, and the
//! revision moves so whoever caches the bindings reads them again. One test in this binary
//! changes the preferences, and it points them at a temporary folder (a process-wide setting)
//! so it cannot touch the user's.

use std::collections::HashMap;

use xerxes_engine::editor::keymap::{
    Chord, bindings, conflicts_with, effective_with, label, overrides, parse_list, revision,
    set_overrides,
};
use xerxes_engine::editor::protocol::Action;

#[test]
fn a_change_is_kept_in_effect_and_announced() {
    let dir = std::env::temp_dir().join(format!("xerxes-keymap-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    // SAFETY: set before anything in this binary reads the preferences, and the other test
    // here never touches them.
    unsafe { std::env::set_var("XERXES_CONFIG", &dir) };

    assert_eq!(label(Action::Undo).as_deref(), Some("Ctrl+Z"));
    assert!(overrides().is_empty());
    let before = revision();

    let mut changed = HashMap::new();
    changed.insert(
        "undo".to_string(),
        parse_list("alt+u, Ctrl+Shift+U").unwrap(),
    );
    set_overrides(changed);

    assert!(revision() > before);
    assert_eq!(label(Action::Undo).as_deref(), Some("Alt+U"));
    let undo: Vec<String> = bindings()
        .into_iter()
        .filter(|(a, _)| *a == Action::Undo)
        .map(|(_, c)| c.to_string())
        .collect();
    assert_eq!(undo, ["Alt+U", "Ctrl+Shift+U"]);
    // Redo kept its defaults.
    assert_eq!(effective_with(Action::Redo, &overrides()).len(), 2);
    // It was written, so the next start has it.
    let saved = std::fs::read_to_string(dir.join("keymap.json")).unwrap();
    assert!(saved.contains("Alt+U"), "{saved}");

    // Clearing the changes brings the defaults back.
    set_overrides(HashMap::new());
    assert_eq!(label(Action::Undo).as_deref(), Some("Ctrl+Z"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn chords_are_spelled_one_way_lists_are_checked_and_clashes_are_found() {
    assert_eq!(
        Chord::parse("shift+ctrl+z").unwrap().to_string(),
        "Ctrl+Shift+Z"
    );
    assert_eq!(
        Chord::parse("alt+ctrl+shift+f3").unwrap().to_string(),
        "Ctrl+Shift+Alt+F3"
    );
    assert_eq!(Chord::parse("delete").unwrap().to_string(), "Delete");

    assert_eq!(parse_list("").unwrap(), Vec::<String>::new());
    assert_eq!(parse_list(" ctrl+z , , F3 ").unwrap(), ["Ctrl+Z", "F3"]);
    let err = parse_list("Ctrl+Z, Hyper+Q").unwrap_err();
    assert!(err.contains("Hyper+Q"), "{err}");

    assert!(conflicts_with(&HashMap::new()).is_empty());
    let mut clash = HashMap::new();
    clash.insert("save".to_string(), vec!["Ctrl+Z".to_string()]);
    let found = conflicts_with(&clash);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].2, "Ctrl+Z");
    assert!(
        (found[0].0, found[0].1) == (Action::Undo, Action::Save)
            || (found[0].0, found[0].1) == (Action::Save, Action::Undo)
    );
}
