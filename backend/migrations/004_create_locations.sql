-- 004_create_locations.sql
-- Warehouse storage locations organized in a zone → shelf → level hierarchy.
-- Each location can hold pipes and tracks capacity usage.
-- Soft delete via deleted_at column.
CREATE TABLE IF NOT EXISTS locations (
    id BIGSERIAL PRIMARY KEY,
    zone_code TEXT NOT NULL,
    shelf_code TEXT NOT NULL,
    level_code TEXT NOT NULL,
    full_code TEXT NOT NULL UNIQUE,
    description TEXT,
    capacity BIGINT,
    used_count BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_locations_full_code ON locations(full_code);
CREATE INDEX idx_locations_zone_code ON locations(zone_code);
