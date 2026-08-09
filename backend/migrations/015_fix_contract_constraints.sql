-- 015_fix_contract_constraints.sql
-- Fix contract payment_type and status CHECK constraints to match code.
-- Uses the SQLite copy-drop-rename pattern:
--   1. Create new table with corrected CHECK constraint
--   2. Copy data from old table
--   3. Drop old table
--   4. Rename new table
-- With INTEGER PRIMARY KEY AUTOINCREMENT, sqlite_sequence is advanced
-- automatically by the explicit-id INSERT ... SELECT, so no setval is needed
-- (the PostgreSQL port used setval() for this).

-- Fix contract_payments.payment_type: add 'progress' and 'retention'
CREATE TABLE IF NOT EXISTS contract_payments_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contract_id INTEGER NOT NULL,
    due_date TEXT NOT NULL,
    amount REAL NOT NULL,
    payment_type TEXT NOT NULL CHECK (payment_type IN ('deposit', 'progress', 'milestone', 'final', 'retention')),
    is_paid INTEGER NOT NULL DEFAULT 0,
    paid_date TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO contract_payments_new SELECT * FROM contract_payments;
DROP TABLE contract_payments;
ALTER TABLE contract_payments_new RENAME TO contract_payments;
CREATE INDEX IF NOT EXISTS idx_contract_payments_contract ON contract_payments(contract_id);

-- Fix contracts.status: add 'cancelled'
CREATE TABLE IF NOT EXISTS contracts_new (
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

INSERT INTO contracts_new SELECT * FROM contracts;
DROP TABLE contracts;
ALTER TABLE contracts_new RENAME TO contracts;

CREATE INDEX IF NOT EXISTS idx_contracts_contract_no ON contracts(contract_no);
CREATE INDEX IF NOT EXISTS idx_contracts_status ON contracts(status);
CREATE INDEX IF NOT EXISTS idx_contracts_type ON contracts(contract_type);
