# Progressive web loading

**Status:** the **art half is built** (2026-10-19). **Code packages are not, and will not be** —
measured on 2026-10-19 and decided against; the measurement and what to do instead are below.
Tracked as stage **2j** in [`__plans__/PROGRESS.md`](../__plans__/PROGRESS.md).

A core feature of every **game** web build: the player downloads a slim boot, then art and code arrive as they move through the scene flow. Games stay Bevy-only (no Dioxus in exports). The loader is a small engine-owned module — no new heavy crates on top of Bevy and Rapier.

The editor's own web build is out of scope here (it already ships as one Dioxus + Bevy page). This document is about `game publish … web` / `template publish … web`.

## Today (what ships)

| Piece | Behaviour |
|-------|-----------|
| Logic, scenes, prefabs, flow | One WASM module. Every scene and every system is compiled into it and downloaded up front. |
| Art | `public/bundles/<id>.zip`, **downloaded per scene**: the flow's `Loading` step fetches what the next scene draws and waits for it (below). `preload_bundles` is only what the menu itself needs. |
| Hosting | Static site + nginx Dockerfile. No SSR. |

Entering another scene only despawns/spawns from data already in that WASM. It fetches that scene's art; it never fetches more game code.

### Built: art by scene, with a gate and a prefetch

The flow is `Preload → Splash → MainMenu → **Loading** → Level` (`engine/src/modules/flow`).

1. **A scene says what it needs, by itself.** Every component that loads a file answers `ComponentDef::needs()` (`SpriteDef`'s image, `MaterialDef`'s texture, `ModelDef`'s model), and `assets::scene_needs` walks a scene's objects — prefabs resolved — to collect them. There is no list to keep in step with the art.
2. **`Loading` has them in memory before the level starts.** It asks for the bundle each file ships in (`Bundles::bundle_of`, from the generated manifest), then for the files themselves once that bundle is extracted, and waits behind a progress bar. It holds the handles (`flow::Loaded`), so the **same** handle the scene will use is the one that was warmed: nothing loads twice, and a second visit starts at once with no screen. A level whose files are already there passes straight through.
3. **A file that cannot be loaded does not trap the player.** It is named in an error and the level starts without it.
3. **The screen says which step it is on,** because the two feel different and only one is the
   network: `Downloading Siavash…  7.5 MB` while bundles are coming, then
   `Preparing Siavash…  0 / 1` while the asset server decodes what arrived. When the level
   starts, the log says how long it took (`Siavash: 1 file(s) ready in 0.6 s`), so a level that
   takes too long says so without a profiler.
4. **Prefetch.** `PrefetchLevel(n)` fetches a level's files without going to it. The engine sends one for the level button **under the pointer**, so choosing from a menu usually finishes the download before the click. Warp will send them for where its links lead.
5. **Bundles are the unit of download.** A heavy file belongs in its scene's bundle, not in `core`: Cyrus's test character names `.bundle("siavash")`, so the menu page downloads no art at all and Siavash's 7.5 MB arrives when Siavash is asked for (or hovered).

This is what the player feels on every platform: no model popping into a level that has already begun, and on the web no download for scenes never played. What is **not** in it: the game's code is still one WASM module.

## Measured: where the megabytes actually are

Before building code packages, we weighed one (`xerxes-ctrl template weigh cyrus`, which prints
this for any web build — `__ctrl__/utils/game/weigh.py` walks the module's sections and its name
section). Cyrus's release web publish, 21 scenes and three playable games:

| | |
|---|---|
| The shipped module | **34.57 MB** (code 29.24 MB, data 4.15 MB, DWARF 0.77 MB) |
| What the player downloads | **8.74 MB** gzipped |
| Art, downloaded per scene | 7.54 MB (`siavash.zip`), and only when Siavash is opened |

And of the 32.07 MB of code in the module rustc wrote:

| Who wrote it | | |
|---|---|---|
| **The game's own code** (21 scenes, three games, the rules crate) | **0.15 MB** | **0.5%** |
| The engine (`xerxes_*`) | 0.06 MB | 0.2% |
| Bevy | 18.20 MB | 56.7% (`bevy_ecs` alone 24.6%, `bevy_reflect` 10.0%) |
| The renderer (wgpu, naga) | 1.80 MB | 5.6% |
| Physics (rapier, parry) | 1.15 MB | 3.6% |

**That is the decision.** A scene package would move about **7 KB** of code, and every scene
together about 0.15 MB — roughly **0.4% of the download**, a few tens of KB once gzipped. The
module is the Bevy runtime; the scenes are a rounding error in it.

### So: no scene code packages

The tooling exists — `dx build --wasm-split` (experimental) splits a linked module into chunks
behind `#[wasm_split]` functions — and it is the right lever for an app whose own code is most of
its bundle. It is the wrong lever here: it would buy 0.4%, in exchange for an experimental flag, a
new dependency, lazily-loaded function boundaries through the flow, and systems that have to be
registered into a running `App`. Nothing can split the part that is actually heavy: Bevy's code is
reached from everywhere, and nothing in the toolchain can share one Bevy runtime across several
wasm modules (separate cdylibs each link their own copy, which is worse than one module).

**What does reduce the first download, in order of what it is worth:**

1. **Compress it properly.** 34.57 MB raw is 9.87 MB at nginx's default gzip level and 8.74 MB at
   level 6, so the shipped `nginx.conf.template` now sets `gzip_comp_level 6` — about 1.1 MB, 25x
   what splitting every scene would have saved. Pre-compressing (`gzip_static`, brotli) is the next
   step if it is ever worth the build time.
2. **Carry less Bevy.** `bevy_reflect` + `erased_serde` + `ron` are ~15% of the code and come in
   with scene (de)serialisation and glTF; the renderer is another ~6%. Trimming features per
   profile is stage **2e** (Compat/Full, phone/desktop), which is where the next real megabytes
   are, and it is the same slim-loader idea applied to the *runtime* instead of to scenes.
3. **Keep art per scene** (done): art is where a game's own megabytes are — one character model is
   7.54 MB, fifty times all of Cyrus's code.

If a game ever grows code of its own worth splitting (megabytes, not kilobytes), the `Loading`
step is already the one gate a code package would hang from: it would be another need kind beside
the art, asked for in the same place. Re-run `weigh` and the number will say when.

### Where the wait moved (and the knob that moves it back)

The gate does not make a game slower; it moves the wait to where it can be seen. Before it, Cyrus
preloaded the bundle with the character in it, so the **boot** paid 7.5 MB and the click into
Siavash was instant — and then the model popped in a second later, mid-run. Now the boot
downloads nothing, the menu is up at once, and the same bytes are fetched when Siavash is asked
for. Measured on the dev build: **0.3 s** from click to playing when the tile was hovered first
(the prefetch had the bundle and had started the decode), **0.6 s** on a fresh page without that
head start, plus whatever the 7.5 MB costs on a real network.

So the trade is per game, and it is one line in the project file:

- **Nothing in `preload_bundles`** (Cyrus today): the page opens fast, a level pays for its own
  art the first time it is opened, and a hover usually pays it before the click does.
- **A bundle named in `preload_bundles`**: the boot waits for it behind the Preload bar, and every
  level that uses it starts instantly, for ever. Right for the art the first level always needs.

Either way the level starts whole; what changes is which screen the waiting happens on.

## Target (what we wanted)

Progressive download of **both** art and code, keyed to the project's scene flow (Preload → Splash → MainMenu → Level #, and later Warp). The art half is built; the code half is the one the measurement above closed.

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

**Not built** (see the measurement): this was the shape a code split would have taken.

1. Partition the game crate into a **boot** target and **scene packages** (crate features, separate cdylibs, or another split the linker can express without new tooling — pick the lightest option that works with Bevy 0.18).
2. Emit `public/packages/<id>.wasm` (names TBD) plus art under `public/bundles/` as today.
3. Emit a small JSON (or typed Rust) **manifest**: boot → menu; each level id → package id(s) + bundle ids.
4. nginx already gzips `.wasm` / `.js` / `.zip`; extend MIME and cache headers for packages the same way.

Dev web (`game dev … web`) may keep a single module for fast iteration until the split is cheap; publish must always produce the progressive layout.

## Runtime side (engine)

Built: the `Loading` step, per-scene art fetching, the handle-holding `Loaded` resource,
`PrefetchLevel`, and hover prefetch in the menu. What is left here:

- Unload policy (memory on phones); today art stays once loaded, which is what makes a second
  visit instant. A phone with many heavy scenes will want a budget.
- A code-package need kind **only if** a game's own code ever grows to matter (above).

## Relation to stage 2e (rendering profiles)

**2e** is phone/desktop × Compat(WebGL2)/Full(WebGPU) × Low/Medium/High and editor Preview. Its "two-tier web builds" share the same boot/loader idea: a thin entry that then pulls the right **renderer profile** package. **2j** owns progressive **scene** packages (code + art by flow). Implement the shared slim loader once; 2e and 2j are two kinds of package the boot may request.

## Non-goals

- Server-side rendering of the game
- Compiling Rust in the browser
- Per-frame streaming of logic
- Progressive loading on Windows/Android in the first version
- New third-party game engines or asset CDNs as hard dependencies
- **Splitting the game's code per scene** (measured 2026-10-19: 0.5% of the module)

## Proof (art packages, 2026-10-19)

Checked in a browser on a Cyrus web dev build: the menu boots with **no** `.zip` request; hovering
the Siavash tile fetches `bundles/siavash.zip`; clicking before that finishes shows
“Loading Siavash… 0 / 1” and no level; the run then starts with the model already there.
Re-measure any build with `xerxes-ctrl game|template weigh <name>`.
