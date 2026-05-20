CREATE TABLE IF NOT EXISTS inventory_checks (
    id TEXT PRIMARY KEY,
    check_no TEXT NOT NULL UNIQUE,
    check_type TEXT NOT NULL,
    operator_id TEXT NOT NULL,
    total_expected INTEGER NOT NULL DEFAULT 0,
    total_confirmed INTEGER NOT NULL DEFAULT 0,
    total_missing INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    status TEXT NOT NULL DEFAULT 'in_progress',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS inventory_check_items (
    id TEXT PRIMARY KEY,
    check_id TEXT NOT NULL,
    pipe_type TEXT NOT NULL,
    pipe_id TEXT NOT NULL,
    expected INTEGER NOT NULL DEFAULT 1,
    confirmed INTEGER NOT NULL DEFAULT 0,
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_inventory_checks_created_at ON inventory_checks(created_at);
CREATE INDEX IF NOT EXISTS idx_inventory_checks_status ON inventory_checks(status);
CREATE INDEX IF NOT EXISTS idx_inventory_check_items_check ON inventory_check_items(check_id);
