-- 010_warehouses.sql — 仓库/库位层级
-- 扩展 004_inventory.sql 中已有的 locations 表，增加 warehouse_id (FK) 与 deleted_at；
-- 新增 warehouses 表作为库位的父级。

CREATE TABLE warehouses (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    address     TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT
);

-- locations 已在 004_inventory.sql 创建；此处补充层级字段与软删除字段。
ALTER TABLE locations ADD COLUMN warehouse_id INTEGER REFERENCES warehouses(id);
ALTER TABLE locations ADD COLUMN deleted_at TEXT;
