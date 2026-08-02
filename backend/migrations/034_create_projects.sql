-- 034_create_projects.sql
-- Project management: project charter, WBS elements, budget transactions.

CREATE TABLE IF NOT EXISTS projects (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    project_no  VARCHAR(40) NOT NULL,
    name        VARCHAR(200) NOT NULL,
    description TEXT,
    status      VARCHAR(20) NOT NULL DEFAULT 'planning',  -- planning | active | on_hold | completed | cancelled
    start_date  DATE,
    end_date    DATE,
    manager_id  BIGINT,
    budget      NUMERIC(18,2) NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ,
    UNIQUE (tenant_id, project_no)
);

CREATE TABLE IF NOT EXISTS wbs_elements (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    project_id  BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    parent_id   BIGINT,
    code        VARCHAR(30) NOT NULL,
    name        VARCHAR(200) NOT NULL,
    sort_order  INT NOT NULL DEFAULT 0,
    weight_pct  NUMERIC(5,2),                -- progress weight within parent
    progress_pct NUMERIC(5,2) NOT NULL DEFAULT 0,
    start_date  DATE,
    end_date    DATE,
    assignee_id BIGINT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_wbs_project ON wbs_elements(project_id);

CREATE TABLE IF NOT EXISTS project_transactions (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    project_id  BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tx_type     VARCHAR(20) NOT NULL,        -- budget | expense | revenue
    amount      NUMERIC(18,2) NOT NULL,
    description TEXT,
    tx_date     DATE NOT NULL DEFAULT CURRENT_DATE,
    created_by  BIGINT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pt_project ON project_transactions(project_id);
