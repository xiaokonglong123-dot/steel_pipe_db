-- 034_create_projects.sql
-- Project management: project charter, WBS elements, budget transactions.
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), DATE -> TEXT,
-- CURRENT_DATE -> date('now'), NUMERIC -> NUMERIC.

CREATE TABLE IF NOT EXISTS projects (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    project_no  TEXT NOT NULL,
    name        TEXT NOT NULL,
    description TEXT,
    status      TEXT NOT NULL DEFAULT 'planning',  -- planning | active | on_hold | completed | cancelled
    start_date  TEXT,                           -- DATE -> TEXT
    end_date    TEXT,
    manager_id  INTEGER,
    budget      REAL NOT NULL DEFAULT 0.0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT,
    UNIQUE (tenant_id, project_no)
);

CREATE TABLE IF NOT EXISTS wbs_elements (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    project_id  INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    parent_id   INTEGER,
    code        TEXT NOT NULL,
    name        TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    weight_pct  REAL,                        -- progress weight within parent
    progress_pct REAL NOT NULL DEFAULT 0.0,
    start_date  TEXT,                           -- DATE -> TEXT
    end_date    TEXT,
    assignee_id INTEGER,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_wbs_project ON wbs_elements(project_id);

CREATE TABLE IF NOT EXISTS project_transactions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    project_id  INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tx_type     TEXT NOT NULL,                  -- budget | expense | revenue
    amount      REAL NOT NULL,
    description TEXT,
    tx_date     TEXT NOT NULL DEFAULT (date('now')),
    created_by  INTEGER,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_pt_project ON project_transactions(project_id);
