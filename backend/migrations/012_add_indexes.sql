-- Add performance indexes for frequently queried columns
-- Indexes on: pipe_number (unique lookups), status (list filtering), composite lookup keys
-- Indexes on: entity_type+entity_id+created_at (audit log filtering)

-- Seamless pipes
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_pipe_number ON seamless_pipes(pipe_number);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_status ON seamless_pipes(status);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_grade ON seamless_pipes(grade);
CREATE INDEX IF NOT EXISTS idx_seamless_pipes_deleted_at ON seamless_pipes(deleted_at);

-- Screen pipes
CREATE INDEX IF NOT EXISTS idx_screen_pipes_pipe_number ON screen_pipes(pipe_number);
CREATE INDEX IF NOT EXISTS idx_screen_pipes_status ON screen_pipes(status);
CREATE INDEX IF NOT EXISTS idx_screen_pipes_deleted_at ON screen_pipes(deleted_at);

-- Inventory
CREATE INDEX IF NOT EXISTS idx_inventory_location_id ON inventory(location_id);
CREATE INDEX IF NOT EXISTS idx_inventory_pipe_type ON inventory(pipe_type);
CREATE INDEX IF NOT EXISTS idx_inventory_pipe_id ON inventory(pipe_id);
CREATE INDEX IF NOT EXISTS idx_inventory_deleted_at ON inventory(deleted_at);

-- Purchase orders
CREATE INDEX IF NOT EXISTS idx_purchase_orders_status ON purchase_orders(status);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_created_at ON purchase_orders(created_at);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_deleted_at ON purchase_orders(deleted_at);

-- Sales orders
CREATE INDEX IF NOT EXISTS idx_sales_orders_status ON sales_orders(status);
CREATE INDEX IF NOT EXISTS idx_sales_orders_created_at ON sales_orders(created_at);
CREATE INDEX IF NOT EXISTS idx_sales_orders_deleted_at ON sales_orders(deleted_at);

-- Quality certificates
CREATE INDEX IF NOT EXISTS idx_quality_certs_grade ON quality_certificates(grade);
CREATE INDEX IF NOT EXISTS idx_quality_certs_created_at ON quality_certificates(created_at);
CREATE INDEX IF NOT EXISTS idx_quality_certs_deleted_at ON quality_certificates(deleted_at);

-- Operation logs (high-volume audit table — most critical)
CREATE INDEX IF NOT EXISTS idx_operation_logs_entity_type ON operation_logs(entity_type);
CREATE INDEX IF NOT EXISTS idx_operation_logs_entity_id ON operation_logs(entity_id);
CREATE INDEX IF NOT EXISTS idx_operation_logs_created_at ON operation_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_operation_logs_user_id ON operation_logs(user_id);