-- 002_create_seamless_pipes.sql
-- Master data for API 5CT seamless pipes (casing & tubing).
-- Tracks pipe specs, dimensions, steel grade, heat treatment, threading, and status lifecycle.
-- No FK constraints — integrity enforced at application layer.
-- Soft delete via deleted_at column.
CREATE TABLE IF NOT EXISTS seamless_pipes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pipe_number TEXT NOT NULL UNIQUE,
    batch_number TEXT,
    pipe_type TEXT NOT NULL CHECK (pipe_type IN ('casing', 'tubing')),
    grade TEXT NOT NULL,
    od REAL NOT NULL,
    wt REAL NOT NULL,
    length REAL,
    weight_per_unit REAL,
    end_type TEXT,
    coupling_type TEXT,
    coupling_od REAL,
    coupling_length REAL,
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

CREATE INDEX idx_seamless_pipes_grade ON seamless_pipes(grade);
CREATE INDEX idx_seamless_pipes_heat_number ON seamless_pipes(heat_number);
CREATE INDEX idx_seamless_pipes_status ON seamless_pipes(status);
CREATE INDEX idx_seamless_pipes_location_id ON seamless_pipes(location_id);
CREATE INDEX idx_seamless_pipes_pipe_type ON seamless_pipes(pipe_type);
CREATE INDEX idx_seamless_pipes_od_wt ON seamless_pipes(od, wt);
CREATE INDEX idx_seamless_pipes_manufacturer ON seamless_pipes(manufacturer);
CREATE INDEX idx_seamless_pipes_search ON seamless_pipes(grade, od, wt, status);
CREATE INDEX idx_seamless_pipes_pipe_number ON seamless_pipes(pipe_number);
