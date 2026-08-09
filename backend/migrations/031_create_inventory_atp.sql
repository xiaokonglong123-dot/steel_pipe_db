-- 031_create_inventory_atp.sql
-- Inventory ATP deep-dive: reservation slots (available-to-promise),
-- internal transfers (location-to-location), cycle count templates.
-- Itemized for the generic ERP: atp_slots and internal_transfers reference
-- item_id (+ sku) instead of the old pipe_type/pipe_number pair.
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), JSONB -> TEXT,
-- NUMERIC -> REAL (sqlx binds f64; NUMERIC affinity would store integral
-- doubles as INTEGER which sqlx cannot decode into f64).

CREATE TABLE IF NOT EXISTS atp_slots (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1.0,
    item_id         INTEGER NOT NULL,
    sku             TEXT,
    quantity_reserved REAL NOT NULL DEFAULT 0.0,
    sales_order_id  INTEGER,
    status          TEXT NOT NULL DEFAULT 'reserved',  -- reserved | released
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    released_at     TEXT
);

CREATE INDEX IF NOT EXISTS idx_atp_item ON atp_slots(item_id);
CREATE INDEX IF NOT EXISTS idx_atp_so ON atp_slots(sales_order_id);

CREATE TABLE IF NOT EXISTS internal_transfers (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1.0,
    transfer_no     TEXT NOT NULL,
    from_location_id INTEGER NOT NULL,
    to_location_id  INTEGER NOT NULL,
    item_id         INTEGER,
    sku             TEXT,
    quantity        REAL NOT NULL DEFAULT 1.0,
    transferred_at  TEXT NOT NULL DEFAULT (datetime('now')),
    status          TEXT NOT NULL DEFAULT 'completed',  -- completed | cancelled
    created_by      INTEGER,
    notes           TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (tenant_id, transfer_no)
);

CREATE INDEX IF NOT EXISTS idx_it_item ON internal_transfers(item_id);

CREATE TABLE IF NOT EXISTS count_templates (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1.0,
    name            TEXT NOT NULL,
    description     TEXT,
    location_ids    TEXT NOT NULL DEFAULT '[]', -- JSONB -> TEXT
    is_active       INTEGER NOT NULL DEFAULT 1.0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS count_sessions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1.0,
    template_id     INTEGER NOT NULL REFERENCES count_templates(id),
    session_no      TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'inprogress',  -- inprogress | completed | cancelled
    started_at      TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at    TEXT,
    result_json     TEXT,                       -- JSONB -> TEXT
    UNIQUE (tenant_id, session_no)
);
