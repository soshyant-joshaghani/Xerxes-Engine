# Architecture

```text
xerxes-engine/
├── engine/          the xerxes_engine crate: library (src/modules: assets, bundles, flow, project, runtime, scene) + the engine app (src/editor)
│   ├── build/       xerxes_build: games' build step, asset naming rules, project scaffold
│   ├── starter/     the built-in empty project every New Project starts from
│   ├── addons/      add-ons: optional gameplay logic a game copies (camera, character, HUD, pause)
│   ├── platform/    android/ (entry + launcher icons), windows/ (exe icon)
│   └── winit/       winit with the Android mouse patch
├── games/           games made with the engine, each its own crate (`game new` games are their own Git repos, gitignored here)
│   └── __templates__/   committed with the engine, read-only in the editor: darius (Darius, in progress), cyrus later, finished games
├── dist/            release builds (gitignored)
├── backend/         Axum: the API series and the multiplayer series (modules/api, modules/multiplayer)
├── tests/           engine/ (bridge, codec, store, flow, assets), backend/ (Axum integration), contract/
├── traefik/ __docs__/ __plans__/
└── __ctrl__/        Python CLI
```

The engine is described in [engine.md](engine.md). In short: a game implements `Scene` (a Bevy world and its bevy_ui HUD) and calls `xerxes_engine::launch`; it never depends on Dioxus. The engine app (the editor) adds a Dioxus UI over its Bevy stage, the same components on every platform: the DOM on the web, Blitz painting over the Bevy window natively. Editor UI and stage exchange data only through the editor bridge.

```text
games/<name>  ──depends on──▶  engine/ (xerxes_engine, default-features = false: Bevy only)
   Scene::build (Bevy systems + bevy_ui HUD)   scene::launch → Bevy App → window / canvas

engine app (feature `editor`)
   editor::launch → host::page(Panel) ⇄ bridge<EditorCommand, EditorSnapshot> ⇄ Bevy stage
                    web: Dioxus DOM + Bevy <canvas> · native: Blitz-painted overlay on the Bevy window
```

The backend is the kit's, plus local-only engine **scaffolding** (`api/engine`: catalog and `<name>.play.localhost`; `api/editor`: project files and jobs for the web editor). Request flow: Route → Service → Repository → PostgreSQL ([wire contract](../../../CONTRACT.md)). Redis is cache and jobs. Games do not call the API for gameplay.

**Boundary:** gameplay (including online rules) lives in `games/<name>/src/`. The multiplayer series (`backend/.../multiplayer/`) is a Colyseus-style **runtime** that hosts those rules in rooms; it is not a second home for darics, modes or scenes ([backend.md](backend.md), [multiplayer.md](multiplayer.md)).

**Web games (planned):** publish splits a Bevy game into a slim boot plus scene packages (code + art) loaded by flow; see [web-progressive.md](web-progressive.md). No SSR; the editor web app stays a separate Dioxus + Bevy page.
