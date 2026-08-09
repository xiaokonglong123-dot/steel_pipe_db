-- 037_create_portal.sql
-- Supplier/customer portal: portal accounts bound to business parties,
-- enabling self-service PO confirmation and SO acknowledgement.
--
-- SQLite port: BIGSERIAL -> INTEGER PRIMARY KEY AUTOINCREMENT,
-- TIMESTAMPTZ -> TEXT, NOW() -> datetime('now'), BOOLEAN -> INTEGER (1/0).

CREATE TABLE IF NOT EXISTS portal_accounts (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1,
    party_type      TEXT NOT NULL,              -- supplier | customer
    party_id        INTEGER NOT NULL,
    username        TEXT NOT NULL,
    password_hash   TEXT NOT NULL,
    is_active       INTEGER NOT NULL DEFAULT 1,
    last_login_at   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (username)
);

CREATE INDEX IF NOT EXISTS idx_portal_party ON portal_accounts(party_type, party_id);

CREATE TABLE IF NOT EXISTS portal_events (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id       INTEGER NOT NULL DEFAULT 1,
    party_type      TEXT NOT NULL,
    party_id        INTEGER NOT NULL,
    event_type      TEXT NOT NULL,              -- po_accepted | so_acknowledged
    ref_id          INTEGER NOT NULL,           -- purchase_order_id / sales_order_id
    notes           TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_portal_events_party ON portal_events(party_type, party_id);
