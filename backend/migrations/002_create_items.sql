-- 002_create_items.sql
-- Generic item (商品) master data — replaces the former seamless_pipes table
-- of the steel-pipe era. Per specs/UBIQUITOUS_LANGUAGE_LATEST.md, Item + SKU is
-- the single product entity for the whole ERP. Industry-specific pipe columns
-- (grade / od / wt / API 5CT / threading / coupling) are removed in favor of
-- a free-form `spec` text column.
-- Soft delete via deleted_at column.
CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sku TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    category TEXT,
    unit TEXT,
    spec TEXT,
    price REAL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_items_sku ON items(sku);
CREATE INDEX IF NOT EXISTS idx_items_category ON items(category);
CREATE INDEX IF NOT EXISTS idx_items_status ON items(status);
CREATE INDEX IF NOT EXISTS idx_items_deleted_at ON items(deleted_at);
