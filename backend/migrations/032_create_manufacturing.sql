-- 032_create_manufacturing.sql
-- Manufacturing: BOMs, work orders (with steps), quality inspections, NCRs.

CREATE TABLE IF NOT EXISTS mfg_boms (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    name        VARCHAR(200) NOT NULL,
    product_type VARCHAR(50) NOT NULL,       -- seamless | screen | welded
    version     INT NOT NULL DEFAULT 1,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS mfg_bom_items (
    id          BIGSERIAL PRIMARY KEY,
    bom_id      BIGINT NOT NULL REFERENCES mfg_boms(id) ON DELETE CASCADE,
    material    VARCHAR(100) NOT NULL,       -- e.g. 'steel_billet' | 'coupling' | 'thread_compound'
    quantity    NUMERIC(14,2) NOT NULL DEFAULT 1,
    unit        VARCHAR(20) NOT NULL DEFAULT 'pcs',
    notes       TEXT
);

CREATE INDEX idx_bom_items_bom ON mfg_bom_items(bom_id);

CREATE TABLE IF NOT EXISTS mfg_work_orders (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    wo_no       VARCHAR(40) NOT NULL,
    bom_id      BIGINT REFERENCES mfg_boms(id),
    product_type VARCHAR(50) NOT NULL,
    quantity    NUMERIC(14,2) NOT NULL DEFAULT 1,
    status      VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending | in_progress | completed | cancelled
    current_step INT NOT NULL DEFAULT 0,
    assigned_to BIGINT,
    due_date    DATE,
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ,
    UNIQUE (tenant_id, wo_no)
);

CREATE INDEX idx_mwo_status ON mfg_work_orders(status);

CREATE TABLE IF NOT EXISTS mfg_work_order_steps (
    id          BIGSERIAL PRIMARY KEY,
    work_order_id BIGINT NOT NULL REFERENCES mfg_work_orders(id) ON DELETE CASCADE,
    step_index  INT NOT NULL,
    step_name   VARCHAR(100) NOT NULL,
    status      VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending | done | skipped
    started_at  TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    notes       TEXT
);

CREATE INDEX idx_mwos_wo ON mfg_work_order_steps(work_order_id);

CREATE TABLE IF NOT EXISTS mfg_inspections (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    work_order_id BIGINT REFERENCES mfg_work_orders(id),
    pipe_id     BIGINT,
    inspection_type VARCHAR(50) NOT NULL,    -- visual | dimensional | hydrostatic | thread
    result      VARCHAR(20) NOT NULL,        -- pass | fail
    inspector   BIGINT,
    notes       TEXT,
    inspected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_mi_wo ON mfg_inspections(work_order_id);

CREATE TABLE IF NOT EXISTS mfg_ncrs (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    ncr_no      VARCHAR(40) NOT NULL,
    work_order_id BIGINT REFERENCES mfg_work_orders(id),
    pipe_id     BIGINT,
    description TEXT NOT NULL,
    severity    VARCHAR(20) NOT NULL DEFAULT 'minor',  -- minor | major | critical
    disposition VARCHAR(20),                 -- rework | scrap | use_as_is
    status      VARCHAR(20) NOT NULL DEFAULT 'open',   -- open | resolved
    created_by  BIGINT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    UNIQUE (tenant_id, ncr_no)
);

CREATE INDEX idx_ncr_wo ON mfg_ncrs(work_order_id);
