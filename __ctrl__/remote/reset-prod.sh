#!/usr/bin/env bash
# Wipe app build images + Postgres/Redis data volumes; keep Traefik base image and SSL on disk.
set -euo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"

source "$(dirname "$0")/lib/ensure-letsencrypt.sh"

echo "Backing up SSL outside repo..."
# shellcheck source=lib/acme-backup.sh
source "$(dirname "$0")/lib/acme-backup.sh"
acme_backup_to_parent || true

echo "Stopping production stack..."
docker compose down --remove-orphans

echo "Removing Postgres and Redis data volumes..."
for vol in xerxes_db-data xerxes_redis-data; do
  if docker volume inspect "$vol" >/dev/null 2>&1; then
    docker volume rm "$vol"
    echo "  removed $vol"
  fi
done

echo "Removing application images (will rebuild on next start)..."
for img in xerxes-backend:prod; do
  if docker image inspect "$img" >/dev/null 2>&1; then
    docker rmi "$img"
    echo "  removed $img"
  fi
done

docker image prune -f >/dev/null || true

echo
echo "Reset complete."
echo "  Kept: traefik:3.6, postgres:18, redis:8-alpine, adminer images"
echo "  Kept: ./letsencrypt/acme.json + ../.foxg-ssl-backups/$(basename "$(pwd)")/"
echo "  Wiped: db-data, redis-data, app :prod images"
echo
echo "Free build cache: bash __ctrl__/remote/prune-docker-build.sh"
echo "Start fresh:      bash __ctrl__/remote/start-prod.sh"
