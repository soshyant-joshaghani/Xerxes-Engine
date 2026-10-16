# Progressive web loading

**Status:** decided, not built. Tracked as stage **2j** in [`__plans__/PROGRESS.md`](../__plans__/PROGRESS.md).

A core feature of every **game** web build: the player downloads a slim boot, then art and code arrive as they move through the scene flow. Games stay Bevy-only (no Dioxus in exports). The loader is a small engine-owned module — no new heavy crates on top of Bevy and Rapier.

The editor's own web build is out of scope here (it already ships as one Dioxus + Bevy page). This document is about `game publish … web` / `template publish … web`.

## Today (what ships)

| Piece | Behaviour |
|-------|-----------|
| Logic, scenes, prefabs, flow | One WASM module. Every scene and every system is compiled into it and downloaded up front. |
| Art | `public/bundles/<id>.zip`. Preload bundles before the first scene; other bundles can load on demand. |
| Hosting | Static site + nginx Dockerfile. No SSR. |

Entering another scene only despawns/spawns from data already in that WASM. It may fetch more art; it never fetches more game code.

## Target (what we want)

Progressive download of **both** art and code, keyed to the project's scene flow (Preload → Splash → MainMenu → Level #, and later Warp).

```text
Browser
  │
  ├─ boot package          engine runtime + flow + menu (or first scene) + preload art
  │
  └─ on LoadLevel / Warp   scene package(s): that scene's code chunk + its art bundles
                           (optional prefetch of likely next packages)
```

| Piece | Behaviour |
|-------|-----------|
| Boot | Small WASM (+ JS glue) that starts Bevy, runs preload, splash and the main menu. Enough to pick a level. |
| Scene package | One unit per scene (or per small group the project declares): prebuilt code for that scene's logic + the art bundles it needs. |
| Manifest | Build-time map from flow edges (level id, scene name, Warp place) → package URLs and their art bundles. |
| Loader | Slim engine code (`modules/` on `wasm32` only): fetch → instantiate → register → extract art → enter the scene. |

Windows and Android keep a single linked binary (progressive loading is a **web** product feature). The same scene/art authorship and flow stay identical on every platform; only the web publish layout splits packages.

## What "compile" means

Rust is compiled at **publish** time. Packages are prebuilt WASM (or equivalent wasm modules the boot can link). At runtime the loader **downloads and instantiates** them (`WebAssembly.compileStreaming` / `instantiate` style). Source `.rs` never ships. There is no rustc and no SSR in the browser.

## Constraints

- **Games are Bevy + engine only.** No Dioxus in game exports (already true; this must stay true).
- **No new heavy dependencies.** Prefer a few hundred lines in the engine and `xerxes_build` over another bundler, WASM runtime crate, or asset framework. Use what the platform and Bevy already give (HTTP fetch / `AssetServer`, WebAssembly JS APIs via `web-sys` if already in the tree, or tiny hand-rolled glue).
- **Same authorship model.** Scenes stay `*.scene.rs`, logic stays in the game crate's modules, art stays in bundles. The split is a **publish layout** and a **runtime loader**, not a second asset format.
- **Flow-driven.** The loader does not guess filenames from the filesystem; it follows the project's levels / Warp graph and the build manifest.
- **Robust over clever.** Cache by content hash, fail loudly with a retryable loading screen, never leave a half-registered scene in the world.

## Build side (`xerxes_build` + publish)

Rough shape when stage 2j lands (details when we implement):

1. Partition the game crate into a **boot** target and **scene packages** (crate features, separate cdylibs, or another split the linker can express without new tooling — pick the lightest option that works with Bevy 0.18).
2. Emit `public/packages/<id>.wasm` (names TBD) plus art under `public/bundles/` as today.
3. Emit a small JSON (or typed Rust) **manifest**: boot → menu; each level id → package id(s) + bundle ids.
4. nginx already gzips `.wasm` / `.js` / `.zip`; extend MIME and cache headers for packages the same way.

Dev web (`game dev … web`) may keep a single module for fast iteration until the split is cheap; publish must always produce the progressive layout.

## Runtime side (engine)

- Extend the existing preload / bundle path so **code packages** are first-class beside art bundles.
- `LoadLevel` / Warp transitions: if the target package is not ready, stay on a loading screen, fetch + instantiate, then spawn the scene.
- Optional prefetch when the menu (or a Warp hub) knows likely next levels.
- Unload policy later (memory on phones); first slice can keep packages once loaded.

## Relation to stage 2e (rendering profiles)

**2e** is phone/desktop × Compat(WebGL2)/Full(WebGPU) × Low/Medium/High and editor Preview. Its "two-tier web builds" share the same boot/loader idea: a thin entry that then pulls the right **renderer profile** package. **2j** owns progressive **scene** packages (code + art by flow). Implement the shared slim loader once; 2e and 2j are two kinds of package the boot may request.

## Non-goals

- Server-side rendering of the game
- Compiling Rust in the browser
- Per-frame streaming of logic
- Progressive loading on Windows/Android in the first version
- New third-party game engines or asset CDNs as hard dependencies

## Proof when built

A Darius (or Cyrus) web publish where the network panel shows: boot + menu art first; opening Qanat (or Rakhsh) downloads that scene's package and art; returning to the menu does not re-download a cached package. Documented in [games.md](games.md) and exercised by a publish smoke check.
