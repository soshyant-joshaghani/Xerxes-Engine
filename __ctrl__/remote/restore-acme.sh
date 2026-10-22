#!/usr/bin/env bash
# Restore acme.json from ../.foxg-ssl-backups/<project>/ into ./letsencrypt/
set -euo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"
# shellcheck source=lib/acme-backup.sh
source "$(dirname "$0")/lib/acme-backup.sh"
acme_restore_from_parent
echo "If proxy is running: docker compose up -d proxy"
