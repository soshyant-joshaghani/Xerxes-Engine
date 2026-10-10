# Deployment

## API

```bat
__ctrl__\xerxes-ctrl.bat backend run prod
__ctrl__\xerxes-ctrl.bat backend run prod --slim   REM no Redis, no worker
__ctrl__\xerxes-ctrl.bat backend stop prod
```

`backend run prod` builds `compose.yml`: backend (the API and multiplayer series together, `SERVICES=all`), worker, Postgres, Redis, Traefik, Adminer. When multiplayer needs its own container, `compose.yml` carries the commented `multiplayer` service to switch on (see [backend.md](backend.md)). Set `DOMAIN` and the URLs in `compose.yml`, and replace the placeholder secrets in `.env`. Outside `ENVIRONMENT=local` the API refuses to start with `SECRET_KEY` or `FIRST_SUPERUSER_PASSWORD` set to `changethis`.

Remote commands use `__ctrl__/servers.json` and the example files under `__ctrl__/safe/`. Real keys stay gitignored. The API applies migrations on start.

## Games

`game publish <name>` writes a release web build to `dist/<name>/web/` (a static site for any static host). `game publish <name> windows` writes the executable and `assets/` to `dist/<name>/windows/`, and `android` writes `dist/<name>/android/<name>.apk`. The engine app publishes with `engine publish <platform>` into `dist/engine/`. Installers and store packages come later, once a game needs them. See [games.md](games.md).
