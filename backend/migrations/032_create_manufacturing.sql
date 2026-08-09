-- 032_create_manufacturing.sql
-- Manufacturing: BOMs, work orders (with steps), quality inspections, NCRs.
-- KEPT in the generic-ERP rewrite (inspection/NCR quality tables stay).
-- Itemized: mfg_inspections.pipe_id and mfg_ncrs.pipe_id -> item_id.
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), DATE -> TEXT,
-- BOOLEAN -> INTEGER (1/0), NUMERIC -> NUMERIC.

CREATE TABLE IF NOT EXISTS mfg_boms (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    name        TEXT NOT NULL,
    product_type TEXT NOT NULL,                 -- finished | semi_finished | custom
    version     INTEGER NOT NULL DEFAULT 1.0,
    is_active   INTEGER NOT NULL DEFAULT 1.0,
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT
);

CREATE TABLE IF NOT EXISTS mfg_bom_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    bom_id      INTEGER NOT NULL REFERENCES mfg_boms(id) ON DELETE CASCADE,
    material    TEXT NOT NULL,                  -- e.g. 'raw_material' | 'component' | 'packaging'
    quantity    REAL NOT NULL DEFAULT 1.0,
    unit        TEXT NOT NULL DEFAULT 'pcs',
    notes       TEXT
);

CREATE INDEX IF NOT EXISTS idx_bom_items_bom ON mfg_bom_items(bom_id);

CREATE TABLE IF NOT EXISTS mfg_work_orders (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    wo_no       TEXT NOT NULL,
    bom_id      INTEGER REFERENCES mfg_boms(id),
    product_type TEXT NOT NULL,
    quantity    REAL NOT NULL DEFAULT 1.0,
    status      TEXT NOT NULL DEFAULT 'pending',  -- pending | in_progress | completed | cancelled
    current_step INTEGER NOT NULL DEFAULT 0,
    assigned_to INTEGER,
    due_date    TEXT,                           -- DATE -> TEXT
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT,
    UNIQUE (tenant_id, wo_no)
);

CREATE INDEX IF NOT EXISTS idx_mwo_status ON mfg_work_orders(status);

CREATE TABLE IF NOT EXISTS mfg_work_order_steps (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    work_order_id INTEGER NOT NULL REFERENCES mfg_work_orders(id) ON DELETE CASCADE,
    step_index  INTEGER NOT NULL,
    step_name   TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending',  -- pending | done | skipped
    started_at  TEXT,
    completed_at TEXT,
    notes       TEXT
);

CREATE INDEX IF NOT EXISTS idx_mwos_wo ON mfg_work_order_steps(work_order_id);

CREATE TABLE IF NOT EXISTS mfg_inspections (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    work_order_id INTEGER REFERENCES mfg_work_orders(id),
    item_id     INTEGER,
    inspection_type TEXT NOT NULL,              -- visual | dimensional | hydrostatic | functional
    result      TEXT NOT NULL,                  -- pass | fail
    inspector   INTEGER,
    notes       TEXT,
    inspected_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_mi_wo ON mfg_inspections(work_order_id);

CREATE TABLE IF NOT EXISTS mfg_ncrs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    ncr_no      TEXT NOT NULL,
    work_order_id INTEGER REFERENCES mfg_work_orders(id),
    item_id     INTEGER,
    description TEXT NOT NULL,
    severity    TEXT NOT NULL DEFAULT 'minor',  -- minor | major | critical
    disposition TEXT,                           -- rework | scrap | use_as_is
    status      TEXT NOT NULL DEFAULT 'open',   -- open | resolved
    created_by  INTEGER,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT,
    UNIQUE (tenant_id, ncr_no)
);

CREATE INDEX IF NOT EXISTS idx_ncr_wo ON mfg_ncrs(work_order_id);
