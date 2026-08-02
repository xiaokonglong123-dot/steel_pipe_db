# 001 — Auth & 身份管理 (Phase 1)

> **版本**: v1.0
> **日期**: 2026-08-02
> **状态**: Draft
> **依赖**: `000-audit-fix.md` (PostgreSQL migration)
> **父文档**: `015-architecture-overview.md`

---

## 1. 目标

从当前的 4 角色硬编码 JWT 系统升级为多租户、细粒度 RBAC 的身份管理体系。

## 2. 功能范围

| 功能 | 当前 | 升级 |
|------|------|------|
| 用户认证 | JWT (HS256) + Argon2id | RS256 key pair + refresh rotation + token blacklist |
| 角色体系 | `admin/warehouse/qc/sales` 4 个固定角色 | RBAC 多对多 (roles + permissions) |
| 组织架构 | 无 | multi-tenant (Tenants → Companies → Departments → Positions) |
| 密码策略 | 无 | 复杂度规则 + 过期 + 锁定 |
| Token | access token (mem) + refresh (cookie) | TTL + Redis blacklist |
| 安全 | 无 MFA | 可选 TOTP / WebAuthn |

## 3. 数据模型

```sql
-- schema: auth

CREATE TABLE auth.tenants (
    id      BIGSERIAL PRIMARY KEY,
    name    VARCHAR(200) NOT NULL,
    domain  VARCHAR(100) UNIQUE,
    is_active BOOLEAN DEFAULT true,
    config  JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE auth.companies (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL REFERENCES auth.tenants(id),
    name VARCHAR(300) NOT NULL,
    tax_id VARCHAR(50),
    is_active BOOLEAN DEFAULT true,
    created_at, updated_at, deleted_at
);

CREATE TABLE auth.departments (
    id BIGSERIAL PRIMARY KEY,
    company_id BIGINT NOT NULL REFERENCES auth.companies(id),
    parent_id BIGINT REFERENCES auth.departments(id),  -- self-referencing
    name VARCHAR(200) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at, updated_at, deleted_at
);

CREATE TABLE auth.positions (
    id BIGSERIAL PRIMARY KEY,
    company_id BIGINT NOT NULL REFERENCES auth.companies(id),
    name VARCHAR(200) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at, updated_at, deleted_at
);

-- 升级后的 users 表
CREATE TABLE auth.users (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL REFERENCES auth.tenants(id),
    username VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(200),
    email VARCHAR(200),
    phone VARCHAR(30),
    is_active BOOLEAN DEFAULT true,
    password_changed_at TIMESTAMPTZ,
    locked_until TIMESTAMPTZ,
    login_failures INT DEFAULT 0,
    totp_enabled BOOLEAN DEFAULT false,
    totp_secret VARCHAR(32),
    language_pref VARCHAR(5) DEFAULT 'zh',
    unit_system VARCHAR(10) DEFAULT 'metric',
    created_at, updated_at, deleted_at
);

-- RBAC (多对多)
CREATE TABLE auth.roles (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    created_at, updated_at
);

CREATE TABLE auth.permissions (
    id BIGSERIAL PRIMARY KEY,
    key   VARCHAR(200) UNIQUE NOT NULL,   -- e.g. 'purchase.order.write'
    description TEXT
);

INSERT INTO auth.permissions (key, description) VALUES
    ('pipe.read', 'Read pipe data'),
    ('pipe.write', 'Create/Update pipe data'),
    ('inventory.inbound', 'Perform inbound operations'),
    ('inventory.outbound', 'Perform outbound operations'),
    ('purchase.read', 'Read purchase orders'),
    ('purchase.approve', 'Approve purchase orders'),
    ('sales.read', 'Read sales orders'),
    ('sales.approve', 'Approve sales orders'),
    ('finance.read', 'Read financial data'),
    ('finance.journal.post', 'Create journal entries'),
    ('finance.pay', 'Confirm payments'),
    ('hr.read', 'Read employee data'),
    ('hr.employee.write', 'Edit employee data'),
    ('manufacturing.bom', 'Manage BOMs'),
    ('manufacturing.work_order', 'Manage 工指令'),
    ('workflow.design', 'Design workflows'),
    ('system.admin', 'System administration');

CREATE TABLE auth.role_permissions (
    role_id BIGINT REFERENCES auth.roles(id),
    permission_id BIGINT REFERENCES auth.permissions(id),
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE auth.user_roles (
    user_id BIGINT REFERENCES auth.users(id),
    role_id BIGINT REFERENCES auth.roles(id),
    PRIMARY KEY (user_id, role_id)
);

-- Person record (hirable entity, linked to user)
CREATE TABLE auth.persons (
    id BIGSERIAL PRIMARY KEY,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    email VARCHAR(200),
    phone VARCHAR(50),
    user_id BIGINT REFERENCES auth.users(id),
    created_at, updated_at
);

-- Refresh token
CREATE TABLE auth.refresh_tokens (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES auth.users(id),
    token_hash VARCHAR(255),
    expires_at TIMESTAMPTZ,
    revoked BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Identity Event Log
CREATE TABLE auth.identity_events (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT,
    event_type VARCHAR(50),  -- login, logout, lock, unlock, pass_change
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

## 4. API 端点

| Method | Path | Description | Permission |
|--------|------|-------------|-----------|
| POST | `/auth/login` | Login (JWT + cookie) | public |
| POST | `/auth/logout` | Revoke refresh | any auth |
| POST | `/auth/refresh` | New token | cookie |
| GET  | `/auth/me` | Current user | any auth |
| PUT  | `/auth/me` | Update profile | any auth |
| GET  | `/api/admin/users` | List users | admin |
| POST | `/api/admin/users` | Create user | admin |
| GET  | `/api/admin/users/:id` | User detail | admin |
| PUT  | `/api/admin/users/:id` | Update user | admin |
| POST | `/api/admin/users/:id/lock` | Lock user | admin |
| POST | `/api/admin/users/:id/unlock` | Unlock user | admin |
| GET  | `/api/admin/roles` | List roles (per tenant) | admin |
| POST | `/api/admin/roles` | Create role | admin |
| GET  | `/api/admin/permissions` | List permissions | admin |
| POST | `/api/admin/tenants` | Tenant CRUD | superadmin |
| GET  | `/api/admin/tenants/:id/companies` | Company tree | admin |

## 5. 前后端模式

**后端** (跟随现有 handler/service/repo 模式):
```
auth crates:
  handlers/auth.rs        → login, refresh, logout, me, password change
  handlers/roles.rs       → RBAC CRUD
  handlers/tenants.rs     → tenant management
  services/auth_service.rs      → 认证逻辑
  services/identity_service.rs  → 用户/角色/权限
  repositories/
    ├── user_repo.rs
    ├── role_repo.rs
    ├── tenant_repo.rs
    └── person_repo.rs
  models/                  → Pg structs
  dto/                     → Requests/Responses
  events/                   → Event publishers (连接其他模块)
```

**前端**:
- `features/auth/` → 增加 `PersonnelPage` (人员管理), `RoleManagementPage`
- 路由: `settings/users` (已有 UserManagementPage), `settings/roles`, `settings/tenants`
- `Can` 组件控制按钮可见性

## 6. Token 处理

```
Login:
  1. Verify password (Argon2id)
  2. Generate access token (HS256 → RS256 eventually)
  3. Generate refresh token: SHA-256 hash → store => cookie
  4. Response: { success: true, data: { token, user } }

Refresh:
  1. Deserialize refresh cookie
  2. Lookup hash, check expired, not revoked
  3. Generate new refresh, revoke old
  4. 旧的 refresh 放到 Redis 黑名单 (1h TTL)

Logout:
  1. Revoke all user refresh tokens
  2. Return 204
```

## 7. 中间件

```rust
// JWT auth middleware (upgraded)
pub async fn auth_middleware(mut req: Request, next: Next) -> Response {
    // 1. extract Bearer token
    // 2. check Redis blacklist (via cache)
    // 3. decode JWT → AuthContext { user_id, tenant_id, username, permissions }
    // 4. Attach to req.extensions_mut()
    // 5. next.run(req).await
}
// RBAC middleware
pub async fn rbac_middleware(mut req: Request, next: Next, perm: &'static str) -> Response {
    let ctx = req.extensions().get::<AuthContext>();
    if ctx.permissions.contains(perm) { next.run(req).await } else { 403 }
}
```