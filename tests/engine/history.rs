//! Undo and redo: steps, merging of drags and typing, redo dropped by a new edit, the clean
//! (saved) state, the step limit, and jumping in the list.

use xerxes_engine::editor::history::History;

/// A document of one number; each edit sets it.
fn edit(h: &mut History<i32>, label: &str, merge: Option<&str>, before: i32, after: i32) {
    h.record(label, merge, before, after);
}

#[test]
fn undo_and_redo_walk_the_steps() {
    let mut h = History::<i32>::new(50);
    assert!(!h.can_undo() && !h.can_redo());
    edit(&mut h, "one", None, 0, 1);
    edit(&mut h, "two", None, 1, 2);
    assert_eq!(h.undo_label(), Some("two"));

    assert_eq!(h.undo().copied(), Some(1));
    assert_eq!(h.undo().copied(), Some(0));
    assert_eq!(h.undo(), None);
    assert_eq!(h.redo_label(), Some("one"));
    assert_eq!(h.redo().copied(), Some(1));
    assert_eq!(h.redo().copied(), Some(2));
    assert_eq!(h.redo(), None);
    assert_eq!(h.labels(), (vec!["one", "two"], 2));
}

#[test]
fn a_new_edit_drops_what_was_undone() {
    let mut h = History::<i32>::new(50);
    edit(&mut h, "one", None, 0, 1);
    edit(&mut h, "two", None, 1, 2);
    h.undo();
    edit(&mut h, "three", None, 1, 5);
    assert!(!h.can_redo());
    assert_eq!(h.labels(), (vec!["one", "three"], 2));
}

#[test]
fn edits_with_one_merge_key_are_one_step_until_sealed() {
    let mut h = History::<i32>::new(50);
    // A drag: many small edits of the same thing.
    for n in 1..=5 {
        edit(&mut h, "move", Some("drag"), n - 1, n);
    }
    assert_eq!(h.labels().0.len(), 1);
    assert_eq!(
        h.undo().copied(),
        Some(0),
        "undo goes back to before the whole drag"
    );
    assert_eq!(h.redo().copied(), Some(5), "redo to after it");

    // A different key, or no key, starts a step; so does a seal (the pointer was let go).
    edit(&mut h, "other", Some("other"), 5, 6);
    assert_eq!(h.labels().0.len(), 2);
    h.seal();
    edit(&mut h, "other", Some("other"), 6, 7);
    assert_eq!(h.labels().0.len(), 3);
    edit(&mut h, "plain", None, 7, 8);
    edit(&mut h, "plain", None, 8, 9);
    assert_eq!(h.labels().0.len(), 5);
}

#[test]
fn undo_ends_the_merging() {
    let mut h = History::<i32>::new(50);
    edit(&mut h, "type", Some("field"), 0, 1);
    h.undo();
    h.redo();
    edit(&mut h, "type", Some("field"), 1, 2);
    assert_eq!(h.labels().0.len(), 2);
}

#[test]
fn the_saved_state_is_tracked() {
    let mut h = History::<i32>::new(50);
    assert!(h.is_clean(), "a freshly opened scene is clean");
    edit(&mut h, "one", None, 0, 1);
    assert!(!h.is_clean());
    h.undo();
    assert!(
        h.is_clean(),
        "undoing back to the opened scene is clean again"
    );
    h.redo();
    h.mark_saved();
    assert!(h.is_clean());

    // Typing right after a save is a step of its own, so the saved state stays reachable.
    edit(&mut h, "type", Some("field"), 1, 2);
    edit(&mut h, "type", Some("field"), 2, 3);
    assert!(!h.is_clean());
    h.undo();
    assert!(h.is_clean());

    // A new edit after undoing past the saved state loses it.
    h.undo();
    edit(&mut h, "other", None, 0, 9);
    h.redo();
    assert!(!h.is_clean());
    h.undo();
    h.undo();
    assert!(!h.is_clean());
}

#[test]
fn the_oldest_steps_go_when_the_limit_is_reached() {
    let mut h = History::<i32>::new(3);
    for n in 0..5 {
        edit(&mut h, &format!("s{n}"), None, n, n + 1);
    }
    assert_eq!(h.labels(), (vec!["s2", "s3", "s4"], 3));
    assert_eq!(h.undo().copied(), Some(4));
    assert_eq!(h.undo().copied(), Some(3));
    assert_eq!(h.undo().copied(), Some(2));
    assert_eq!(h.undo(), None);
    // The saved state (the start) is gone with the old steps.
    assert!(!h.is_clean());
}

#[test]
fn jump_goes_to_the_state_after_a_number_of_steps() {
    let mut h = History::<i32>::new(50);
    edit(&mut h, "a", None, 0, 1);
    edit(&mut h, "b", None, 1, 2);
    edit(&mut h, "c", None, 2, 3);
    assert_eq!(h.jump(1), Some(1));
    assert_eq!(h.labels().1, 1);
    assert_eq!(h.jump(1), None, "already there");
    assert_eq!(h.jump(3), Some(3));
    assert_eq!(h.jump(0), Some(0));
    assert_eq!(h.jump(99), Some(3), "past the end is the end");
    assert!(History::<i32>::new(5).jump(1).is_none());
}

#[test]
fn clear_forgets_everything() {
    let mut h = History::<i32>::new(50);
    edit(&mut h, "a", None, 0, 1);
    h.clear();
    assert!(!h.can_undo() && !h.can_redo() && h.is_clean());
}
