-- 035_create_assets.sql
-- Fixed assets: registration, straight-line depreciation, disposal.

CREATE TABLE IF NOT EXISTS fixed_assets (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL DEFAULT 1,
    asset_no        VARCHAR(40) NOT NULL,
    name            VARCHAR(200) NOT NULL,
    category        VARCHAR(50) NOT NULL DEFAULT 'equipment',  -- equipment | vehicle | building | tooling
    purchase_date   DATE NOT NULL,
    purchase_cost   NUMERIC(18,2) NOT NULL DEFAULT 0,
    salvage_value   NUMERIC(18,2) NOT NULL DEFAULT 0,
    useful_life_months INT NOT NULL DEFAULT 60,
    current_value   NUMERIC(18,2) NOT NULL DEFAULT 0,
    status          VARCHAR(20) NOT NULL DEFAULT 'active',  -- active | disposed
    location        VARCHAR(100),
    department_id   BIGINT,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    UNIQUE (tenant_id, asset_no)
);

CREATE TABLE IF NOT EXISTS depreciation_entries (
    id          BIGSERIAL PRIMARY KEY,
    asset_id    BIGINT NOT NULL REFERENCES fixed_assets(id) ON DELETE CASCADE,
    period      VARCHAR(7) NOT NULL,           -- 'YYYY-MM'
    amount      NUMERIC(18,2) NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (asset_id, period)
);

CREATE INDEX idx_dep_asset ON depreciation_entries(asset_id);
