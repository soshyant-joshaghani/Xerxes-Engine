#!/usr/bin/env bash
# Copy letsencrypt/acme.json to ../.foxg-ssl-backups/<project>/ (outside the repo).
set -euo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"
# shellcheck source=lib/acme-backup.sh
source "$(dirname "$0")/lib/acme-backup.sh"
acme_backup_to_parent
