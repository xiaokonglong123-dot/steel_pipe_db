#!/bin/bash
# Steel Pipe DB - SQLite Database Restore Script
# Lists available backups and restores from a specified backup

set -euo pipefail

# Configuration
DB_PATH="${DB_PATH:-./data/steel_pipe.db}"
BACKUP_DIR="${BACKUP_DIR:-./backups}"
LOG_FILE="${LOG_FILE:-./backups/restore.log}"

# Log function
log() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] $1"
    echo "$msg" | tee -a "$LOG_FILE"
}

# List available backups
list_backups() {
    echo "Available backups:"
    echo "================="
    local count=0
    for f in $(find "$BACKUP_DIR" -name "steel_pipe_*.db.gz" -type f | sort -r); do
        count=$((count + 1))
        local size=$(du -h "$f" | cut -f1)
        local date=$(stat -c %y "$f" 2>/dev/null || stat -f %Sm "$f" 2>/dev/null)
        echo "  [$count] $(basename "$f") ($size) - $date"
    done
    if [ $count -eq 0 ]; then
        echo "  No backups found in $BACKUP_DIR"
        return 1
    fi
    echo ""
    return 0
}

# Restore from backup file
restore_backup() {
    local backup_file="$1"

    if [ ! -f "$backup_file" ]; then
        log "ERROR: Backup file not found: $backup_file"
        exit 1
    fi

    log "Restoring from: $backup_file"

    # Create a temporary directory for decompression
    local tmp_dir=$(mktemp -d)
    local tmp_db="${tmp_dir}/steel_pipe.db"

    # Decompress backup
    if ! gunzip -c "$backup_file" > "$tmp_db"; then
        log "ERROR: Failed to decompress backup"
        rm -rf "$tmp_dir"
        exit 1
    fi

    # Verify database integrity
    log "Verifying database integrity..."
    if ! sqlite3 "$tmp_db" "PRAGMA integrity_check;" | grep -q "ok"; then
        log "ERROR: Database integrity check failed"
        rm -rf "$tmp_dir"
        exit 1
    fi
    log "Database integrity verified"

    # Create backup of current database before restore
    if [ -f "$DB_PATH" ]; then
        local pre_restore_backup="${DB_PATH}.pre_restore_$(date +%Y%m%d_%H%M%S)"
        cp "$DB_PATH" "$pre_restore_backup"
        log "Created pre-restore backup: $pre_restore_backup"
    fi

    # Ensure target directory exists
    mkdir -p "$(dirname "$DB_PATH")"

    # Restore the database
    cp "$tmp_db" "$DB_PATH"
    log "Database restored to: $DB_PATH"

    # Clean up
    rm -rf "$tmp_dir"

    # Verify restored database
    log "Verifying restored database..."
    if sqlite3 "$DB_PATH" "PRAGMA integrity_check;" | grep -q "ok"; then
        log "Restore completed successfully"
    else
        log "WARNING: Restored database integrity check failed"
    fi
}

# Main script
case "${1:-}" in
    ""|"-l"|"--list")
        list_backups
        ;;
    "-r"|"--restore")
        if [ -z "${2:-}" ]; then
            echo "Usage: $0 --restore <backup_number|backup_file>"
            echo ""
            list_backups
            exit 1
        fi

        # Check if argument is a number (backup index)
        if [[ "$2" =~ ^[0-9]+$ ]]; then
            # Get backup file by index
            backup_file=$(find "$BACKUP_DIR" -name "steel_pipe_*.db.gz" -type f | sort -r | sed -n "${2}p")
            if [ -z "$backup_file" ]; then
                log "ERROR: Invalid backup number: $2"
                exit 1
            fi
            restore_backup "$backup_file"
        else
            # Treat as file path
            restore_backup "$2"
        fi
        ;;
    "-h"|"--help")
        echo "Usage: $0 [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  -l, --list          List available backups"
        echo "  -r, --restore <N>   Restore from backup N (number or file path)"
        echo "  -h, --help          Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0                  # List available backups"
        echo "  $0 --restore 1      # Restore most recent backup"
        echo "  $0 --restore ./backups/steel_pipe_20240101_120000.db.gz"
        ;;
    *)
        log "ERROR: Unknown option: $1"
        echo "Usage: $0 [-l|--list] [-r|--restore <N>] [-h|--help]"
        exit 1
        ;;
esac
