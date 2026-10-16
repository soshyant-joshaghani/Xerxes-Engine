# winit, vendored for Xerxes

This is winit 0.30.13 with **two small patches in the Android input handler**, both marked `Xerxes patch` in the source:

1. **Real mouse input** (cursor position, buttons and the wheel). Stock winit does not deliver them on Android, and the editor and games need the mouse there.
2. **Gamepad axes.** Stock winit turns a controller's stick motion into a phantom touch. The patch keeps the left stick, right stick, triggers and hat in a static and exposes them as `winit::platform::android::gamepad_axes()`, which the engine's input manager reads (`engine/src/modules/input/android.rs`). The controller's buttons already arrive as ordinary key events.

The engine and every game use it through `[patch.crates-io]` (`winit = { path = ".../engine/winit" }`); the project scaffold writes that line into new games.

A third patch is designed but not built: Android pointer capture, for a locked FPS-style look ([`__docs__/input.md`](../../__docs__/input.md)).

**When to drop it:** when Bevy moves to a winit with Android mouse support, delete this folder and the `[patch.crates-io]` entries (the engine's, the starter's, and the scaffold's in `build/src/scaffold.rs`). Until then, do not edit it beyond that patch, and do not move this folder: every game's `Cargo.toml` points here.
