-- 011_add_rejection_reason.sql
-- Adds rejection_reason column to inbound_records and outbound_records tables.
-- Allows recording a reason when an inbound or outbound record is rejected during approval workflow.
-- Column is nullable (TEXT) — only populated when approval_status = 'rejected'.
ALTER TABLE inbound_records ADD COLUMN rejection_reason TEXT;
ALTER TABLE outbound_records ADD COLUMN rejection_reason TEXT;
