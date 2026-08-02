-- 003_create_screen_pipes.sql
-- Master data for API 5CT screen pipes (slotted / wire-wrapped).
-- Similar structure to seamless_pipes but with screen-specific fields (slot width, slot pattern, etc).
-- No FK constraints — integrity enforced at application layer.
-- Soft delete via deleted_at column.
CREATE TABLE IF NOT EXISTS screen_pipes (
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
    status TEXT NOT NULL DEFAULT 'in_stock' CHECK (status IN ('in_stock', 'outbound', 'scrapped')),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_screen_pipes_screen_type ON screen_pipes(screen_type);
CREATE INDEX idx_screen_pipes_base_grade ON screen_pipes(base_grade);
CREATE INDEX idx_screen_pipes_status ON screen_pipes(status);
CREATE INDEX idx_screen_pipes_heat_number ON screen_pipes(heat_number);
CREATE INDEX idx_screen_pipes_pipe_number ON screen_pipes(pipe_number);
