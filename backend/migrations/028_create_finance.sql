-- 028_create_finance.sql
-- Finance module: chart of accounts, journal entries (with details),
-- invoices (AR/AP), payments. All in public schema.

CREATE TABLE IF NOT EXISTS chart_of_accounts (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    code        VARCHAR(30) NOT NULL,
    name        VARCHAR(100) NOT NULL,
    account_type VARCHAR(20) NOT NULL,   -- asset | liability | equity | revenue | expense
    parent_id   BIGINT,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ,
    UNIQUE (tenant_id, code)
);

CREATE TABLE IF NOT EXISTS journal_entries (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    entry_no    VARCHAR(40) NOT NULL,
    entry_date  DATE NOT NULL,
    description TEXT,
    status      VARCHAR(20) NOT NULL DEFAULT 'draft',  -- draft | posted
    currency    VARCHAR(3) NOT NULL DEFAULT 'CNY',
    created_by  BIGINT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    posted_at   TIMESTAMPTZ,
    UNIQUE (tenant_id, entry_no)
);

CREATE TABLE IF NOT EXISTS journal_entry_details (
    id          BIGSERIAL PRIMARY KEY,
    entry_id    BIGINT NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    account_id  BIGINT NOT NULL REFERENCES chart_of_accounts(id),
    debit       NUMERIC(18,2) NOT NULL DEFAULT 0,
    credit      NUMERIC(18,2) NOT NULL DEFAULT 0,
    description TEXT
);

CREATE INDEX idx_jed_entry ON journal_entry_details(entry_id);

CREATE TABLE IF NOT EXISTS finance_invoices (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    invoice_no  VARCHAR(40) NOT NULL,
    invoice_type VARCHAR(20) NOT NULL,   -- sales (AR) | purchase (AP)
    party_id    BIGINT NOT NULL,         -- customer_id (sales) or supplier_id (purchase)
    order_id    BIGINT,                  -- originating sales/purchase order
    amount      NUMERIC(18,2) NOT NULL DEFAULT 0,
    tax_amount  NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
    status      VARCHAR(20) NOT NULL DEFAULT 'draft',  -- draft | confirmed | paid | void
    due_date    DATE,
    issued_at   TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, invoice_no)
);

CREATE TABLE IF NOT EXISTS finance_invoice_items (
    id          BIGSERIAL PRIMARY KEY,
    invoice_id  BIGINT NOT NULL REFERENCES finance_invoices(id) ON DELETE CASCADE,
    description VARCHAR(200),
    quantity    NUMERIC(14,2) NOT NULL DEFAULT 1,
    unit_price  NUMERIC(18,2) NOT NULL DEFAULT 0,
    amount      NUMERIC(18,2) NOT NULL DEFAULT 0
);

CREATE INDEX idx_fii_invoice ON finance_invoice_items(invoice_id);

CREATE TABLE IF NOT EXISTS finance_payments (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    payment_no  VARCHAR(40) NOT NULL,
    invoice_id  BIGINT REFERENCES finance_invoices(id),
    direction   VARCHAR(10) NOT NULL,     -- in (receipt) | out (payment)
    amount      NUMERIC(18,2) NOT NULL DEFAULT 0,
    method      VARCHAR(20) NOT NULL DEFAULT 'bank_transfer',  -- bank_transfer | cash | check
    paid_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reference   VARCHAR(100),
    created_by  BIGINT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, payment_no)
);

CREATE INDEX idx_fp_invoice ON finance_payments(invoice_id);
