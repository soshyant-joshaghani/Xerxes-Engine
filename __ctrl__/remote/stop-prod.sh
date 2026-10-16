#!/usr/bin/env bash
# Stop the production stack (containers only — volumes and ./letsencrypt/ are kept).
set -euo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"
COMPOSE_ARGS=(-f compose.yml)
docker compose "${COMPOSE_ARGS[@]}" down --remove-orphans
echo
echo "Production stack stopped."
echo "Certificates: ./letsencrypt/acme.json (unchanged)"
echo "DB/Redis data: still on Docker volumes (use reset-prod.sh to wipe)"
