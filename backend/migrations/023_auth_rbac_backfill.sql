-- 023_auth_rbac_backfill.sql
-- The columns this migration used to backfill (tenants.code, departments.path
-- / sort_order, roles.is_system / is_active) are now defined directly in
-- 022_create_auth_rbac.sql, so fresh SQLite installs already have the full
-- shape. The remaining statements backfill the row-level defaults that depend
-- on seed data.
--
-- SQLite port: `d.id::text` -> `d.id` (|| concatenation converts the integer
-- to text automatically); UPDATE table aliases are not supported by SQLite, so
-- the outer row is referenced by table name instead.

-- The 022 seed inserted the default tenant with code 'DEFAULT' already; keep
-- the backfill for databases that predate the column default.
UPDATE tenants SET code = 'DEFAULT' WHERE id = 1 AND code IS NULL;

-- Root departments get their materialized path; children are patched
-- incrementally (single pass is enough since seeds are shallow).
UPDATE departments
SET path = CASE
    WHEN parent_id IS NULL THEN '/' || id
    ELSE (SELECT p.path FROM departments p WHERE p.id = departments.parent_id) || '/' || id
END
WHERE path = '/';

-- The 022 seed marked the four built-in roles as system roles.
UPDATE roles SET is_system = 1 WHERE id IN (1, 2, 3, 4) AND is_system = 0;
