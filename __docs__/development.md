# Development

```bat
__ctrl__\xerxes-ctrl.bat setup-local
__ctrl__\xerxes-ctrl.bat game list             REM games/ A-Z from 1
__ctrl__\xerxes-ctrl.bat engine run web         REM engine app on http://localhost:5100
__ctrl__\xerxes-ctrl.bat template dev darius windows
__ctrl__\xerxes-ctrl.bat template dev darius android     REM device or emulator via adb
```

The first Bevy build takes several minutes; later ones are incremental. Engine and games share `engine/target` (the CLI sets `CARGO_TARGET_DIR`), so Bevy is compiled once per feature set.

The dev stack is infra and the backend. Start the engine app with `engine run web` (served at `engine.localhost`).

```bat
__ctrl__\xerxes-ctrl.bat backend run dev
```

| Surface | URL |
|---------|-----|
| Engine app (`engine run web`) | http://engine.localhost (direct: http://localhost:5100) |
| Published games | http://<name>.play.localhost |
| API docs | http://api.localhost/docs |
| Scalar | http://api.localhost/sdoc |
| Direct API | http://localhost:8000/docs |
| Adminer | http://adminer.localhost |

`backend run dev --slim` skips Redis and the worker. Only one Traefik stack can bind port 80: stop the other proxy, or run `backend run dev --only apps` and use the direct URLs.

The full pipeline (index, dev, publish, new, templates) is in [games.md](games.md). For AI-assisted work point the agent at [AGENTS.md](../AGENTS.md) and [engine.md](engine.md).
