#!/usr/bin/env bash
# Free disk from Docker build cache without losing SSL certs.
# 1. Backs up acme.json outside the repo
# 2. docker builder prune -af  (does NOT delete ./letsencrypt/)
set -euo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"
# shellcheck source=lib/acme-backup.sh
source "$(dirname "$0")/lib/acme-backup.sh"

echo "==> Backing up SSL outside repo (before build cache prune)..."
acme_backup_to_parent || echo "    (no SSL file to back up — continuing)"

echo "==> Pruning Docker build cache..."
docker builder prune -af

echo
echo "Build cache cleared."
echo "SSL backup (if any): ../.foxg-ssl-backups/$(basename "$(pwd)")/acme.json"
echo "Restore later:       bash __ctrl__/remote/restore-acme.sh"
