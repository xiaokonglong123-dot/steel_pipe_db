-- 029_create_procurement.sql
-- Procurement deep-dive: purchase requisitions, goods receipts, supplier
-- quotes, supplier scorecard support.

CREATE TABLE IF NOT EXISTS purchase_requisitions (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    req_no      VARCHAR(40) NOT NULL,
    title       VARCHAR(200) NOT NULL,
    department_id BIGINT,
    applicant_id BIGINT,
    expected_date DATE,
    status      VARCHAR(20) NOT NULL DEFAULT 'draft',  -- draft | submitted | approved | rejected
    items_json  JSONB NOT NULL DEFAULT '[]',
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ,
    UNIQUE (tenant_id, req_no)
);

CREATE TABLE IF NOT EXISTS po_receipts (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    receipt_no  VARCHAR(40) NOT NULL,
    purchase_order_id BIGINT NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status      VARCHAR(20) NOT NULL DEFAULT 'received',  -- received | inspected | accepted
    notes       TEXT,
    created_by  BIGINT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, receipt_no)
);

CREATE TABLE IF NOT EXISTS po_receipt_items (
    id          BIGSERIAL PRIMARY KEY,
    receipt_id  BIGINT NOT NULL REFERENCES po_receipts(id) ON DELETE CASCADE,
    pipe_id     BIGINT,
    pipe_number VARCHAR(100),
    quantity    NUMERIC(14,2) NOT NULL DEFAULT 1,
    remark      TEXT
);

CREATE INDEX idx_pri_receipt ON po_receipt_items(receipt_id);

CREATE TABLE IF NOT EXISTS supplier_quotes (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    quote_no    VARCHAR(40) NOT NULL,
    supplier_id BIGINT NOT NULL,
    title       VARCHAR(200),
    valid_until DATE,
    total_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    status      VARCHAR(20) NOT NULL DEFAULT 'draft',  -- draft | sent | accepted | expired
    items_json  JSONB NOT NULL DEFAULT '[]',
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ,
    UNIQUE (tenant_id, quote_no)
);

CREATE INDEX idx_sq_supplier ON supplier_quotes(supplier_id);
