# winit, vendored for Xerxes

This is winit 0.30.13 with **two small patches in the Android input handler**, both marked `Xerxes patch` in the source:

1. **Real mouse input** (cursor position, buttons and the wheel). Stock winit does not deliver them on Android, and the editor and games need the mouse there.
2. **Gamepad axes.** Stock winit turns a controller's stick motion into a phantom touch. The patch keeps the left stick, right stick, triggers and hat in a static and exposes them as `winit::platform::android::gamepad_axes()`, which the engine's input manager reads (`engine/src/modules/input/android.rs`). The controller's buttons already arrive as ordinary key events.

The engine and every game use it through `[patch.crates-io]` (`winit = { path = ".../engine/vendors/winit" }`); the project scaffold writes that line into new games.

A third patch is designed but not built: Android pointer capture, for a locked FPS-style look ([`__docs__/input.md`](../../__docs__/input.md)).

**When to drop it:** when Bevy moves to a winit with Android mouse support, delete this folder and the `[patch.crates-io]` entries (the engine's, the starter's, and the scaffold's in `build/src/scaffold.rs`). Until then, do not edit it beyond that patch, and do not move this folder: every game's `Cargo.toml` points here.

## Trimmed, and how to update it

This copy keeps only what the engine builds: `src/`, `Cargo.toml`, `build.rs`, the licence and the
README. Upstream's examples, tests, benchmarks, docs, changelog and CI files are gone, together
with the Cargo sections that named them (`[[example]]`, `[[test]]`, dev-dependencies) — nothing
here builds them, and they pulled crates into the lockfile for nothing. Prose that upstream keeps
*inside* `src/` goes too (winit compiles its whole release history into the crate as doc comments,
165 KB of markdown behind `#[cfg(doc)]`), with the `mod` that declared it. What is left under
`src/` is code and the few files the code itself includes: winit's web worker scripts, and one
8 KB x11 parser fixture inside a `#[cfg(test)]` module, which costs nothing to carry and would
cost a hand-maintained edit to drop.

The patch above also lives beside the source as `xerxes.patch`, a plain diff against trimmed
upstream, so a new version is a command rather than an afternoon of merging:

```
xerxes-ctrl vendor list                  what is vendored, and whether its patch still applies
xerxes-ctrl vendor update winit VERSION  fetch it, trim it, re-apply xerxes.patch
xerxes-ctrl vendor patch winit           re-record xerxes.patch after editing the source by hand
```

`update` stops and says so if the patch no longer applies: fix the few lines marked `Xerxes patch`
by hand, run `vendor patch winit`, then build a game (`template build cyrus web`) before trusting
it. Edit this copy **only** through that patch, and never move this folder — every game's
`Cargo.toml` points here.
