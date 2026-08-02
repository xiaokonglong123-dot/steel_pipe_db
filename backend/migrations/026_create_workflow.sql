-- 026_create_workflow.sql
-- Unified approval workflow engine: definitions (templates), instances,
-- approval nodes, delegations, escalations. All tables in public schema
-- (consistent with the rest of the codebase; the spec's `workflow.` schema
-- prefix is dropped for test-isolation simplicity).

CREATE TABLE IF NOT EXISTS workflow_definitions (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    name            VARCHAR(200) NOT NULL,
    entity_type     VARCHAR(50) NOT NULL,          -- 'purchase_order' | 'sales_order' | 'leave_request' | ...
    description     TEXT,
    definition_json JSONB NOT NULL DEFAULT '{}',   -- ordered nodes + edges
    callback_action VARCHAR(100),                  -- e.g. 'approve_purchase_order' (consumed by business modules)
    version         INT NOT NULL DEFAULT 1,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX idx_wf_defs_tenant ON workflow_definitions(tenant_id, entity_type);

CREATE TABLE IF NOT EXISTS workflow_instances (
    id              BIGSERIAL PRIMARY KEY,
    definition_id   BIGINT NOT NULL REFERENCES workflow_definitions(id),
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    entity_type     VARCHAR(50) NOT NULL,
    entity_id       BIGINT NOT NULL,
    amount          NUMERIC(18,2),                 -- context for amount-based condition routing
    status          VARCHAR(20) NOT NULL DEFAULT 'running',  -- running | approved | rejected | cancelled
    current_step    INT NOT NULL DEFAULT 0,
    initiator_id    BIGINT NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at     TIMESTAMPTZ
);

CREATE INDEX idx_wf_inst_entity ON workflow_instances(entity_type, entity_id);
CREATE INDEX idx_wf_inst_status ON workflow_instances(status);

CREATE TABLE IF NOT EXISTS approval_nodes (
    id              BIGSERIAL PRIMARY KEY,
    instance_id     BIGINT NOT NULL REFERENCES workflow_instances(id) ON DELETE CASCADE,
    step_index      INT NOT NULL,
    node_key        VARCHAR(100) NOT NULL,
    assignee_type   VARCHAR(20) NOT NULL,          -- 'role' | 'user' | 'any'
    assignee_value  VARCHAR(200),                  -- role name / user id
    condition_json  JSONB,                         -- e.g. {"amount_gt": 50000}
    status          VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending | approved | rejected | skipped
    approver_id     BIGINT,
    approval_reason TEXT,
    due_date        TIMESTAMPTZ,
    decided_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_nodes_instance ON approval_nodes(instance_id, step_index);
CREATE INDEX idx_nodes_status ON approval_nodes(status);

CREATE TABLE IF NOT EXISTS workflow_delegations (
    id                  BIGSERIAL PRIMARY KEY,
    original_user_id    BIGINT NOT NULL REFERENCES users(id),
    delegated_user_id   BIGINT NOT NULL REFERENCES users(id),
    entity_type         VARCHAR(50),
    starts_at           TIMESTAMPTZ NOT NULL,
    ends_at             TIMESTAMPTZ,
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workflow_escalations (
    id              BIGSERIAL PRIMARY KEY,
    node_id         BIGINT NOT NULL REFERENCES approval_nodes(id) ON DELETE CASCADE,
    escalation_level INT NOT NULL DEFAULT 1,
    notified_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
