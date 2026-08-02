-- 023_auth_rbac_backfill.sql
-- Backfills columns that 022 shipped without: the repo layer (auth/repos.rs)
-- references code/path/sort_order/is_system/is_active, which 022's CREATE
-- TABLE statements omitted. 023 exists so databases that already ran 022 get
-- the same shape as fresh installs (which now include the columns in 022).
--
-- All ALTERs are IF NOT EXISTS so re-running (or running on a fresh DB that
-- already has 022's new shape) is a no-op.

ALTER TABLE tenants
    ADD COLUMN IF NOT EXISTS code VARCHAR(50) UNIQUE;

-- The 022 seed inserted the default tenant without a code; backfill it.
UPDATE tenants SET code = 'DEFAULT' WHERE id = 1 AND code IS NULL;

-- code is NOT NULL in fresh installs; enforce the same invariant here.
ALTER TABLE tenants
    ALTER COLUMN code SET NOT NULL;

ALTER TABLE departments
    ADD COLUMN IF NOT EXISTS path TEXT NOT NULL DEFAULT '/',
    ADD COLUMN IF NOT EXISTS sort_order INT NOT NULL DEFAULT 0;

-- Root departments get their materialized path; children are patched
-- incrementally (single pass is enough since seeds are shallow).
UPDATE departments d
SET path = CASE
    WHEN d.parent_id IS NULL THEN '/' || d.id::text
    ELSE (SELECT p.path FROM departments p WHERE p.id = d.parent_id) || '/' || d.id::text
END
WHERE d.path = '/';

ALTER TABLE roles
    ADD COLUMN IF NOT EXISTS is_system BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE;

-- The 022 seed marked the four built-in roles as system roles.
UPDATE roles SET is_system = TRUE WHERE id IN (1, 2, 3, 4) AND is_system = FALSE;
