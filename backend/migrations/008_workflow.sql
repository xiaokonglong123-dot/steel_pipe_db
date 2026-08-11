-- 008_workflow.sql — 审批流（数据驱动）

CREATE TABLE workflows (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    applies_to  TEXT NOT NULL CHECK (applies_to IN ('purchase_order','sales_order','inbound_record','outbound_record')),
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE workflow_states (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id INTEGER NOT NULL REFERENCES workflows(id),
    state_key   TEXT NOT NULL,
    doc_status  INTEGER NOT NULL DEFAULT 0,
    is_initial  INTEGER NOT NULL DEFAULT 0,
    is_final    INTEGER NOT NULL DEFAULT 0,
    UNIQUE(workflow_id, state_key)
);

CREATE TABLE workflow_transitions (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id    INTEGER NOT NULL REFERENCES workflows(id),
    from_state_id  INTEGER NOT NULL REFERENCES workflow_states(id),
    to_state_id    INTEGER NOT NULL REFERENCES workflow_states(id),
    action         TEXT NOT NULL,
    required_role  TEXT,
    is_auto        INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE workflow_instances (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id    INTEGER NOT NULL REFERENCES workflows(id),
    business_type  TEXT NOT NULL,
    business_id    INTEGER NOT NULL,
    current_state  TEXT NOT NULL,
    status         TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','completed','cancelled')),
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE workflow_tasks (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    instance_id  INTEGER NOT NULL REFERENCES workflow_instances(id),
    state_key    TEXT NOT NULL,
    assignee_id  INTEGER REFERENCES users(id),
    status       TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','completed','skipped')),
    action       TEXT,
    comment      TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);
