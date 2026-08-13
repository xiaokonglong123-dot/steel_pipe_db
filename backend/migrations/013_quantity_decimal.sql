-- 013_quantity_decimal.sql — 数量列 REAL → TEXT（Decimal canonical 字符串）
--
-- 背景(ADR-002 延伸)：金额已用 TEXT + rust_decimal。库存/订单数量同样应在内存用
-- rust_decimal 保证精确算术与比较，杜绝 f64 在 `quantity = quantity + ?`、`SUM(quantity)`、
-- `diff = ? - system_qty` 等路径的浮点漂移。
--
-- 迁移纪律：不修改已执行迁移(001-012)。此处只对"数量"列做表重建以变更存储亲和性
-- 为 TEXT，并转换存量 REAL 值。被重建表均不被其它表以 FK 引用（已验证），故 DROP 旧表安全。
--
-- SQLite 无 ALTER COLUMN TYPE；采用标准 12 步策略：RENAME → CREATE(TEXT) → INSERT(CAST) → DROP。

-- ---------- inventory ----------
ALTER TABLE inventory RENAME TO inventory_013_old;
CREATE TABLE inventory (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id     INTEGER NOT NULL REFERENCES items(id),
    location_id INTEGER NOT NULL REFERENCES locations(id),
    quantity    TEXT NOT NULL DEFAULT '0',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(item_id, location_id)
);
INSERT INTO inventory (id, item_id, location_id, quantity, created_at, updated_at)
    SELECT id, item_id, location_id, CAST(quantity AS TEXT), created_at, updated_at
    FROM inventory_013_old;
DROP TABLE inventory_013_old;

-- ---------- inventory_logs ----------
ALTER TABLE inventory_logs RENAME TO inventory_logs_013_old;
DROP INDEX IF EXISTS idx_invlogs_item;
DROP INDEX IF EXISTS idx_invlogs_created;
CREATE TABLE inventory_logs (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id      INTEGER NOT NULL REFERENCES items(id),
    location_id  INTEGER REFERENCES locations(id),
    change_type  TEXT NOT NULL CHECK (change_type IN ('inbound','outbound','check_adjust')),
    quantity     TEXT NOT NULL,
    ref_type     TEXT,
    ref_id       INTEGER,
    notes        TEXT,
    created_by   INTEGER REFERENCES users(id),
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_invlogs_item ON inventory_logs(item_id);
CREATE INDEX idx_invlogs_created ON inventory_logs(created_at);
INSERT INTO inventory_logs (id, item_id, location_id, change_type, quantity, ref_type, ref_id, notes, created_by, created_at)
    SELECT id, item_id, location_id, change_type, CAST(quantity AS TEXT), ref_type, ref_id, notes, created_by, created_at
    FROM inventory_logs_013_old;
DROP TABLE inventory_logs_013_old;

-- ---------- inbound_items ----------
ALTER TABLE inbound_items RENAME TO inbound_items_013_old;
CREATE TABLE inbound_items (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id  INTEGER NOT NULL REFERENCES inbound_records(id),
    item_id    INTEGER NOT NULL REFERENCES items(id),
    location_id INTEGER REFERENCES locations(id),
    quantity   TEXT NOT NULL,
    notes      TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO inbound_items (id, record_id, item_id, location_id, quantity, notes, created_at)
    SELECT id, record_id, item_id, location_id, CAST(quantity AS TEXT), notes, created_at
    FROM inbound_items_013_old;
DROP TABLE inbound_items_013_old;

-- ---------- outbound_items ----------
ALTER TABLE outbound_items RENAME TO outbound_items_013_old;
CREATE TABLE outbound_items (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id  INTEGER NOT NULL REFERENCES outbound_records(id),
    item_id    INTEGER NOT NULL REFERENCES items(id),
    location_id INTEGER REFERENCES locations(id),
    quantity   TEXT NOT NULL,
    notes      TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO outbound_items (id, record_id, item_id, location_id, quantity, notes, created_at)
    SELECT id, record_id, item_id, location_id, CAST(quantity AS TEXT), notes, created_at
    FROM outbound_items_013_old;
DROP TABLE outbound_items_013_old;

-- ---------- check_items ----------
ALTER TABLE check_items RENAME TO check_items_013_old;
CREATE TABLE check_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id   INTEGER NOT NULL REFERENCES check_records(id),
    item_id     INTEGER NOT NULL REFERENCES items(id),
    system_qty  TEXT,
    actual_qty  TEXT,
    diff        TEXT,
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO check_items (id, record_id, item_id, system_qty, actual_qty, diff, notes, created_at)
    SELECT id, record_id, item_id,
           CASE WHEN system_qty IS NULL THEN NULL ELSE CAST(system_qty AS TEXT) END,
           CASE WHEN actual_qty IS NULL THEN NULL ELSE CAST(actual_qty AS TEXT) END,
           CASE WHEN diff IS NULL THEN NULL ELSE CAST(diff AS TEXT) END,
           notes, created_at
    FROM check_items_013_old;
DROP TABLE check_items_013_old;

-- ---------- reservations ----------
ALTER TABLE reservations RENAME TO reservations_013_old;
DROP INDEX IF EXISTS idx_reservations_item;
CREATE TABLE reservations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id     INTEGER NOT NULL REFERENCES items(id),
    quantity    TEXT NOT NULL,
    order_type  TEXT NOT NULL CHECK (order_type IN ('sales')),
    order_id    INTEGER NOT NULL,
    status      TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','released','cancelled')),
    created_by  INTEGER REFERENCES users(id),
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    released_at TEXT
);
CREATE INDEX idx_reservations_item ON reservations(item_id, status);
INSERT INTO reservations (id, item_id, quantity, order_type, order_id, status, created_by, created_at, released_at)
    SELECT id, item_id, CAST(quantity AS TEXT), order_type, order_id, status, created_by, created_at, released_at
    FROM reservations_013_old;
DROP TABLE reservations_013_old;

-- ---------- purchase_order_items ----------
ALTER TABLE purchase_order_items RENAME TO purchase_order_items_013_old;
CREATE TABLE purchase_order_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id    INTEGER NOT NULL REFERENCES purchase_orders(id),
    item_id     INTEGER NOT NULL REFERENCES items(id),
    quantity    TEXT NOT NULL,
    received_qty TEXT NOT NULL DEFAULT '0',
    unit_price  TEXT,
    total_price TEXT,
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO purchase_order_items (id, order_id, item_id, quantity, received_qty, unit_price, total_price, notes, created_at)
    SELECT id, order_id, item_id, CAST(quantity AS TEXT), CAST(received_qty AS TEXT), unit_price, total_price, notes, created_at
    FROM purchase_order_items_013_old;
DROP TABLE purchase_order_items_013_old;

-- ---------- sales_order_items ----------
ALTER TABLE sales_order_items RENAME TO sales_order_items_013_old;
CREATE TABLE sales_order_items (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id     INTEGER NOT NULL REFERENCES sales_orders(id),
    item_id      INTEGER NOT NULL REFERENCES items(id),
    quantity     TEXT NOT NULL,
    shipped_qty  TEXT NOT NULL DEFAULT '0',
    unit_price   TEXT,
    total_price  TEXT,
    notes        TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO sales_order_items (id, order_id, item_id, quantity, shipped_qty, unit_price, total_price, notes, created_at)
    SELECT id, order_id, item_id, CAST(quantity AS TEXT), CAST(shipped_qty AS TEXT), unit_price, total_price, notes, created_at
    FROM sales_order_items_013_old;
DROP TABLE sales_order_items_013_old;
