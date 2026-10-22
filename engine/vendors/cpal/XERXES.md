# cpal, vendored for Xerxes

This is cpal 0.15.3 with **one patch in the web audio backend**
(`src/host/webaudio/mod.rs`), marked `Xerxes patch` in the source.

**What it changes.** The browser audio callback runs for as long as the page does, and every call
it makes into the browser can fail: a context the browser closed or never allowed to start, a
buffer it will not take. Upstream `.expect()`s all of them. A panic in wasm cannot be caught and
poisons the module, so **one failed audio write ends the whole game** — a black page and
`RuntimeError: unreachable` in the console, with the game's own code nowhere near the failure.

The patch reports the first failure to the console and returns from the callback: the stream goes
quiet and the game plays on. A game that loses its sound is a bug to chase; a game that disappears
is a game nobody can play.

Seen in the wild as `TypeError: arg0.copyToChannel is not a function` (cpal's web backend ignores
the stream's `error_callback` entirely — the argument is named `_error_callback` upstream — so
there was no other way to survive it).

`src/lib.rs` also carries `#![allow(deprecated, mismatched_lifetime_syntaxes)]`: built from a
path, upstream's own warnings would otherwise show in every build of every game.

The engine and every game use this copy through `[patch.crates-io]`
(`cpal = { path = ".../engine/vendors/cpal" }`), which is how it reaches Bevy — `bevy_audio` → `rodio` →
`cpal`. The project scaffold writes that line into new games
(`engine/build/src/scaffold.rs`).

**When to drop it:** when the backend upstream stops panicking in the callback (or Bevy moves to
a rodio/cpal that does), delete this folder and the `[patch.crates-io]` entries (the engine's, the
starter's, the templates workspace's, and the scaffold's). Until then, do not edit it beyond that
patch, and do not move this folder: every game's `Cargo.toml` points here.

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
xerxes-ctrl vendor update cpal VERSION  fetch it, trim it, re-apply xerxes.patch
xerxes-ctrl vendor patch cpal           re-record xerxes.patch after editing the source by hand
```

`update` stops and says so if the patch no longer applies: fix the few lines marked `Xerxes patch`
by hand, run `vendor patch cpal`, then build a game (`template build cyrus web`) before trusting
it. Edit this copy **only** through that patch, and never move this folder — every game's
`Cargo.toml` points here.
