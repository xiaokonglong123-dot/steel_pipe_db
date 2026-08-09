-- 030_create_sales_crm.sql
-- Sales CRM deep-dive: shipments (delivery tracking) and sales quotes
-- (quotation → order conversion).
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), DATE -> TEXT,
-- CURRENT_DATE -> date('now'), JSONB -> TEXT, NUMERIC -> NUMERIC.

CREATE TABLE IF NOT EXISTS sales_shipments (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    shipment_no TEXT NOT NULL,
    sales_order_id INTEGER NOT NULL,
    shipped_at  TEXT,
    carrier     TEXT,
    tracking_no TEXT,
    status      TEXT NOT NULL DEFAULT 'pending',  -- pending | shipped | delivered
    items_json  TEXT NOT NULL DEFAULT '[]',    -- JSONB -> TEXT
    notes       TEXT,
    created_by  INTEGER,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (tenant_id, shipment_no)
);

CREATE INDEX IF NOT EXISTS idx_ss_order ON sales_shipments(sales_order_id);

CREATE TABLE IF NOT EXISTS sales_quotes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    quote_no    TEXT NOT NULL,
    customer_id INTEGER NOT NULL,
    quote_date  TEXT NOT NULL DEFAULT (date('now')),
    valid_until TEXT,                          -- DATE -> TEXT
    total_amount REAL NOT NULL DEFAULT 0.0,
    status      TEXT NOT NULL DEFAULT 'draft',  -- draft | confirmed | converted | expired
    items_json  TEXT NOT NULL DEFAULT '[]',    -- JSONB -> TEXT
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT,
    UNIQUE (tenant_id, quote_no)
);

CREATE INDEX IF NOT EXISTS idx_sq_customer ON sales_quotes(customer_id);
