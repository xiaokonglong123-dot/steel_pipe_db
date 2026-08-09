-- 012_add_indexes.sql
-- Performance indexes for frequently queried columns across business tables.
--
-- Index strategy (generic ERP):
--   - status: list filtering (most queries filter by status)
--   - deleted_at: soft-delete exclusion (WHERE deleted_at IS NULL on every query)
--   - created_at: date-range filtering and sorting
--   - full_code: location hierarchy lookups
--   - entity_type/entity_id/user_id: audit log filtering (high-volume table)
--
-- Pipe-table indexes (seamless/screen/welded) and quality-cert indexes were
-- removed together with the dropped tables.
-- All indexes use CREATE INDEX IF NOT EXISTS for idempotency.

-- Locations (inventory locations)
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

-- Operation logs (high-volume audit table — most critical)
CREATE INDEX IF NOT EXISTS idx_operation_logs_entity_type ON operation_logs(entity_type);
CREATE INDEX IF NOT EXISTS idx_operation_logs_entity_id ON operation_logs(entity_id);
CREATE INDEX IF NOT EXISTS idx_operation_logs_created_at ON operation_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_operation_logs_user_id ON operation_logs(user_id);
