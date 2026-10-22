#!/usr/bin/env bash
# Usage: start-prod.sh [--slim]   (--slim: no Redis and no worker; backend run prod --slim)
set -euo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"
SLIM=0
[[ "${1:-}" == "--slim" ]] && SLIM=1
if [[ ! -f .env ]]; then
  cp -n .env.example .env 2>/dev/null || true
fi

# Bake + default attestations can fail with: image "…:prod": already exists.
# Prefer classic compose build.
export COMPOSE_BAKE=false
export BUILDX_NO_DEFAULT_ATTESTATIONS=1

source "$(dirname "$0")/lib/ensure-letsencrypt.sh"
docker network inspect traefik-public >/dev/null 2>&1 || docker network create traefik-public

# Clear app tags so export can always write (BuildKit may refuse overwrite).
for img in xerxes-backend:prod; do
  if docker image inspect "$img" >/dev/null 2>&1; then
    docker rmi -f "$img" || true
  fi
done

COMPOSE_ARGS=(-f compose.yml)

# Build first (sole builder), then up without --build so the shared-tag
# worker does not race a registry pull.
docker compose "${COMPOSE_ARGS[@]}" build backend
if [[ "$SLIM" == 1 ]]; then
  # Only what the API needs: --no-deps leaves Redis (a backend dependency) and the worker out.
  docker compose "${COMPOSE_ARGS[@]}" up -d --pull never --no-deps db backend adminer proxy
  echo
  echo "xerxes production stack started (slim: no Redis, no worker)."
else
  docker compose "${COMPOSE_ARGS[@]}" up -d --pull never
  echo
  echo "xerxes production stack started (includes Redis queue worker)."
fi
echo "Update DOMAIN in compose.yml / .env, ACME email in compose.traefik.yml, and DNS before going live."
