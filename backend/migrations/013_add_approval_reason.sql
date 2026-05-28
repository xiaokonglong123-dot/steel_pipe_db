-- 013_add_approval_reason.sql
-- Add approval_reason column to inbound/outbound records for audit trail.
-- Records who approved/rejected and why.

ALTER TABLE inbound_records ADD COLUMN approval_reason TEXT;
ALTER TABLE outbound_records ADD COLUMN approval_reason TEXT;
