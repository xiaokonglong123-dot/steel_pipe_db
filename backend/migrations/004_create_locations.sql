-- 004_create_locations.sql
-- Warehouse storage locations organized in a zone → shelf → level hierarchy.
-- Each location can hold pipes and tracks capacity usage.
-- Soft delete via deleted_at column.
CREATE TABLE IF NOT EXISTS locations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    zone_code TEXT NOT NULL,
    shelf_code TEXT NOT NULL,
    level_code TEXT NOT NULL,
    full_code TEXT NOT NULL UNIQUE,
    description TEXT,
    capacity INTEGER,
    used_count INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX idx_locations_full_code ON locations(full_code);
CREATE INDEX idx_locations_zone_code ON locations(zone_code);
