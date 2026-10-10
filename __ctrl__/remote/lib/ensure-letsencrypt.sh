#!/usr/bin/env bash
# Ensure ./letsencrypt/acme.json exists for Traefik ACME (bind mount).
# Migrates once from the legacy named volume if present.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LE_DIR="$ROOT/letsencrypt"
ACME="$LE_DIR/acme.json"
LEGACY_VOL="xerxes_traefik-public-certificates"

# shellcheck source=acme-backup.sh
source "$(dirname "${BASH_SOURCE[0]}")/acme-backup.sh"

mkdir -p "$LE_DIR"

if [[ ! -f "$ACME" ]]; then
  touch "$ACME"
fi
chmod 600 "$ACME"

if [[ ! -s "$ACME" ]]; then
  acme_restore_from_parent 2>/dev/null || true
fi

if [[ ! -s "$ACME" ]] && docker volume inspect "$LEGACY_VOL" >/dev/null 2>&1; then
  echo "Migrating acme.json from legacy volume $LEGACY_VOL ..."
  docker run --rm \
    -v "${LEGACY_VOL}:/from:ro" \
    -v "${LE_DIR}:/to" \
    alpine:3.20 \
    sh -c 'if [ -f /from/acme.json ]; then cp -a /from/acme.json /to/acme.json; fi'
  chmod 600 "$ACME"
fi
