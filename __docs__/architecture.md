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

The backend is the kit's, plus local-only engine dev services (`backend/src/modules/api/engine/`: the game catalog and `<name>.play.localhost`). Request flow: Route (Axum handler) → Service → Repository (trait, SQLx) → PostgreSQL. It speaks the [wire contract](../../../CONTRACT.md): `/api/v1`, `snake_case` JSON, `{"detail": "..."}` errors, form login, JWT HS256. Redis is the cache and the job queue; both degrade softly. Games do not call the API yet. When multiplayer arrives (the M track of Phase 3), the authoritative rooms and state sync live in `backend/src/modules/multiplayer/`, a second series of modules beside the API ([backend.md](backend.md)).
