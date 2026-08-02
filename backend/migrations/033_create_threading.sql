-- 033_create_threading.sql
-- Pipe threading: machining records + geometry cache (API 5CT thread math).

CREATE TABLE IF NOT EXISTS threading_records (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    pipe_id     BIGINT,
    pipe_number VARCHAR(100),
    thread_type VARCHAR(50) NOT NULL,        -- API 5B round | buttress | API 5C
    od          DOUBLE PRECISION NOT NULL,   -- pipe outside diameter (mm)
    wt          DOUBLE PRECISION NOT NULL,   -- wall thickness (mm)
    grade       VARCHAR(20),                 -- J55 | N80 | L80 | P110 ...
    threads_per_inch DOUBLE PRECISION,
    pitch_diameter DOUBLE PRECISION,         -- measured pitch diameter (mm)
    makeup_torque DOUBLE PRECISION,          -- ft-lbs
    machined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    operator    BIGINT,
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tr_pipe ON threading_records(pipe_id);

CREATE TABLE IF NOT EXISTS thread_geometry_cache (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL DEFAULT 1,
    pipe_type   VARCHAR(50) NOT NULL,
    od          DOUBLE PRECISION NOT NULL,
    wt          DOUBLE PRECISION NOT NULL,
    grade       VARCHAR(20) NOT NULL,
    connection_type VARCHAR(50) NOT NULL,    -- round | buttress | premium
    joint_efficiency DOUBLE PRECISION,       -- 0..1
    burst_pressure DOUBLE PRECISION,         -- psi
    collapse_pressure DOUBLE PRECISION,      -- psi
    tension_capacity DOUBLE PRECISION,       -- lbs
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (pipe_type, od, wt, grade, connection_type)
);
