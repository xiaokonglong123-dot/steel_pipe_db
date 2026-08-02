-- 002_create_seamless_pipes.sql
-- Master data for API 5CT seamless pipes (casing & tubing).
-- Tracks pipe specs, dimensions, steel grade, heat treatment, threading, and status lifecycle.
-- No FK constraints — integrity enforced at application layer.
-- Soft delete via deleted_at column.
--
-- NOTE: the status CHECK uses the full PipeStatus enum set (matching
-- 019_fix_status_checks_and_indexes.sql) instead of the original 3-value list,
-- because 017_seed_initial_data.sql inserts rows with status 'new' — which the
-- original CHECK rejected (on SQLite as well; the fresh-install chain was broken).
CREATE TABLE IF NOT EXISTS seamless_pipes (
    id BIGSERIAL PRIMARY KEY,
    pipe_number TEXT NOT NULL UNIQUE,
    batch_number TEXT,
    pipe_type TEXT NOT NULL CHECK (pipe_type IN ('casing', 'tubing')),
    grade TEXT NOT NULL,
    od DOUBLE PRECISION NOT NULL,
    wt DOUBLE PRECISION NOT NULL,
    length DOUBLE PRECISION,
    weight_per_unit DOUBLE PRECISION,
    end_type TEXT,
    coupling_type TEXT,
    coupling_od DOUBLE PRECISION,
    coupling_length DOUBLE PRECISION,
    heat_number TEXT,
    serial_number TEXT,
    manufacturer TEXT,
    production_date TIMESTAMPTZ,
    cert_number TEXT,
    location_id BIGINT,
    status TEXT NOT NULL DEFAULT 'in_stock' CHECK (status IN ('new', 'in_stock', 'outbound', 'scrapped', 'in_transit', 'reserved')),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_seamless_pipes_grade ON seamless_pipes(grade);
CREATE INDEX idx_seamless_pipes_heat_number ON seamless_pipes(heat_number);
CREATE INDEX idx_seamless_pipes_status ON seamless_pipes(status);
CREATE INDEX idx_seamless_pipes_location_id ON seamless_pipes(location_id);
CREATE INDEX idx_seamless_pipes_pipe_type ON seamless_pipes(pipe_type);
CREATE INDEX idx_seamless_pipes_od_wt ON seamless_pipes(od, wt);
CREATE INDEX idx_seamless_pipes_manufacturer ON seamless_pipes(manufacturer);
CREATE INDEX idx_seamless_pipes_search ON seamless_pipes(grade, od, wt, status);
CREATE INDEX idx_seamless_pipes_pipe_number ON seamless_pipes(pipe_number);
