-- Storage locations (zone / shelf / level hierarchy)
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
