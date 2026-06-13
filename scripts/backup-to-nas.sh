#!/usr/bin/env bash
#
# Back up the budget-analyser SQLite database to the aduki NAS.
#
# Takes a consistent online snapshot with `sqlite3 .backup` (safe even while
# the app is running), then streams it to the NAS over a single SSH
# connection — so you're prompted for the NAS password exactly once.
#
# Usage:
#   ./scripts/backup-to-nas.sh
#
# Override any of these via environment variables if needed:
#   NAS_USER   SSH user on the NAS            (default: adukiman)
#   NAS_HOST   NAS hostname                   (default: aduki-nas.local)
#   NAS_DIR    remote backup directory        (default: ~/backups/budget-analyser)
#   DB_PATH    local database to back up      (default: data/budget.db)
#
set -euo pipefail

NAS_USER="${NAS_USER:-adukiman}"
NAS_HOST="${NAS_HOST:-aduki-nas.local}"
NAS_DIR="${NAS_DIR:-backups/budget-analyser}"   # relative to the NAS user's home
DB_PATH="${DB_PATH:-data/budget.db}"

# Resolve DB_PATH relative to the repo root (parent of this script's dir).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
[[ "$DB_PATH" = /* ]] || DB_PATH="$REPO_ROOT/$DB_PATH"

if [[ ! -f "$DB_PATH" ]]; then
  echo "error: database not found at $DB_PATH" >&2
  exit 1
fi

STAMP="$(date +%Y%m%d-%H%M%S)"
REMOTE_FILE="budget-${STAMP}.db"
SNAPSHOT="$(mktemp -t budget-backup-XXXXXX).db"
trap 'rm -f "$SNAPSHOT"' EXIT

echo "Taking consistent snapshot of $DB_PATH ..."
sqlite3 "$DB_PATH" ".backup '$SNAPSHOT'"
sqlite3 "$SNAPSHOT" 'PRAGMA integrity_check;' >/dev/null
SIZE="$(du -h "$SNAPSHOT" | cut -f1)"
echo "Snapshot OK (${SIZE})."

echo "Copying to ${NAS_USER}@${NAS_HOST}:${NAS_DIR}/${REMOTE_FILE}"
echo "(you'll be prompted for the NAS password)"

# Single SSH connection: create the dir and write the file via stdin.
ssh "${NAS_USER}@${NAS_HOST}" \
  "mkdir -p '${NAS_DIR}' && cat > '${NAS_DIR}/${REMOTE_FILE}'" < "$SNAPSHOT"

echo "Done. Backup stored on NAS as ${NAS_DIR}/${REMOTE_FILE}"
