-- Inbound records (header)
CREATE TABLE IF NOT EXISTS inbound_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inbound_no TEXT NOT NULL UNIQUE,
    inbound_type TEXT NOT NULL CHECK (inbound_type IN ('purchase', 'production', 'return')),
    order_id INTEGER,
    supplier_id INTEGER,
    notes TEXT,
    approval_status TEXT NOT NULL DEFAULT 'auto_approved' CHECK (approval_status IN ('auto_approved', 'pending', 'approved', 'rejected')),
    handled_by INTEGER,
    handled_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX idx_inbound_records_inbound_no ON inbound_records(inbound_no);
CREATE INDEX idx_inbound_records_inbound_type ON inbound_records(inbound_type);
CREATE INDEX idx_inbound_records_order_id ON inbound_records(order_id);

-- Inbound items (each pipe)
CREATE TABLE IF NOT EXISTS inbound_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inbound_id INTEGER NOT NULL,
    pipe_type TEXT NOT NULL,
    pipe_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_inbound_items_inbound_id ON inbound_items(inbound_id);
CREATE INDEX idx_inbound_items_pipe ON inbound_items(pipe_type, pipe_id);

-- Outbound records (header)
CREATE TABLE IF NOT EXISTS outbound_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    outbound_no TEXT NOT NULL UNIQUE,
    outbound_type TEXT NOT NULL CHECK (outbound_type IN ('sales', 'transfer', 'scrapped')),
    order_id INTEGER,
    customer_id INTEGER,
    notes TEXT,
    approval_status TEXT NOT NULL DEFAULT 'auto_approved' CHECK (approval_status IN ('auto_approved', 'pending', 'approved', 'rejected')),
    handled_by INTEGER,
    handled_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX idx_outbound_records_outbound_no ON outbound_records(outbound_no);
CREATE INDEX idx_outbound_records_outbound_type ON outbound_records(outbound_type);
CREATE INDEX idx_outbound_records_order_id ON outbound_records(order_id);

-- Outbound items (each pipe)
CREATE TABLE IF NOT EXISTS outbound_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    outbound_id INTEGER NOT NULL,
    pipe_type TEXT NOT NULL,
    pipe_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_outbound_items_outbound_id ON outbound_items(outbound_id);
CREATE INDEX idx_outbound_items_pipe ON outbound_items(pipe_type, pipe_id);

-- Inventory change logs (per-pipe granularity)
CREATE TABLE IF NOT EXISTS inventory_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pipe_type TEXT NOT NULL,
    pipe_id INTEGER NOT NULL,
    change_type TEXT NOT NULL CHECK (change_type IN ('inbound', 'outbound', 'transfer', 'check_adjust')),
    ref_type TEXT,
    ref_id INTEGER,
    from_location_id INTEGER,
    to_location_id INTEGER,
    notes TEXT,
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_inventory_logs_pipe ON inventory_logs(pipe_type, pipe_id);
CREATE INDEX idx_inventory_logs_created_at ON inventory_logs(created_at);
CREATE INDEX idx_inventory_logs_change_type ON inventory_logs(change_type);

-- Inventory check records
CREATE TABLE IF NOT EXISTS inventory_check_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    check_no TEXT NOT NULL UNIQUE,
    location_id INTEGER,
    status TEXT NOT NULL DEFAULT 'in_progress' CHECK (status IN ('in_progress', 'completed', 'cancelled')),
    notes TEXT,
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

-- Inventory check items
CREATE TABLE IF NOT EXISTS inventory_check_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    check_id INTEGER NOT NULL,
    pipe_type TEXT NOT NULL,
    pipe_id INTEGER NOT NULL,
    expected_status TEXT NOT NULL,
    found_status TEXT,
    is_match INTEGER,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_inventory_check_items_check_id ON inventory_check_items(check_id);
