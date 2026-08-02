-- 015_fix_contract_constraints.sql
-- Fix contract payment_type and status CHECK constraints to match code.
-- PostgreSQL port keeps the copy-drop-rename pattern from the SQLite version.
-- After each copy, the BIGSERIAL sequence is advanced past the copied ids via
-- setval — otherwise the next auto-generated id would collide with copied rows.

-- Fix contract_payments.payment_type: add 'progress' and 'retention'
CREATE TABLE IF NOT EXISTS contract_payments_new (
    id BIGSERIAL PRIMARY KEY,
    contract_id BIGINT NOT NULL,
    due_date TIMESTAMPTZ NOT NULL,
    amount DOUBLE PRECISION NOT NULL,
    payment_type TEXT NOT NULL CHECK (payment_type IN ('deposit', 'progress', 'milestone', 'final', 'retention')),
    is_paid BOOLEAN NOT NULL DEFAULT FALSE,
    paid_date TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO contract_payments_new SELECT * FROM contract_payments;
DROP TABLE contract_payments;
ALTER TABLE contract_payments_new RENAME TO contract_payments;
SELECT setval(pg_get_serial_sequence('contract_payments', 'id'), COALESCE((SELECT MAX(id) FROM contract_payments), 1));
CREATE INDEX idx_contract_payments_contract ON contract_payments(contract_id);

-- Fix contracts.status: add 'cancelled'
CREATE TABLE IF NOT EXISTS contracts_new (
    id BIGSERIAL PRIMARY KEY,
    contract_no TEXT NOT NULL UNIQUE,
    contract_type TEXT NOT NULL CHECK (contract_type IN ('sales', 'purchase')),
    title TEXT NOT NULL,
    party_a TEXT NOT NULL,
    party_b TEXT NOT NULL,
    sign_date TIMESTAMPTZ,
    start_date TIMESTAMPTZ,
    end_date TIMESTAMPTZ,
    total_amount DOUBLE PRECISION,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'completed', 'terminated', 'cancelled')),
    notes TEXT,
    created_by BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

INSERT INTO contracts_new SELECT * FROM contracts;
DROP TABLE contracts;
ALTER TABLE contracts_new RENAME TO contracts;
SELECT setval(pg_get_serial_sequence('contracts', 'id'), COALESCE((SELECT MAX(id) FROM contracts), 1));

CREATE INDEX idx_contracts_contract_no ON contracts(contract_no);
CREATE INDEX idx_contracts_status ON contracts(status);
CREATE INDEX idx_contracts_type ON contracts(contract_type);
