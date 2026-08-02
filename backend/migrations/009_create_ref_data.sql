-- 009_create_ref_data.sql
-- Contracts and reference data tables.
-- Contracts: sales/procurement contracts with payment milestones and status tracking.
-- Contract items: line items linked to a contract.
-- Contract payments: payment milestones with due dates and amounts.
-- Soft delete via deleted_at column.
CREATE TABLE IF NOT EXISTS contracts (
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
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'completed', 'terminated')),
    notes TEXT,
    created_by BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_contracts_contract_no ON contracts(contract_no);
CREATE INDEX idx_contracts_status ON contracts(status);
CREATE INDEX idx_contracts_type ON contracts(contract_type);

-- Contract items
CREATE TABLE IF NOT EXISTS contract_items (
    id BIGSERIAL PRIMARY KEY,
    contract_id BIGINT NOT NULL,
    pipe_type TEXT NOT NULL,
    grade TEXT NOT NULL,
    od DOUBLE PRECISION NOT NULL,
    wt DOUBLE PRECISION NOT NULL,
    quantity BIGINT NOT NULL,
    unit_price DOUBLE PRECISION,
    total_price DOUBLE PRECISION,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contract_items_contract ON contract_items(contract_id);

-- Contract payment schedules
CREATE TABLE IF NOT EXISTS contract_payments (
    id BIGSERIAL PRIMARY KEY,
    contract_id BIGINT NOT NULL,
    due_date TIMESTAMPTZ NOT NULL,
    amount DOUBLE PRECISION NOT NULL,
    payment_type TEXT NOT NULL CHECK (payment_type IN ('deposit', 'milestone', 'final')),
    is_paid BOOLEAN NOT NULL DEFAULT FALSE,
    paid_date TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contract_payments_contract ON contract_payments(contract_id);
