-- 005_purchasing.sql — 采购

CREATE TABLE purchase_orders (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    order_no      TEXT NOT NULL UNIQUE,
    supplier_id   INTEGER NOT NULL REFERENCES suppliers(id),
    order_date    TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','submitted','approved','rejected','ordered','partially_received','received','cancelled')),
    doc_status    INTEGER NOT NULL DEFAULT 0,
    total_amount  TEXT NOT NULL DEFAULT '0',
    currency      TEXT NOT NULL DEFAULT 'CNY',
    notes         TEXT,
    created_by    INTEGER REFERENCES users(id),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at    TEXT
);
CREATE INDEX idx_po_supplier ON purchase_orders(supplier_id);
CREATE INDEX idx_po_status ON purchase_orders(status);

CREATE TABLE purchase_order_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id    INTEGER NOT NULL REFERENCES purchase_orders(id),
    item_id     INTEGER NOT NULL REFERENCES items(id),
    quantity    REAL NOT NULL,
    received_qty REAL NOT NULL DEFAULT 0,
    unit_price  TEXT,
    total_price TEXT,
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
