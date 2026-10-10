# winit, vendored for Xerxes

This is winit 0.30.13 with **one patch**: real mouse input on Android (cursor position, buttons and the wheel). It is marked `Xerxes patch` in the source. Stock winit does not deliver them on Android, and the editor and games need the mouse there.

The engine and every game use it through `[patch.crates-io]` (`winit = { path = ".../engine/winit" }`); the project scaffold writes that line into new games.

A second patch is designed but not built: Android pointer capture, for a locked FPS-style look ([`__docs__/input.md`](../../__docs__/input.md)).

**When to drop it:** when Bevy moves to a winit with Android mouse support, delete this folder and the `[patch.crates-io]` entries (the engine's, the starter's, and the scaffold's in `build/src/scaffold.rs`). Until then, do not edit it beyond that patch, and do not move this folder: every game's `Cargo.toml` points here.
