-- 037_create_portal.sql
-- Supplier/customer portal: portal accounts bound to business parties,
-- enabling self-service PO confirmation and SO acknowledgement.

CREATE TABLE IF NOT EXISTS portal_accounts (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    party_type      VARCHAR(20) NOT NULL,       -- supplier | customer
    party_id        BIGINT NOT NULL,
    username        VARCHAR(100) NOT NULL,
    password_hash   TEXT NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    last_login_at   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (username)
);

CREATE INDEX idx_portal_party ON portal_accounts(party_type, party_id);

CREATE TABLE IF NOT EXISTS portal_events (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    party_type      VARCHAR(20) NOT NULL,
    party_id        BIGINT NOT NULL,
    event_type      VARCHAR(50) NOT NULL,       -- po_accepted | so_acknowledged
    ref_id          BIGINT NOT NULL,            -- purchase_order_id / sales_order_id
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_portal_events_party ON portal_events(party_type, party_id);
