-- 013_add_approval_reason.sql
-- Add approval_reason column to inbound/outbound records for audit trail.
-- Records who approved/rejected and why.
-- SQLite supports ALTER TABLE ADD COLUMN directly.
ALTER TABLE inbound_records ADD COLUMN approval_reason TEXT;
ALTER TABLE outbound_records ADD COLUMN approval_reason TEXT;
