-- 035_create_assets.sql
-- Fixed assets: registration, straight-line depreciation, disposal.
-- KEPT in the generic-ERP rewrite (固定资产 assets module stays).
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), DATE -> TEXT,
-- NUMERIC -> NUMERIC.

CREATE TABLE IF NOT EXISTS fixed_assets (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1.0,
    asset_no        TEXT NOT NULL,
    name            TEXT NOT NULL,
    category        TEXT NOT NULL DEFAULT 'equipment',  -- equipment | vehicle | building | tooling
    purchase_date   TEXT NOT NULL,              -- DATE -> TEXT
    purchase_cost   REAL NOT NULL DEFAULT 0.0,
    salvage_value   REAL NOT NULL DEFAULT 0.0,
    useful_life_months INTEGER NOT NULL DEFAULT 60,
    current_value   REAL NOT NULL DEFAULT 0.0,
    status          TEXT NOT NULL DEFAULT 'active',  -- active | disposed
    location        TEXT,
    department_id   INTEGER,
    notes           TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at      TEXT,
    UNIQUE (tenant_id, asset_no)
);

CREATE TABLE IF NOT EXISTS depreciation_entries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id    INTEGER NOT NULL REFERENCES fixed_assets(id) ON DELETE CASCADE,
    period      TEXT NOT NULL,                  -- 'YYYY-MM'
    amount      REAL NOT NULL DEFAULT 0.0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (asset_id, period)
);

CREATE INDEX IF NOT EXISTS idx_dep_asset ON depreciation_entries(asset_id);
