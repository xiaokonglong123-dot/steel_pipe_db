-- 002_catalog.sql — 商品主数据

CREATE TABLE items (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    sku        TEXT NOT NULL UNIQUE,
    name       TEXT NOT NULL,
    category   TEXT,
    unit       TEXT,
    spec       TEXT,
    status     TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','active','disabled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);
CREATE INDEX idx_items_sku ON items(sku);
CREATE INDEX idx_items_category ON items(category);
CREATE INDEX idx_items_status ON items(status);
