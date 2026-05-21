-- Contracts
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
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'completed', 'terminated')),
    notes TEXT,
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX idx_contracts_contract_no ON contracts(contract_no);
CREATE INDEX idx_contracts_status ON contracts(status);
CREATE INDEX idx_contracts_type ON contracts(contract_type);

-- Contract items
CREATE TABLE IF NOT EXISTS contract_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contract_id INTEGER NOT NULL,
    pipe_type TEXT NOT NULL,
    grade TEXT NOT NULL,
    od REAL NOT NULL,
    wt REAL NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price REAL,
    total_price REAL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_contract_items_contract ON contract_items(contract_id);

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

CREATE INDEX idx_contract_payments_contract ON contract_payments(contract_id);
