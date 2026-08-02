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
-- All tables live in the public schema, matching the other 21 migrations.
-- Companies/positions are deferred (user decision: two-level org tree only).

-- ---------------------------------------------------------------------------
-- 1. Org tree: tenants + departments
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS tenants (
    id         BIGSERIAL PRIMARY KEY,
    name       VARCHAR(200) NOT NULL,
    domain     VARCHAR(100) UNIQUE,
    is_active  BOOLEAN NOT NULL DEFAULT TRUE,
    config     JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_tenants_domain ON tenants(domain);

CREATE TABLE IF NOT EXISTS departments (
    id         BIGSERIAL PRIMARY KEY,
    tenant_id  BIGINT NOT NULL REFERENCES tenants(id),
    parent_id  BIGINT REFERENCES departments(id),  -- self-referencing tree
    name       VARCHAR(200) NOT NULL,
    is_active  BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_departments_tenant ON departments(tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_departments_parent ON departments(parent_id) WHERE deleted_at IS NULL;

-- ---------------------------------------------------------------------------
-- 2. RBAC: roles, permissions, mappings
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS roles (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   BIGINT NOT NULL REFERENCES tenants(id),
    name        VARCHAR(100) NOT NULL,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

-- Role names must be unique per tenant (soft-delete aware via partial index).
CREATE UNIQUE INDEX uni_roles_tenant_name ON roles (tenant_id, name) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS permissions (
    id          BIGSERIAL PRIMARY KEY,
    key         VARCHAR(200) UNIQUE NOT NULL,   -- e.g. 'purchase.order.write'
    description TEXT
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id       BIGINT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id BIGINT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id BIGINT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX idx_user_roles_user ON user_roles(user_id);

-- ---------------------------------------------------------------------------
-- 3. Seed: default tenant, permission dictionary, default roles
-- (seeded BEFORE the users ALTER so the FK backfill in the ALTER resolves)
-- ---------------------------------------------------------------------------

-- Default tenant (id = 1). All existing users are scoped to it via the
-- ALTER DEFAULT below; new tenants are created by system admins later.
INSERT INTO tenants (id, name, domain, config)
VALUES (1, '默认组织', 'default', '{"default": true}')
ON CONFLICT (id) DO NOTHING;

INSERT INTO permissions (key, description) VALUES
    ('pipe.read',                '查看钢管数据'),
    ('pipe.write',               '新增/修改钢管数据'),
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
    ('manufacturing.work_order', '管理工指令'),
    ('workflow.design',          '设计审批流程'),
    ('system.admin',             '系统管理')
ON CONFLICT (key) DO NOTHING;

-- Default roles mirror the legacy 4-role system so existing behavior is
-- preserved through user_roles once the transition completes.
INSERT INTO roles (id, tenant_id, name, description) VALUES
    (1, 1, 'admin',     '系统管理员：全部权限'),
    (2, 1, 'warehouse', '仓库管理员：钢管与出入库'),
    (3, 1, 'qc',        '质检员：质检与质量证书'),
    (4, 1, 'sales',     '销售：订单与客户')
ON CONFLICT (id) DO NOTHING;

-- admin → everything
INSERT INTO role_permissions (role_id, permission_id)
SELECT 1, id FROM permissions
ON CONFLICT DO NOTHING;

-- warehouse → pipes + inventory
INSERT INTO role_permissions (role_id, permission_id)
SELECT 2, id FROM permissions WHERE key IN
    ('pipe.read', 'pipe.write', 'inventory.inbound', 'inventory.outbound', 'inventory.check')
ON CONFLICT DO NOTHING;

-- qc → pipes (read) + quality
INSERT INTO role_permissions (role_id, permission_id)
SELECT 3, id FROM permissions WHERE key IN
    ('pipe.read', 'quality.read', 'quality.write', 'inventory.check')
ON CONFLICT DO NOTHING;

-- sales → pipes (read) + orders + customers
INSERT INTO role_permissions (role_id, permission_id)
SELECT 4, id FROM permissions WHERE key IN
    ('pipe.read', 'purchase.read', 'sales.read', 'sales.approve')
ON CONFLICT DO NOTHING;

-- Legacy `role` column values are mapped to role ids for existing users, so
-- old users immediately get their permissions through user_roles.
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id FROM users u JOIN roles r
  ON r.name = CASE u.role WHEN 'admin' THEN 'admin'
                          WHEN 'warehouse' THEN 'warehouse'
                          WHEN 'qc' THEN 'qc'
                          WHEN 'sales' THEN 'sales' END
WHERE u.deleted_at IS NULL
ON CONFLICT DO NOTHING;

-- ---------------------------------------------------------------------------
-- 4. users upgrade: tenant scoping + account-security columns
-- (runs LAST: PG backfills existing rows with the DEFAULT and validates the
-- FK immediately, so the seed tenant must already exist)
-- ---------------------------------------------------------------------------

ALTER TABLE users
    ADD COLUMN tenant_id BIGINT NOT NULL DEFAULT 1 REFERENCES tenants(id),
    ADD COLUMN password_changed_at TIMESTAMPTZ,
    ADD COLUMN locked_until TIMESTAMPTZ,
    ADD COLUMN login_failures INT NOT NULL DEFAULT 0;

CREATE INDEX idx_users_tenant ON users(tenant_id) WHERE deleted_at IS NULL;
