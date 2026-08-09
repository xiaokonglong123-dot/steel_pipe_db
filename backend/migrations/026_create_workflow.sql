-- 026_create_workflow.sql
-- Unified approval workflow engine: definitions (templates), instances,
-- approval nodes, delegations, escalations. SQLite has no schemas (the old
-- `workflow.` schema prefix is dropped, consistent with the rest of the code).
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), JSONB -> TEXT,
-- BOOLEAN -> INTEGER (1/0), NUMERIC(18,2) -> NUMERIC.

CREATE TABLE IF NOT EXISTS workflow_definitions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1.0,
    name            TEXT NOT NULL,
    entity_type     TEXT NOT NULL,             -- 'purchase_order' | 'sales_order' | 'leave_request' | ...
    description     TEXT,
    definition_json TEXT NOT NULL DEFAULT '{}',-- ordered nodes + edges (JSONB -> TEXT)
    callback_action TEXT,                      -- e.g. 'approve_purchase_order' (consumed by business modules)
    version         INTEGER NOT NULL DEFAULT 1.0,
    is_active       INTEGER NOT NULL DEFAULT 1.0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at      TEXT
);

CREATE INDEX IF NOT EXISTS idx_wf_defs_tenant ON workflow_definitions(tenant_id, entity_type);

CREATE TABLE IF NOT EXISTS workflow_instances (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    definition_id   INTEGER NOT NULL REFERENCES workflow_definitions(id),
    tenant_id       INTEGER NOT NULL DEFAULT 1.0,
    entity_type     TEXT NOT NULL,
    entity_id       INTEGER NOT NULL,
    amount          REAL,                   -- context for amount-based condition routing
    status          TEXT NOT NULL DEFAULT 'running',  -- running | approved | rejected | cancelled
    current_step    INTEGER NOT NULL DEFAULT 0,
    initiator_id    INTEGER NOT NULL REFERENCES users(id),
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    finished_at     TEXT
);

CREATE INDEX IF NOT EXISTS idx_wf_inst_entity ON workflow_instances(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_wf_inst_status ON workflow_instances(status);

CREATE TABLE IF NOT EXISTS approval_nodes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    instance_id     INTEGER NOT NULL REFERENCES workflow_instances(id) ON DELETE CASCADE,
    step_index      INTEGER NOT NULL,
    node_key        TEXT NOT NULL,
    assignee_type   TEXT NOT NULL,             -- 'role' | 'user' | 'any'
    assignee_value  TEXT,                      -- role name / user id
    condition_json  TEXT,                      -- e.g. {"amount_gt": 50000} (JSONB -> TEXT)
    status          TEXT NOT NULL DEFAULT 'pending', -- pending | approved | rejected | skipped
    approver_id     INTEGER,
    approval_reason TEXT,
    due_date        TEXT,
    decided_at      TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_nodes_instance ON approval_nodes(instance_id, step_index);
CREATE INDEX IF NOT EXISTS idx_nodes_status ON approval_nodes(status);

CREATE TABLE IF NOT EXISTS workflow_delegations (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    original_user_id    INTEGER NOT NULL REFERENCES users(id),
    delegated_user_id   INTEGER NOT NULL REFERENCES users(id),
    entity_type         TEXT,
    starts_at           TEXT NOT NULL,
    ends_at             TEXT,
    is_active           INTEGER NOT NULL DEFAULT 1.0,
    created_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS workflow_escalations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id         INTEGER NOT NULL REFERENCES approval_nodes(id) ON DELETE CASCADE,
    escalation_level INTEGER NOT NULL DEFAULT 1.0,
    notified_at     TEXT NOT NULL DEFAULT (datetime('now'))
);
