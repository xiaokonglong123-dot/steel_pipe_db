-- Add rejection_reason column to inbound_records and outbound_records
ALTER TABLE inbound_records ADD COLUMN rejection_reason TEXT;
ALTER TABLE outbound_records ADD COLUMN rejection_reason TEXT;
