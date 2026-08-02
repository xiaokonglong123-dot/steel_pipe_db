-- 024_sync_serial_sequences.sql
-- 022 seeded tenants/roles with explicit ids, which does not advance the
-- BIGSERIAL sequences — a subsequent INSERT would collide with id 1.
SELECT setval(pg_get_serial_sequence('tenants', 'id'), GREATEST((SELECT COALESCE(MAX(id), 1) FROM tenants), 1));
SELECT setval(pg_get_serial_sequence('roles', 'id'), GREATEST((SELECT COALESCE(MAX(id), 1) FROM roles), 1));
