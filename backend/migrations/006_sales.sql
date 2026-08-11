-- 006_sales.sql — 销售

CREATE TABLE sales_orders (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    order_no      TEXT NOT NULL UNIQUE,
    customer_id   INTEGER NOT NULL REFERENCES customers(id),
    order_date    TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','submitted','approved','rejected','awaiting_shipment','partially_shipped','shipped','cancelled')),
    doc_status    INTEGER NOT NULL DEFAULT 0,
    total_amount  TEXT NOT NULL DEFAULT '0',
    currency      TEXT NOT NULL DEFAULT 'CNY',
    notes         TEXT,
    created_by    INTEGER REFERENCES users(id),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at    TEXT
);
CREATE INDEX idx_so_customer ON sales_orders(customer_id);
CREATE INDEX idx_so_status ON sales_orders(status);

CREATE TABLE sales_order_items (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id     INTEGER NOT NULL REFERENCES sales_orders(id),
    item_id      INTEGER NOT NULL REFERENCES items(id),
    quantity     REAL NOT NULL,
    shipped_qty  REAL NOT NULL DEFAULT 0,
    unit_price   TEXT,
    total_price  TEXT,
    notes        TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
