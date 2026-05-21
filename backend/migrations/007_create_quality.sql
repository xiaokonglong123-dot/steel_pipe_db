-- Quality certificates
CREATE TABLE IF NOT EXISTS quality_certs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cert_number TEXT NOT NULL UNIQUE,
    pipe_type TEXT NOT NULL,
    pipe_id INTEGER NOT NULL,
    cert_date TEXT,
    result TEXT NOT NULL DEFAULT 'pending' CHECK (result IN ('pass', 'fail', 'pending')),
    inspector TEXT,
    inspection_body TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX idx_quality_certs_cert_number ON quality_certs(cert_number);
CREATE INDEX idx_quality_certs_pipe ON quality_certs(pipe_type, pipe_id);

-- API 5CT grade reference data
CREATE TABLE IF NOT EXISTS api_5ct_grade_ref (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    grade TEXT NOT NULL UNIQUE,
    yield_strength_min REAL,
    yield_strength_max REAL,
    tensile_strength_min REAL,
    hardness_max TEXT,
    carbon_content_max REAL,
    manganese_content_max REAL,
    phosphorus_content_max REAL,
    sulfur_content_max REAL,
    notes TEXT
);

-- Pipe attachments (files)
CREATE TABLE IF NOT EXISTS pipe_attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pipe_type TEXT NOT NULL,
    pipe_id INTEGER NOT NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    content_type TEXT,
    uploaded_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_pipe_attachments_pipe ON pipe_attachments(pipe_type, pipe_id);
