#!/usr/bin/env bash
# Copy acme.json from the legacy Docker named volume into ./letsencrypt/
set -euo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"

LEGACY_VOL="${1:-xerxes_traefik-public-certificates}"
LE_DIR="letsencrypt"
ACME="$LE_DIR/acme.json"

mkdir -p "$LE_DIR"

if ! docker volume inspect "$LEGACY_VOL" >/dev/null 2>&1; then
  echo "Volume not found: $LEGACY_VOL"
  echo "List volumes: docker volume ls | grep -i traefik"
  exit 1
fi

if [[ -f "$ACME" ]] && [[ -s "$ACME" ]]; then
  backup="$ACME.bak.$(date +%Y%m%d-%H%M%S)"
  cp -a "$ACME" "$backup"
  echo "Existing $ACME backed up to $backup"
fi

echo "Copying acme.json from volume $LEGACY_VOL ..."
docker run --rm \
  -v "${LEGACY_VOL}:/from:ro" \
  -v "$(pwd)/${LE_DIR}:/to" \
  alpine:3.20 \
  sh -c 'test -f /from/acme.json || { echo "No acme.json in volume"; exit 1; }; cp -a /from/acme.json /to/acme.json'

chmod 600 "$ACME"
echo "Done. Certificates are in $ACME"
echo "Restart proxy: docker compose up -d proxy"
