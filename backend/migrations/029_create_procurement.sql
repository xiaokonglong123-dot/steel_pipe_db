-- 029_create_procurement.sql
-- Procurement deep-dive: purchase requisitions, goods receipts, supplier
-- quotes, supplier scorecard support.
-- Itemized for the generic ERP: po_receipt_items references item_id (+ sku)
-- instead of the old pipe_id/pipe_number pair.
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), DATE -> TEXT,
-- JSONB -> TEXT, NUMERIC(14,2)/NUMERIC(18,2) -> NUMERIC.

CREATE TABLE IF NOT EXISTS purchase_requisitions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    req_no      TEXT NOT NULL,
    title       TEXT NOT NULL,
    department_id INTEGER,
    applicant_id INTEGER,
    expected_date TEXT,                        -- DATE -> TEXT
    status      TEXT NOT NULL DEFAULT 'draft',  -- draft | submitted | approved | rejected
    items_json  TEXT NOT NULL DEFAULT '[]',    -- JSONB -> TEXT
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT,
    UNIQUE (tenant_id, req_no)
);

CREATE TABLE IF NOT EXISTS po_receipts (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    receipt_no  TEXT NOT NULL,
    purchase_order_id INTEGER NOT NULL,
    received_at TEXT NOT NULL DEFAULT (datetime('now')),
    status      TEXT NOT NULL DEFAULT 'received',  -- received | inspected | accepted
    notes       TEXT,
    created_by  INTEGER,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (tenant_id, receipt_no)
);

CREATE TABLE IF NOT EXISTS po_receipt_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    receipt_id  INTEGER NOT NULL REFERENCES po_receipts(id) ON DELETE CASCADE,
    item_id     INTEGER,
    sku         TEXT,
    quantity    REAL NOT NULL DEFAULT 1.0,
    remark      TEXT
);

CREATE INDEX IF NOT EXISTS idx_pri_receipt ON po_receipt_items(receipt_id);
CREATE INDEX IF NOT EXISTS idx_pri_item ON po_receipt_items(item_id);

CREATE TABLE IF NOT EXISTS supplier_quotes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    quote_no    TEXT NOT NULL,
    supplier_id INTEGER NOT NULL,
    title       TEXT,
    valid_until TEXT,                          -- DATE -> TEXT
    total_amount REAL NOT NULL DEFAULT 0.0,
    status      TEXT NOT NULL DEFAULT 'draft',  -- draft | sent | accepted | expired
    items_json  TEXT NOT NULL DEFAULT '[]',    -- JSONB -> TEXT
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT,
    UNIQUE (tenant_id, quote_no)
);

CREATE INDEX IF NOT EXISTS idx_sq_supplier ON supplier_quotes(supplier_id);
