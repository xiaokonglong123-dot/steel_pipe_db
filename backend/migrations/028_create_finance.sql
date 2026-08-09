-- 028_create_finance.sql
-- Finance module: chart of accounts, journal entries (with details),
-- invoices (AR/AP), payments. SQLite has no schemas.
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), DATE -> TEXT,
-- BOOLEAN -> INTEGER (1/0), NUMERIC(18,2) -> NUMERIC.

CREATE TABLE IF NOT EXISTS chart_of_accounts (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    code        TEXT NOT NULL,
    name        TEXT NOT NULL,
    account_type TEXT NOT NULL,   -- asset | liability | equity | revenue | expense
    parent_id   INTEGER,
    is_active   INTEGER NOT NULL DEFAULT 1.0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT,
    UNIQUE (tenant_id, code)
);

CREATE TABLE IF NOT EXISTS journal_entries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    entry_no    TEXT NOT NULL,
    entry_date  TEXT NOT NULL,                -- DATE -> TEXT
    description TEXT,
    status      TEXT NOT NULL DEFAULT 'draft',  -- draft | posted
    currency    TEXT NOT NULL DEFAULT 'CNY',
    created_by  INTEGER,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    posted_at   TEXT,
    UNIQUE (tenant_id, entry_no)
);

CREATE TABLE IF NOT EXISTS journal_entry_details (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id    INTEGER NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    account_id  INTEGER NOT NULL REFERENCES chart_of_accounts(id),
    debit       REAL NOT NULL DEFAULT 0.0,
    credit      REAL NOT NULL DEFAULT 0.0,
    description TEXT
);

CREATE INDEX IF NOT EXISTS idx_jed_entry ON journal_entry_details(entry_id);

CREATE TABLE IF NOT EXISTS finance_invoices (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    invoice_no  TEXT NOT NULL,
    invoice_type TEXT NOT NULL,   -- sales (AR) | purchase (AP)
    party_id    INTEGER NOT NULL, -- customer_id (sales) or supplier_id (purchase)
    order_id    INTEGER,          -- originating sales/purchase order
    amount      REAL NOT NULL DEFAULT 0.0,
    tax_amount  REAL NOT NULL DEFAULT 0.0,
    total_amount REAL NOT NULL DEFAULT 0.0,
    status      TEXT NOT NULL DEFAULT 'draft',  -- draft | confirmed | paid | void
    due_date    TEXT,             -- DATE -> TEXT
    issued_at   TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (tenant_id, invoice_no)
);

CREATE TABLE IF NOT EXISTS finance_invoice_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id  INTEGER NOT NULL REFERENCES finance_invoices(id) ON DELETE CASCADE,
    description TEXT,
    quantity    REAL NOT NULL DEFAULT 1.0,
    unit_price  REAL NOT NULL DEFAULT 0.0,
    amount      REAL NOT NULL DEFAULT 0.0
);

CREATE INDEX IF NOT EXISTS idx_fii_invoice ON finance_invoice_items(invoice_id);

CREATE TABLE IF NOT EXISTS finance_payments (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL DEFAULT 1.0,
    payment_no  TEXT NOT NULL,
    invoice_id  INTEGER REFERENCES finance_invoices(id),
    direction   TEXT NOT NULL,     -- in (receipt) | out (payment)
    amount      REAL NOT NULL DEFAULT 0.0,
    method      TEXT NOT NULL DEFAULT 'bank_transfer',  -- bank_transfer | cash | check
    paid_at     TEXT NOT NULL DEFAULT (datetime('now')),
    reference   TEXT,
    created_by  INTEGER,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (tenant_id, payment_no)
);

CREATE INDEX IF NOT EXISTS idx_fp_invoice ON finance_payments(invoice_id);
