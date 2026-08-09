-- 022_create_auth_rbac.sql
-- RBAC (role-based access control) + multi-tenant identity layer.
--
-- Replaces the hardcoded 4-role design (users.role CHECK in 001) with a
-- normalized RBAC model: tenants -> departments (org tree), roles <-> users
-- and roles <-> permissions (both many-to-many), plus a global permission
-- dictionary. `users.role` is KEPT for backward compatibility with the
-- existing `require_role` middleware during the transition; new permission
-- checks layer on top via user_roles.
--
-- SQLite port notes:
--   BIGSERIAL            -> INTEGER PRIMARY KEY AUTOINCREMENT
--   TIMESTAMPTZ          -> TEXT, NOW() -> datetime('now')
--   JSONB                -> TEXT (stored as JSON text)
--   BOOLEAN              -> INTEGER (1/0)
--   VARCHAR(n)           -> TEXT
--   `public schema`      -> SQLite has no schemas
--   INSERT ... SELECT ... ON CONFLICT DO NOTHING -> INSERT OR IGNORE
--     (SQLite's upsert clause does not support the INSERT...SELECT form)
--   ADD COLUMN ... REFERENCES -> NOT allowed by SQLite when foreign_keys=ON,
--     so users.tenant_id is added without an inline REFERENCES clause (the
--     relationship is documented in a comment; other tables keep REFERENCES).
--   ALTER TABLE adds one column at a time (SQLite does not support the
--     PostgreSQL multi-column ADD COLUMN form).
--   Columns added later in 023 (code/path/sort_order/is_system/is_active) are
--     defined here directly so fresh installs get the full shape.

-- ---------------------------------------------------------------------------
-- 1. Org tree: tenants + departments
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS tenants (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    domain     TEXT UNIQUE,
    code       TEXT NOT NULL DEFAULT 'DEFAULT',
    is_active  INTEGER NOT NULL DEFAULT 1,
    config     TEXT,                          -- JSONB -> TEXT
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_tenants_domain ON tenants(domain);

CREATE TABLE IF NOT EXISTS departments (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id  INTEGER NOT NULL REFERENCES tenants(id),
    parent_id  INTEGER REFERENCES departments(id),  -- self-referencing tree
    name       TEXT NOT NULL,
    path       TEXT NOT NULL DEFAULT '/',
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active  INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_departments_tenant ON departments(tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_departments_parent ON departments(parent_id) WHERE deleted_at IS NULL;

-- ---------------------------------------------------------------------------
-- 2. RBAC: roles, permissions, mappings
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS roles (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id   INTEGER NOT NULL REFERENCES tenants(id),
    name        TEXT NOT NULL,
    description TEXT,
    is_system   INTEGER NOT NULL DEFAULT 0,
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT
);

-- Role names must be unique per tenant (soft-delete aware via partial index).
CREATE UNIQUE INDEX IF NOT EXISTS uni_roles_tenant_name ON roles (tenant_id, name) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS permissions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    key         TEXT UNIQUE NOT NULL,        -- e.g. 'purchase.order.write'
    description TEXT
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id       INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user ON user_roles(user_id);

-- ---------------------------------------------------------------------------
-- 3. Seed: default tenant, permission dictionary, default roles
-- (seeded BEFORE the users ALTER so the FK backfill in the ALTER resolves)
-- ---------------------------------------------------------------------------

-- Default tenant (id = 1). All existing users are scoped to it via the
-- ALTER DEFAULT below; new tenants are created by system admins later.
INSERT INTO tenants (id, name, domain, code, config)
VALUES (1, '默认组织', 'default', 'DEFAULT', '{"default": true}')
ON CONFLICT (id) DO NOTHING;

INSERT INTO permissions (key, description) VALUES
    ('item.read',                '查看商品数据'),
    ('item.write',               '新增/修改商品数据'),
    ('quality.read',             '查看质检数据'),
    ('quality.write',            '新增/修改质检数据'),
    ('inventory.inbound',        '执行入库操作'),
    ('inventory.outbound',       '执行出库操作'),
    ('inventory.check',          '执行盘点操作'),
    ('purchase.read',            '查看采购订单'),
    ('purchase.approve',         '审批采购订单'),
    ('sales.read',               '查看销售订单'),
    ('sales.approve',            '审批销售订单'),
    ('finance.read',             '查看财务数据'),
    ('finance.journal.post',     '创建记账凭证'),
    ('finance.pay',              '确认付款'),
    ('hr.read',                  '查看员工数据'),
    ('hr.employee.write',        '编辑员工数据'),
    ('manufacturing.bom',        '管理 BOM'),
    ('manufacturing.work_order', '管理工单'),
    ('workflow.design',          '设计审批流程'),
    ('system.admin',             '系统管理')
ON CONFLICT (key) DO NOTHING;

-- Default roles mirror the legacy 4-role system so existing behavior is
-- preserved through user_roles once the transition completes.
INSERT INTO roles (id, tenant_id, name, description) VALUES
    (1, 1, 'admin',     '系统管理员：全部权限'),
    (2, 1, 'warehouse', '仓库管理员：商品与出入库'),
    (3, 1, 'qc',        '质检员：质检与质量记录'),
    (4, 1, 'sales',     '销售：订单与客户')
ON CONFLICT (id) DO NOTHING;

-- admin → everything
INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT 1, id FROM permissions;

-- warehouse → items + inventory
INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT 2, id FROM permissions WHERE key IN
    ('item.read', 'item.write', 'inventory.inbound', 'inventory.outbound', 'inventory.check');

-- qc → items (read) + quality
INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT 3, id FROM permissions WHERE key IN
    ('item.read', 'quality.read', 'quality.write', 'inventory.check');

-- sales → items (read) + orders + customers
INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT 4, id FROM permissions WHERE key IN
    ('item.read', 'purchase.read', 'sales.read', 'sales.approve');

-- Legacy `role` column values are mapped to role ids for existing users, so
-- old users immediately get their permissions through user_roles.
INSERT OR IGNORE INTO user_roles (user_id, role_id)
SELECT u.id, r.id FROM users u JOIN roles r
  ON r.name = CASE u.role WHEN 'admin' THEN 'admin'
                          WHEN 'warehouse' THEN 'warehouse'
                          WHEN 'qc' THEN 'qc'
                          WHEN 'sales' THEN 'sales' END
WHERE u.deleted_at IS NULL;

-- ---------------------------------------------------------------------------
-- 4. users upgrade: tenant scoping + account-security columns
-- (runs LAST: SQLite backfills existing rows with the DEFAULT; the default
-- tenant id 1 is seeded above. An inline REFERENCES clause is omitted because
-- SQLite forbids ADD COLUMN ... REFERENCES when foreign_keys=ON — the FK is
-- enforced logically through user_roles and the app layer.)
-- ---------------------------------------------------------------------------

ALTER TABLE users ADD COLUMN tenant_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE users ADD COLUMN password_changed_at TEXT;
ALTER TABLE users ADD COLUMN locked_until TEXT;
ALTER TABLE users ADD COLUMN login_failures INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id) WHERE deleted_at IS NULL;
