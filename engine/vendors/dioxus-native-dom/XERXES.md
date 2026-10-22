# dioxus-native-dom, vendored for Xerxes

This is dioxus-native-dom 0.7.10 with **one patch in the mutation writer**
(`src/mutation_writer.rs`), marked `Xerxes patch` in the source. It works with the patched
`blitz-dom` (`engine/vendors/blitz-dom`), which records the ids of freed nodes.

**What it changes.** Dioxus keeps a map from its element ids to blitz node ids. Blitz frees a node
when its subtree is removed, and then reuses the freed id for the next node created. Upstream never
clears the map entry for a freed node, so an element can keep pointing at an id that now belongs to
another node. Reassigning that element then drops the other node, and a later lookup through it
panics with `invalid key` in `node_at_path`.

The patch adds `forget_freed`, which clears every map entry for a node blitz has freed. It runs at
the start of each mutation, before any lookup or reuse of an id.

Remove the patch once upstream clears freed nodes from its element map, or once the mapping is
replaced by something that does not reuse ids.

## Trimmed, and how to update it

This copy keeps only `src/` and `Cargo.toml`. Our change is `xerxes.patch`, a diff against the
trimmed upstream. To update: `xerxes-ctrl vendor update dioxus-native-dom <version>`. To re-record
the patch after editing the source by hand: `xerxes-ctrl vendor patch dioxus-native-dom`.

Wired in `engine/Cargo.toml` under `[patch.crates-io]`, next to `blitz-dom`, because blitz-dom and
dioxus-native-dom are used together by the engine's editor.
