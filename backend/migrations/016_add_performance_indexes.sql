-- 016_add_performance_indexes.sql
-- Composite and covering indexes for common query patterns (generic ERP).
--
-- Strategy:
--   - Composite indexes pair soft-delete (deleted_at) with the most common
--     filter columns so the DB can satisfy "WHERE deleted_at IS NULL AND status=X"
--     from a single index scan instead of a table scan + filter.
--   - JOIN-path indexes speed up detail-view queries that link header ↔ items.
--   - All use CREATE INDEX IF NOT EXISTS for idempotency.
--
-- Pipe-table composite indexes (seamless/screen/welded) and quality-cert
-- indexes were removed together with the dropped tables. Inventory indexes now
-- key on item_id instead of (pipe_type, pipe_id).

-- ============================================================
-- Locations — active location list + zone filtering
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_loc_deleted_active
    ON locations(deleted_at, is_active);

CREATE INDEX IF NOT EXISTS idx_loc_deleted_zone
    ON locations(deleted_at, zone_code);

-- ============================================================
-- Inbound records — list by approval status
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_inb_deleted_status
    ON inbound_records(deleted_at, approval_status);

CREATE INDEX IF NOT EXISTS idx_inb_deleted_type
    ON inbound_records(deleted_at, inbound_type);

-- ============================================================
-- Outbound records — list by approval status
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_outb_deleted_status
    ON outbound_records(deleted_at, approval_status);

CREATE INDEX IF NOT EXISTS idx_outb_deleted_type
    ON outbound_records(deleted_at, outbound_type);

-- ============================================================
-- Purchase orders — list by status + date range
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_po_deleted_status
    ON purchase_orders(deleted_at, status);

CREATE INDEX IF NOT EXISTS idx_po_deleted_supplier
    ON purchase_orders(deleted_at, supplier_id);

-- Covering index for the purchase order list page
CREATE INDEX IF NOT EXISTS idx_po_list_cover
    ON purchase_orders(deleted_at, status, order_no, supplier_id, order_date, total_amount);

-- ============================================================
-- Sales orders — list by status + date range
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_so_deleted_status
    ON sales_orders(deleted_at, status);

CREATE INDEX IF NOT EXISTS idx_so_deleted_customer
    ON sales_orders(deleted_at, customer_id);

CREATE INDEX IF NOT EXISTS idx_so_list_cover
    ON sales_orders(deleted_at, status, order_no, customer_id, order_date, total_amount);

-- ============================================================
-- Contracts — list by status and type
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_ct_deleted_status
    ON contracts(deleted_at, status);

CREATE INDEX IF NOT EXISTS idx_ct_deleted_type
    ON contracts(deleted_at, contract_type);

-- ============================================================
-- Suppliers / Customers — active entity lookups
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_sup_deleted_active
    ON suppliers(deleted_at, is_active);

CREATE INDEX IF NOT EXISTS idx_cus_deleted_active
    ON customers(deleted_at, is_active);

-- ============================================================
-- Operation logs — composite for timeline and entity queries
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_ol_entity_time
    ON operation_logs(entity_type, entity_id, created_at);

CREATE INDEX IF NOT EXISTS idx_ol_user_time
    ON operation_logs(user_id, created_at);

-- ============================================================
-- Inventory logs — item movement history
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_il_item_time
    ON inventory_logs(item_id, created_at);

CREATE INDEX IF NOT EXISTS idx_il_change_type
    ON inventory_logs(change_type, created_at);

-- ============================================================
-- Inventory check records — status filtering
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_ic_deleted_status
    ON inventory_check_records(deleted_at, status);

-- ============================================================
-- Contract payments — active unpaid lookups
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_cp_contract_paid
    ON contract_payments(contract_id, is_paid);

-- ============================================================
-- Refresh tokens — active token cleanup
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_rt_user_expires
    ON refresh_tokens(user_id, expires_at);

-- ============================================================
-- Users — active user lookups by role
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_users_deleted_active
    ON users(deleted_at, is_active);

CREATE INDEX IF NOT EXISTS idx_users_deleted_role
    ON users(deleted_at, role);
