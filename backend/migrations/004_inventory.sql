-- 004_inventory.sql — 库存

CREATE TABLE locations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    description TEXT,
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE inventory (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id     INTEGER NOT NULL REFERENCES items(id),
    location_id INTEGER NOT NULL REFERENCES locations(id),
    quantity    REAL NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(item_id, location_id)
);

CREATE TABLE inventory_logs (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id      INTEGER NOT NULL REFERENCES items(id),
    location_id  INTEGER REFERENCES locations(id),
    change_type  TEXT NOT NULL CHECK (change_type IN ('inbound','outbound','check_adjust')),
    quantity     REAL NOT NULL,
    ref_type     TEXT,
    ref_id       INTEGER,
    notes        TEXT,
    created_by   INTEGER REFERENCES users(id),
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_invlogs_item ON inventory_logs(item_id);
CREATE INDEX idx_invlogs_created ON inventory_logs(created_at);

CREATE TABLE inbound_records (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    record_no     TEXT NOT NULL UNIQUE,
    inbound_type  TEXT NOT NULL CHECK (inbound_type IN ('purchase','production','return','other')),
    order_id      INTEGER REFERENCES purchase_orders(id),
    supplier_id   INTEGER REFERENCES suppliers(id),
    status        TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','posted','cancelled')),
    notes         TEXT,
    created_by    INTEGER REFERENCES users(id),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at    TEXT
);

CREATE TABLE inbound_items (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id  INTEGER NOT NULL REFERENCES inbound_records(id),
    item_id    INTEGER NOT NULL REFERENCES items(id),
    location_id INTEGER REFERENCES locations(id),
    quantity   REAL NOT NULL,
    notes      TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE outbound_records (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    record_no     TEXT NOT NULL UNIQUE,
    outbound_type TEXT NOT NULL CHECK (outbound_type IN ('sales','requisition','other')),
    order_id      INTEGER REFERENCES sales_orders(id),
    customer_id   INTEGER REFERENCES customers(id),
    status        TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','posted','cancelled')),
    notes         TEXT,
    created_by    INTEGER REFERENCES users(id),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at    TEXT
);

CREATE TABLE outbound_items (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id  INTEGER NOT NULL REFERENCES outbound_records(id),
    item_id    INTEGER NOT NULL REFERENCES items(id),
    location_id INTEGER REFERENCES locations(id),
    quantity   REAL NOT NULL,
    notes      TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE check_records (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    record_no    TEXT NOT NULL UNIQUE,
    location_id  INTEGER REFERENCES locations(id),
    status       TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','counted','posted','cancelled')),
    notes        TEXT,
    created_by   INTEGER REFERENCES users(id),
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at   TEXT
);

CREATE TABLE check_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id   INTEGER NOT NULL REFERENCES check_records(id),
    item_id     INTEGER NOT NULL REFERENCES items(id),
    system_qty  REAL,
    actual_qty  REAL,
    diff        REAL,
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE reservations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id     INTEGER NOT NULL REFERENCES items(id),
    quantity    REAL NOT NULL,
    order_type  TEXT NOT NULL CHECK (order_type IN ('sales')),
    order_id    INTEGER NOT NULL,
    status      TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','released','cancelled')),
    created_by  INTEGER REFERENCES users(id),
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    released_at TEXT
);
CREATE INDEX idx_reservations_item ON reservations(item_id, status);
