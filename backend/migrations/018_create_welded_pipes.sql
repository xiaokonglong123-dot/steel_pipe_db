-- 018_create_welded_pipes.sql
-- Master data for API 5L welded pipes.
-- Tracks pipe specs, dimensions, steel grade, seam type, and status lifecycle.
-- No FK constraints — integrity enforced at application layer.
-- Soft delete via deleted_at column.
CREATE TABLE IF NOT EXISTS welded_pipes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pipe_number TEXT NOT NULL UNIQUE,
    batch_number TEXT,
    pipe_type TEXT NOT NULL CHECK (pipe_type IN ('erw', 'saw', 'hfi')),
    grade TEXT NOT NULL,
    od REAL NOT NULL,
    wt REAL NOT NULL,
    length REAL,
    weight_per_unit REAL,
    end_type TEXT,
    seam_type TEXT,
    heat_number TEXT,
    serial_number TEXT,
    manufacturer TEXT,
    production_date TEXT,
    cert_number TEXT,
    location_id INTEGER,
    status TEXT NOT NULL DEFAULT 'in_stock' CHECK (status IN ('in_stock', 'outbound', 'scrapped')),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

-- Indexes for common query patterns
CREATE INDEX idx_welded_pipes_pipe_number ON welded_pipes(pipe_number);
CREATE INDEX idx_welded_pipes_grade ON welded_pipes(grade);
CREATE INDEX idx_welded_pipes_pipe_type ON welded_pipes(pipe_type);
CREATE INDEX idx_welded_pipes_status ON welded_pipes(status);
CREATE INDEX idx_welded_pipes_heat_number ON welded_pipes(heat_number);
CREATE INDEX idx_welded_pipes_od_wt ON welded_pipes(od, wt);
CREATE INDEX idx_welded_pipes_location_id ON welded_pipes(location_id);
CREATE INDEX idx_welded_pipes_manufacturer ON welded_pipes(manufacturer);

-- Composite indexes for soft-delete + filter patterns
CREATE INDEX IF NOT EXISTS idx_wp_deleted_status
    ON welded_pipes(deleted_at, status);

CREATE INDEX IF NOT EXISTS idx_wp_deleted_type
    ON welded_pipes(deleted_at, pipe_type);

CREATE INDEX IF NOT EXISTS idx_wp_deleted_location
    ON welded_pipes(deleted_at, location_id);

CREATE INDEX IF NOT EXISTS idx_wp_deleted_grade_status
    ON welded_pipes(deleted_at, grade, status);

-- Covering index for the main list page (index-only scan)
CREATE INDEX IF NOT EXISTS idx_wp_list_cover
    ON welded_pipes(deleted_at, status, pipe_number, grade, od, wt, location_id, pipe_type);