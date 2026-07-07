-- 012_add_indexes.sql
-- Performance indexes for frequently queried columns across all business tables.
--
-- Index strategy:
--   - pipe_number: unique lookups (exact match by pipe identifier)
--   - status: list filtering (most queries filter by status)
--   - deleted_at: soft-delete exclusion (WHERE deleted_at IS NULL on every query)
--   - created_at: date-range filtering and sorting
--   - grade: pipe spec filtering (seamless pipes only)
--   - full_code: location hierarchy lookups
--   - entity_type/entity_id/user_id: audit log filtering (high-volume table)
--
-- All indexes use CREATE INDEX IF NOT EXISTS for idempotency.
-- SQLite WAL mode + these indexes provide sufficient read performance for typical workloads.
-- For very large datasets, consider adding composite indexes based on actual query patterns.

-- Seamless pipes
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_pipe_number ON seamless_pipes(pipe_number);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_status ON seamless_pipes(status);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_grade ON seamless_pipes(grade);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_deleted_at ON seamless_pipes(deleted_at);

-- Screen pipes
CREATE INDEX IF NOT EXISTS idx_screen_pipes_pipe_number ON screen_pipes(pipe_number);
CREATE INDEX IF NOT EXISTS idx_screen_pipes_status ON screen_pipes(status);
CREATE INDEX IF NOT EXISTS idx_screen_pipes_deleted_at ON screen_pipes(deleted_at);

-- Location (inventory locations)
CREATE INDEX IF NOT EXISTS idx_locations_full_code ON locations(full_code);
CREATE INDEX IF NOT EXISTS idx_locations_deleted_at ON locations(deleted_at);

-- Purchase orders
CREATE INDEX IF NOT EXISTS idx_purchase_orders_status ON purchase_orders(status);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_created_at ON purchase_orders(created_at);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_deleted_at ON purchase_orders(deleted_at);

-- Sales orders
CREATE INDEX IF NOT EXISTS idx_sales_orders_status ON sales_orders(status);
CREATE INDEX IF NOT EXISTS idx_sales_orders_created_at ON sales_orders(created_at);
CREATE INDEX IF NOT EXISTS idx_sales_orders_deleted_at ON sales_orders(deleted_at);

-- Quality certificates
CREATE INDEX IF NOT EXISTS idx_quality_certs_result ON quality_certs(result);
CREATE INDEX IF NOT EXISTS idx_quality_certs_created_at ON quality_certs(created_at);
CREATE INDEX IF NOT EXISTS idx_quality_certs_deleted_at ON quality_certs(deleted_at);

-- Operation logs (high-volume audit table — most critical)
CREATE INDEX IF NOT EXISTS idx_operation_logs_entity_type ON operation_logs(entity_type);
CREATE INDEX IF NOT EXISTS idx_operation_logs_entity_id ON operation_logs(entity_id);
CREATE INDEX IF NOT EXISTS idx_operation_logs_created_at ON operation_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_operation_logs_user_id ON operation_logs(user_id);