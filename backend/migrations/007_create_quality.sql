-- 007_create_quality.sql
-- Quality inspection tables: certificates, mechanical test results, NDT results.
-- Certificates link to pipes and track inspection status (draft → issued → revoked).
-- Mechanical tests: tensile, yield, elongation, impact, hardness.
-- NDT types: UT (ultrasonic), MI (magnetic particle), MPI (magnetic particle inspection).
-- Soft delete via deleted_at column.
CREATE TABLE IF NOT EXISTS quality_certs (
    id BIGSERIAL PRIMARY KEY,
    cert_number TEXT NOT NULL UNIQUE,
    pipe_type TEXT NOT NULL,
    pipe_id BIGINT NOT NULL,
    cert_date TIMESTAMPTZ,
    result TEXT NOT NULL DEFAULT 'pending' CHECK (result IN ('pass', 'fail', 'pending')),
    inspector TEXT,
    inspection_body TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_quality_certs_cert_number ON quality_certs(cert_number);
CREATE INDEX idx_quality_certs_pipe ON quality_certs(pipe_type, pipe_id);

-- API 5CT grade reference data
CREATE TABLE IF NOT EXISTS api_5ct_grade_ref (
    id BIGSERIAL PRIMARY KEY,
    grade TEXT NOT NULL UNIQUE,
    yield_strength_min DOUBLE PRECISION,
    yield_strength_max DOUBLE PRECISION,
    tensile_strength_min DOUBLE PRECISION,
    hardness_max TEXT,
    carbon_content_max DOUBLE PRECISION,
    manganese_content_max DOUBLE PRECISION,
    phosphorus_content_max DOUBLE PRECISION,
    sulfur_content_max DOUBLE PRECISION,
    notes TEXT
);

-- Pipe attachments (files)
CREATE TABLE IF NOT EXISTS pipe_attachments (
    id BIGSERIAL PRIMARY KEY,
    pipe_type TEXT NOT NULL,
    pipe_id BIGINT NOT NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT,
    content_type TEXT,
    uploaded_by BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pipe_attachments_pipe ON pipe_attachments(pipe_type, pipe_id);
