-- 016_add_performance_indexes.sql
-- Composite and covering indexes for common query patterns.
--
-- Strategy:
--   - Composite indexes pair soft-delete (deleted_at) with the most common
--     filter columns so the DB can satisfy "WHERE deleted_at IS NULL AND status=X"
--     from a single index scan instead of a table scan + filter.
--   - Covering indexes include extra columns (pipe_type, location_id) so common
--     list queries can be answered entirely from the index without touching the
--     table rows (index-only scan).
--   - JOIN-path indexes speed up detail-view queries that link header ↔ items.
--   - All use CREATE INDEX IF NOT EXISTS for idempotency.

-- ============================================================
-- Seamless pipes — composite indexes for list/filter patterns
-- ============================================================

-- Status list: "list all in_stock seamless pipes" (most common page view)
CREATE INDEX IF NOT EXISTS idx_sp_deleted_status
    ON seamless_pipes(deleted_at, status);

-- Type filter: "list all casing" or "list all tubing"
CREATE INDEX IF NOT EXISTS idx_sp_deleted_type
    ON seamless_pipes(deleted_at, pipe_type);

-- Location lookup: "which pipes are at location X"
CREATE INDEX IF NOT EXISTS idx_sp_deleted_location
    ON seamless_pipes(deleted_at, location_id);

-- Grade + status: "list all J55 in_stock" (dashboard + filtering)
CREATE INDEX IF NOT EXISTS idx_sp_deleted_grade_status
    ON seamless_pipes(deleted_at, grade, status);

-- Covering index for the main list page (avoids table lookup for common columns)
CREATE INDEX IF NOT EXISTS idx_sp_list_cover
    ON seamless_pipes(deleted_at, status, pipe_number, grade, od, wt, location_id, pipe_type);

-- ============================================================
-- Screen pipes — same patterns as seamless pipes
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_scrp_deleted_status
    ON screen_pipes(deleted_at, status);

CREATE INDEX IF NOT EXISTS idx_scrp_deleted_type
    ON screen_pipes(deleted_at, screen_type);

CREATE INDEX IF NOT EXISTS idx_scrp_deleted_location
    ON screen_pipes(deleted_at, location_id);

CREATE INDEX IF NOT EXISTS idx_scrp_deleted_grade_status
    ON screen_pipes(deleted_at, base_grade, status);

CREATE INDEX IF NOT EXISTS idx_scrp_list_cover
    ON screen_pipes(deleted_at, status, pipe_number, base_grade, base_od, base_wt, location_id, screen_type);

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
-- Quality certs — list by result + pipe reference
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_qc_deleted_result
    ON quality_certs(deleted_at, result);

CREATE INDEX IF NOT EXISTS idx_qc_deleted_pipe
    ON quality_certs(deleted_at, pipe_type, pipe_id);

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
-- Inventory logs — pipe movement history
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_il_pipe_time
    ON inventory_logs(pipe_type, pipe_id, created_at);

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
