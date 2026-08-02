-- 005_create_inventory.sql
-- Core inventory tables: inbound/outbound records + items, inventory current state, movement logs.
-- Inbound types: purchase, production, return, transfer.
-- Outbound types: sales, scrapped, transfer.
-- Approval workflow: pending → approved/rejected; auto_approved skips approval.
-- Inventory logs provide per-pipe audit trail for traceability.
-- No FK constraints — integrity enforced at application layer.
-- Soft delete via deleted_at column.
CREATE TABLE IF NOT EXISTS inbound_records (
    id BIGSERIAL PRIMARY KEY,
    inbound_no TEXT NOT NULL UNIQUE,
    inbound_type TEXT NOT NULL CHECK (inbound_type IN ('purchase', 'production', 'return')),
    order_id BIGINT,
    supplier_id BIGINT,
    notes TEXT,
    approval_status TEXT NOT NULL DEFAULT 'auto_approved' CHECK (approval_status IN ('auto_approved', 'pending', 'approved', 'rejected')),
    handled_by BIGINT,
    handled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_inbound_records_inbound_no ON inbound_records(inbound_no);
CREATE INDEX idx_inbound_records_inbound_type ON inbound_records(inbound_type);
CREATE INDEX idx_inbound_records_order_id ON inbound_records(order_id);

-- Inbound items (each pipe)
CREATE TABLE IF NOT EXISTS inbound_items (
    id BIGSERIAL PRIMARY KEY,
    inbound_id BIGINT NOT NULL,
    pipe_type TEXT NOT NULL,
    pipe_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_inbound_items_inbound_id ON inbound_items(inbound_id);
CREATE INDEX idx_inbound_items_pipe ON inbound_items(pipe_type, pipe_id);

-- Outbound records (header)
CREATE TABLE IF NOT EXISTS outbound_records (
    id BIGSERIAL PRIMARY KEY,
    outbound_no TEXT NOT NULL UNIQUE,
    outbound_type TEXT NOT NULL CHECK (outbound_type IN ('sales', 'transfer', 'scrapped')),
    order_id BIGINT,
    customer_id BIGINT,
    notes TEXT,
    approval_status TEXT NOT NULL DEFAULT 'auto_approved' CHECK (approval_status IN ('auto_approved', 'pending', 'approved', 'rejected')),
    handled_by BIGINT,
    handled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_outbound_records_outbound_no ON outbound_records(outbound_no);
CREATE INDEX idx_outbound_records_outbound_type ON outbound_records(outbound_type);
CREATE INDEX idx_outbound_records_order_id ON outbound_records(order_id);

-- Outbound items (each pipe)
CREATE TABLE IF NOT EXISTS outbound_items (
    id BIGSERIAL PRIMARY KEY,
    outbound_id BIGINT NOT NULL,
    pipe_type TEXT NOT NULL,
    pipe_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_outbound_items_outbound_id ON outbound_items(outbound_id);
CREATE INDEX idx_outbound_items_pipe ON outbound_items(pipe_type, pipe_id);

-- Inventory change logs (per-pipe granularity)
CREATE TABLE IF NOT EXISTS inventory_logs (
    id BIGSERIAL PRIMARY KEY,
    pipe_type TEXT NOT NULL,
    pipe_id BIGINT NOT NULL,
    change_type TEXT NOT NULL CHECK (change_type IN ('inbound', 'outbound', 'transfer', 'check_adjust')),
    ref_type TEXT,
    ref_id BIGINT,
    from_location_id BIGINT,
    to_location_id BIGINT,
    notes TEXT,
    created_by BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_inventory_logs_pipe ON inventory_logs(pipe_type, pipe_id);
CREATE INDEX idx_inventory_logs_created_at ON inventory_logs(created_at);
CREATE INDEX idx_inventory_logs_change_type ON inventory_logs(change_type);

-- Inventory check records
CREATE TABLE IF NOT EXISTS inventory_check_records (
    id BIGSERIAL PRIMARY KEY,
    check_no TEXT NOT NULL UNIQUE,
    location_id BIGINT,
    status TEXT NOT NULL DEFAULT 'in_progress' CHECK (status IN ('in_progress', 'completed', 'cancelled')),
    notes TEXT,
    created_by BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Inventory check items
CREATE TABLE IF NOT EXISTS inventory_check_items (
    id BIGSERIAL PRIMARY KEY,
    check_id BIGINT NOT NULL,
    pipe_type TEXT NOT NULL,
    pipe_id BIGINT NOT NULL,
    expected_status TEXT NOT NULL,
    found_status TEXT,
    is_match BOOLEAN,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_inventory_check_items_check_id ON inventory_check_items(check_id);
