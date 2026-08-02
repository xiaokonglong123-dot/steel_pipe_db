-- 019_fix_status_checks_and_indexes.sql
-- Fix CHECK constraints on status columns for seamless_pipes, screen_pipes,
-- and welded_pipes to include all statuses used by the Rust PipeStatus enum:
--   new, in_stock, outbound, scrapped, in_transit, reserved
--
-- The copy-drop-rename pattern is preserved from the SQLite version:
--   1. Create new table with corrected CHECK constraint
--   2. Copy data from old table
--   3. Drop old table
--   4. Rename new table
--   5. Advance the BIGSERIAL sequence past the copied ids (setval) —
--      without this the next auto-generated id would collide with copied rows
--   6. Recreate all indexes
--
-- Also adds missing pipe traceability indexes on inventory-related tables.
-- No FK constraints in this project (enforced at app layer), so no FK concerns
-- and no PRAGMA equivalent is needed around the rebuilds.

-- ============================================================
-- 1. Fix seamless_pipes status CHECK constraint
-- ============================================================

CREATE TABLE IF NOT EXISTS seamless_pipes_new (
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

INSERT INTO seamless_pipes_new SELECT * FROM seamless_pipes;
DROP TABLE seamless_pipes;
ALTER TABLE seamless_pipes_new RENAME TO seamless_pipes;
SELECT setval(pg_get_serial_sequence('seamless_pipes', 'id'), COALESCE((SELECT MAX(id) FROM seamless_pipes), 1));

-- Recreate seamless_pipes indexes
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_grade ON seamless_pipes(grade);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_heat_number ON seamless_pipes(heat_number);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_status ON seamless_pipes(status);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_location_id ON seamless_pipes(location_id);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_pipe_type ON seamless_pipes(pipe_type);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_od_wt ON seamless_pipes(od, wt);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_manufacturer ON seamless_pipes(manufacturer);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_search ON seamless_pipes(grade, od, wt, status);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_pipe_number ON seamless_pipes(pipe_number);
CREATE INDEX IF NOT EXISTS idx_sp_deleted_status ON seamless_pipes(deleted_at, status);
CREATE INDEX IF NOT EXISTS idx_sp_deleted_type ON seamless_pipes(deleted_at, pipe_type);
CREATE INDEX IF NOT EXISTS idx_sp_deleted_location ON seamless_pipes(deleted_at, location_id);
CREATE INDEX IF NOT EXISTS idx_sp_deleted_grade_status ON seamless_pipes(deleted_at, grade, status);
CREATE INDEX IF NOT EXISTS idx_sp_list_cover ON seamless_pipes(deleted_at, status, pipe_number, grade, od, wt, location_id, pipe_type);

-- ============================================================
-- 2. Fix screen_pipes status CHECK constraint
-- ============================================================

CREATE TABLE IF NOT EXISTS screen_pipes_new (
    id BIGSERIAL PRIMARY KEY,
    pipe_number TEXT NOT NULL UNIQUE,
    batch_number TEXT,
    screen_type TEXT NOT NULL CHECK (screen_type IN ('wire_wrapped', 'slotted', 'punched', 'metal_felt')),
    slot_size DOUBLE PRECISION,
    filtration_grade TEXT,
    base_od DOUBLE PRECISION NOT NULL,
    base_wt DOUBLE PRECISION NOT NULL,
    base_grade TEXT NOT NULL,
    base_end_type TEXT,
    length DOUBLE PRECISION,
    weight_per_unit DOUBLE PRECISION,
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

INSERT INTO screen_pipes_new SELECT * FROM screen_pipes;
DROP TABLE screen_pipes;
ALTER TABLE screen_pipes_new RENAME TO screen_pipes;
SELECT setval(pg_get_serial_sequence('screen_pipes', 'id'), COALESCE((SELECT MAX(id) FROM screen_pipes), 1));

-- Recreate screen_pipes indexes
CREATE INDEX IF NOT EXISTS idx_screen_pipes_screen_type ON screen_pipes(screen_type);
CREATE INDEX IF NOT EXISTS idx_screen_pipes_base_grade ON screen_pipes(base_grade);
CREATE INDEX IF NOT EXISTS idx_screen_pipes_status ON screen_pipes(status);
CREATE INDEX IF NOT EXISTS idx_screen_pipes_heat_number ON screen_pipes(heat_number);
CREATE INDEX IF NOT EXISTS idx_screen_pipes_pipe_number ON screen_pipes(pipe_number);
CREATE INDEX IF NOT EXISTS idx_scrp_deleted_status ON screen_pipes(deleted_at, status);
CREATE INDEX IF NOT EXISTS idx_scrp_deleted_type ON screen_pipes(deleted_at, screen_type);
CREATE INDEX IF NOT EXISTS idx_scrp_deleted_location ON screen_pipes(deleted_at, location_id);
CREATE INDEX IF NOT EXISTS idx_scrp_deleted_grade_status ON screen_pipes(deleted_at, base_grade, status);
CREATE INDEX IF NOT EXISTS idx_scrp_list_cover ON screen_pipes(deleted_at, status, pipe_number, base_grade, base_od, base_wt, location_id, screen_type);

-- ============================================================
-- 3. Fix welded_pipes status CHECK constraint
-- ============================================================

CREATE TABLE IF NOT EXISTS welded_pipes_new (
    id BIGSERIAL PRIMARY KEY,
    pipe_number TEXT NOT NULL UNIQUE,
    batch_number TEXT,
    pipe_type TEXT NOT NULL CHECK (pipe_type IN ('erw', 'saw', 'hfi')),
    grade TEXT NOT NULL,
    od DOUBLE PRECISION NOT NULL,
    wt DOUBLE PRECISION NOT NULL,
    length DOUBLE PRECISION,
    weight_per_unit DOUBLE PRECISION,
    end_type TEXT,
    seam_type TEXT,
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

INSERT INTO welded_pipes_new SELECT * FROM welded_pipes;
DROP TABLE welded_pipes;
ALTER TABLE welded_pipes_new RENAME TO welded_pipes;
SELECT setval(pg_get_serial_sequence('welded_pipes', 'id'), COALESCE((SELECT MAX(id) FROM welded_pipes), 1));

-- Recreate welded_pipes indexes
CREATE INDEX IF NOT EXISTS idx_welded_pipes_pipe_number ON welded_pipes(pipe_number);
CREATE INDEX IF NOT EXISTS idx_welded_pipes_grade ON welded_pipes(grade);
CREATE INDEX IF NOT EXISTS idx_welded_pipes_pipe_type ON welded_pipes(pipe_type);
CREATE INDEX IF NOT EXISTS idx_welded_pipes_status ON welded_pipes(status);
CREATE INDEX IF NOT EXISTS idx_welded_pipes_heat_number ON welded_pipes(heat_number);
CREATE INDEX IF NOT EXISTS idx_welded_pipes_od_wt ON welded_pipes(od, wt);
CREATE INDEX IF NOT EXISTS idx_welded_pipes_location_id ON welded_pipes(location_id);
CREATE INDEX IF NOT EXISTS idx_welded_pipes_manufacturer ON welded_pipes(manufacturer);
CREATE INDEX IF NOT EXISTS idx_wp_deleted_status ON welded_pipes(deleted_at, status);
CREATE INDEX IF NOT EXISTS idx_wp_deleted_type ON welded_pipes(deleted_at, pipe_type);
CREATE INDEX IF NOT EXISTS idx_wp_deleted_location ON welded_pipes(deleted_at, location_id);
CREATE INDEX IF NOT EXISTS idx_wp_deleted_grade_status ON welded_pipes(deleted_at, grade, status);
CREATE INDEX IF NOT EXISTS idx_wp_list_cover ON welded_pipes(deleted_at, status, pipe_number, grade, od, wt, location_id, pipe_type);

-- ============================================================
-- 4. Add pipe traceability indexes on inventory tables
-- ============================================================
-- These speed up queries that link inventory movements back to specific pipes.
-- inbound_items and outbound_items indexes already exist from 005_create_inventory.sql;
-- inventory_check_items is still missing its pipe index.

CREATE INDEX IF NOT EXISTS idx_inbound_items_pipe ON inbound_items(pipe_type, pipe_id);
CREATE INDEX IF NOT EXISTS idx_outbound_items_pipe ON outbound_items(pipe_type, pipe_id);
CREATE INDEX IF NOT EXISTS idx_check_items_pipe ON inventory_check_items(pipe_type, pipe_id);
