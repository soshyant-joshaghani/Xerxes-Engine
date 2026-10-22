# blitz-dom, vendored for Xerxes

This is blitz-dom 0.2.4 with **two patches in the tree mutator** (`src/mutator.rs`), each marked
`Xerxes patch` in the source.

**What they change.** Dioxus tells Blitz to insert nodes before an anchor node. Upstream unwraps
the anchor's index in its parent's child list. When the Dioxus tree and the Blitz tree have gone
out of step, that unwrap panics, and on Android the whole editor dies with
`called Option::unwrap() on a None value` at `mutator.rs:411`.

1. **Fallback in `insert_nodes_before`.** An anchor its parent no longer lists appends the new
   nodes to that parent; an anchor with no parent skips the insert. Both cases print a line to
   stderr. This hides the desync rather than fixing it.
2. **Same-parent move in `add_children_to_parent`.** When Dioxus moves a row within its own parent,
   the row is inserted into the parent's list, and then the old-parent cleanup removes every entry
   of it, including the one just inserted. The row's `parent` still points at this parent, which no
   longer lists it, and the next insert before that row misses its anchor. The patch removes the
   row from its slot before inserting it, and skips the old-parent cleanup when the old parent is
   the new one. Found with a device log: the "appended instead" anchor was a row moved within its
   own parent just before.

3. **Freed node ids** (`src/document.rs`, `src/mutator.rs`). Every freed node's id goes into
   `BaseDocument::freed_nodes`, which the embedder drains. Blitz reuses a freed id for the next node,
   so an embedder that keeps ids must forget them; `dioxus-native-dom` does this (see its XERXES.md).

4. **Click only on the same element** (`src/events/mouse.rs`, `handle_mouseup`). Upstream always
   dispatches a click on release, so a finger that scrolled a list and lifted over a button clicked
   it. The patch restores upstream's own commented-out check: a click needs the press and the
   release on the same element (or the same non-anonymous ancestor).

Remove the fallback once this is confirmed on device (no "appended instead" lines in the log while
opening Cyrus projects).

## Trimmed, and how to update it

This copy keeps only what the engine builds: `src/`, `assets/` (read by `include_str!` at build
time) and `Cargo.toml`. Our change is `xerxes.patch`, a diff against the trimmed upstream. To
update: `xerxes-ctrl vendor update blitz-dom <version>`. To re-record the patch after editing the
source by hand: `xerxes-ctrl vendor patch blitz-dom`.

Wired in `engine/Cargo.toml` under `[patch.crates-io]`. Only the engine's editor uses it (the games
do not depend on Dioxus), so the starter, the templates workspace and the scaffold do not need it.
