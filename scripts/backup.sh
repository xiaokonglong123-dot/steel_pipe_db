#!/bin/bash
# Steel Pipe DB - SQLite Database Backup Script
# Creates timestamped, gzipped backups with retention policy

set -euo pipefail

# Configuration
DB_PATH="${DB_PATH:-./data/steel_pipe.db}"
BACKUP_DIR="${BACKUP_DIR:-./backups}"
RETENTION_DAYS="${RETENTION_DAYS:-7}"
LOG_FILE="${LOG_FILE:-./backups/backup.log}"

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Generate timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_FILE="${BACKUP_DIR}/steel_pipe_${TIMESTAMP}.db.gz"

# Log function
log() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] $1"
    echo "$msg" | tee -a "$LOG_FILE"
}

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    log "ERROR: Database file not found at $DB_PATH"
    exit 1
fi

# Create backup using SQLite .backup command (safe for live databases)
log "Starting backup of $DB_PATH"
if sqlite3 "$DB_PATH" ".backup '${BACKUP_FILE%.gz}'" 2>/dev/null; then
    # Compress the backup
    gzip "${BACKUP_FILE%.gz}"
    BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
    log "Backup created: $BACKUP_FILE ($BACKUP_SIZE)"
else
    # Fallback to direct copy if sqlite3 .backup fails
    cp "$DB_PATH" "${BACKUP_FILE%.gz}"
    gzip "${BACKUP_FILE%.gz}"
    BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
    log "Backup created (copy): $BACKUP_FILE ($BACKUP_SIZE)"
fi

# Remove backups older than retention period
log "Cleaning up backups older than $RETENTION_DAYS days"
find "$BACKUP_DIR" -name "steel_pipe_*.db.gz" -mtime +$RETENTION_DAYS -delete -print | while read -r f; do
    log "Deleted old backup: $f"
done

# Show current backups
BACKUP_COUNT=$(find "$BACKUP_DIR" -name "steel_pipe_*.db.gz" | wc -l)
log "Total backups: $BACKUP_COUNT"

log "Backup completed successfully"
