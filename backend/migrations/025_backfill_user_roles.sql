-- 025_backfill_user_roles.sql
-- Backfills user_roles for users that have a legacy `role` column value but
-- no role binding (the 022 seed only matched users that existed when it ran,
-- so installs where the admin was bootstrapped AFTER migrations have none).
-- ON CONFLICT makes this idempotent for databases that already have bindings.
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id FROM users u JOIN roles r
  ON r.name = CASE u.role WHEN 'admin' THEN 'admin'
                          WHEN 'warehouse' THEN 'warehouse'
                          WHEN 'qc' THEN 'qc'
                          WHEN 'sales' THEN 'sales' END
WHERE u.deleted_at IS NULL
ON CONFLICT DO NOTHING;
