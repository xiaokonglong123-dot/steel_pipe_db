-- 030_create_sales_crm.sql
-- Sales CRM deep-dive: shipments (delivery tracking) and sales quotes
-- (quotation → order conversion).

CREATE TABLE IF NOT EXISTS sales_shipments (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    shipment_no VARCHAR(40) NOT NULL,
    sales_order_id BIGINT NOT NULL,
    shipped_at  TIMESTAMPTZ,
    carrier     VARCHAR(100),
    tracking_no VARCHAR(100),
    status      VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending | shipped | delivered
    items_json  JSONB NOT NULL DEFAULT '[]',
    notes       TEXT,
    created_by  BIGINT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, shipment_no)
);

CREATE INDEX idx_ss_order ON sales_shipments(sales_order_id);

CREATE TABLE IF NOT EXISTS sales_quotes (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    quote_no    VARCHAR(40) NOT NULL,
    customer_id BIGINT NOT NULL,
    quote_date  DATE NOT NULL DEFAULT CURRENT_DATE,
    valid_until DATE,
    total_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    status      VARCHAR(20) NOT NULL DEFAULT 'draft',  -- draft | confirmed | converted | expired
    items_json  JSONB NOT NULL DEFAULT '[]',
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ,
    UNIQUE (tenant_id, quote_no)
);

CREATE INDEX idx_sq_customer ON sales_quotes(customer_id);
