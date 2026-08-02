-- 031_create_inventory_atp.sql
-- Inventory ATP deep-dive: reservation slots (available-to-promise),
-- internal transfers (location-to-location), cycle count templates.

CREATE TABLE IF NOT EXISTS atp_slots (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    pipe_type       VARCHAR(20) NOT NULL,
    pipe_number     VARCHAR(100),
    quantity_reserved NUMERIC(14,2) NOT NULL DEFAULT 0,
    sales_order_id  BIGINT,
    status          VARCHAR(20) NOT NULL DEFAULT 'reserved',  -- reserved | released
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    released_at     TIMESTAMPTZ
);

CREATE INDEX idx_atp_pipe ON atp_slots(pipe_type, pipe_number);
CREATE INDEX idx_atp_so ON atp_slots(sales_order_id);

CREATE TABLE IF NOT EXISTS internal_transfers (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    transfer_no     VARCHAR(40) NOT NULL,
    from_location_id BIGINT NOT NULL,
    to_location_id  BIGINT NOT NULL,
    pipe_id         BIGINT,
    pipe_number     VARCHAR(100),
    quantity        NUMERIC(14,2) NOT NULL DEFAULT 1,
    transferred_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status          VARCHAR(20) NOT NULL DEFAULT 'completed',  -- completed | cancelled
    created_by      BIGINT,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, transfer_no)
);

CREATE TABLE IF NOT EXISTS count_templates (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    name            VARCHAR(100) NOT NULL,
    description     TEXT,
    location_ids    JSONB NOT NULL DEFAULT '[]',
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS count_sessions (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    template_id     BIGINT NOT NULL REFERENCES count_templates(id),
    session_no      VARCHAR(40) NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'inprogress',  -- inprogress | completed | cancelled
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ,
    result_json     JSONB,
    UNIQUE (tenant_id, session_no)
);
