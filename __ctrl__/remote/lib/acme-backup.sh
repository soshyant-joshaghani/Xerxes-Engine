#!/usr/bin/env bash
# Parent-dir SSL backup — ../.foxg-ssl-backups/<project>/acme.json
# Survives docker builder prune and repo-local cleanup.

set -euo pipefail

_acme_paths() {
  ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
  LE_DIR="$ROOT/letsencrypt"
  ACME="$LE_DIR/acme.json"
  BACKUP_ROOT="$(cd "$ROOT/.." && pwd)/.foxg-ssl-backups"
  BACKUP_DIR="$BACKUP_ROOT/$(basename "$ROOT")"
  BACKUP_FILE="$BACKUP_DIR/acme.json"
}

acme_backup_to_parent() {
  _acme_paths
  mkdir -p "$BACKUP_DIR"
  if [[ ! -f "$ACME" ]]; then
    echo "No local acme.json at $ACME — nothing to back up."
    return 1
  fi
  if [[ ! -s "$ACME" ]]; then
    echo "Local acme.json is empty — skipping backup."
    return 1
  fi
  cp -a "$ACME" "$BACKUP_FILE"
  chmod 600 "$BACKUP_FILE"
  date -u +%Y-%m-%dT%H:%M:%SZ >"$BACKUP_DIR/last-backup.txt" 2>/dev/null || true
  echo "Backed up SSL to $BACKUP_FILE ($(wc -c <"$BACKUP_FILE") bytes)"
}

acme_restore_from_parent() {
  _acme_paths
  if [[ ! -f "$BACKUP_FILE" ]] || [[ ! -s "$BACKUP_FILE" ]]; then
    echo "No backup at $BACKUP_FILE"
    return 1
  fi
  mkdir -p "$LE_DIR"
  if [[ -f "$ACME" ]] && [[ -s "$ACME" ]]; then
    local backup="$ACME.bak.$(date +%Y%m%d-%H%M%S)"
    cp -a "$ACME" "$backup"
    echo "Existing acme.json copied to $backup"
  fi
  cp -a "$BACKUP_FILE" "$ACME"
  chmod 600 "$ACME"
  echo "Restored SSL from $BACKUP_FILE → $ACME ($(wc -c <"$ACME") bytes)"
}
