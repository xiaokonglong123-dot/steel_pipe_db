-- 007_finance.sql — 财务

CREATE TABLE accounts (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    parent_id   INTEGER REFERENCES accounts(id),
    account_type TEXT NOT NULL CHECK (account_type IN ('asset','liability','equity','income','expense')),
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE journal_entries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_no    TEXT NOT NULL UNIQUE,
    entry_date  TEXT NOT NULL,
    description TEXT,
    status      TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','posted','voided')),
    ref_type    TEXT,
    ref_id      INTEGER,
    created_by  INTEGER REFERENCES users(id),
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE journal_entry_lines (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id    INTEGER NOT NULL REFERENCES journal_entries(id),
    account_id  INTEGER NOT NULL REFERENCES accounts(id),
    debit       TEXT NOT NULL DEFAULT '0',
    credit      TEXT NOT NULL DEFAULT '0',
    description TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE invoices (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_no  TEXT NOT NULL UNIQUE,
    invoice_date TEXT NOT NULL,
    party_type  TEXT NOT NULL CHECK (party_type IN ('supplier','customer')),
    party_id    INTEGER NOT NULL,
    amount      TEXT NOT NULL,
    ref_type    TEXT,
    ref_id      INTEGER,
    status      TEXT NOT NULL DEFAULT 'unpaid' CHECK (status IN ('unpaid','partially_paid','paid','voided')),
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE payments (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    payment_no  TEXT NOT NULL UNIQUE,
    payment_date TEXT NOT NULL,
    supplier_id INTEGER REFERENCES suppliers(id),
    amount      TEXT NOT NULL,
    invoice_id  INTEGER REFERENCES invoices(id),
    method      TEXT,
    notes       TEXT,
    created_by  INTEGER REFERENCES users(id),
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
