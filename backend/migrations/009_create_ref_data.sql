-- 009_create_ref_data.sql
-- Contracts and reference data tables.
-- Contracts: sales/procurement contracts with payment milestones and status tracking.
-- Contract items: line items linked to a contract, itemized (item_id).
-- Contract payments: payment milestones with due dates and amounts.
-- Soft delete via deleted_at column.
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), BOOLEAN -> INTEGER (1/0).
CREATE TABLE IF NOT EXISTS contracts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contract_no TEXT NOT NULL UNIQUE,
    contract_type TEXT NOT NULL CHECK (contract_type IN ('sales', 'purchase')),
    title TEXT NOT NULL,
    party_a TEXT NOT NULL,
    party_b TEXT NOT NULL,
    sign_date TEXT,
    start_date TEXT,
    end_date TEXT,
    total_amount REAL,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'completed', 'terminated', 'cancelled')),
    notes TEXT,
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_contracts_contract_no ON contracts(contract_no);
CREATE INDEX IF NOT EXISTS idx_contracts_status ON contracts(status);
CREATE INDEX IF NOT EXISTS idx_contracts_type ON contracts(contract_type);

-- Contract items
CREATE TABLE IF NOT EXISTS contract_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contract_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    quantity REAL NOT NULL,
    unit_price REAL,
    total_price REAL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_contract_items_contract ON contract_items(contract_id);
CREATE INDEX IF NOT EXISTS idx_contract_items_item ON contract_items(item_id);

-- Contract payment schedules
CREATE TABLE IF NOT EXISTS contract_payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contract_id INTEGER NOT NULL,
    due_date TEXT NOT NULL,
    amount REAL NOT NULL,
    payment_type TEXT NOT NULL CHECK (payment_type IN ('deposit', 'milestone', 'final')),
    is_paid INTEGER NOT NULL DEFAULT 0,
    paid_date TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_contract_payments_contract ON contract_payments(contract_id);
