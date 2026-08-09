# 001 — Auth & 身份管理 (Phase 1)

> **版本**: v1.0
> **日期**: 2026-08-02
> **状态**: Draft
> **依赖**: `000-audit-fix.md` (SQLite 迁移策略)
> **父文档**: `015-architecture-overview.md`

---

## 1. 目标

从当前的 4 角色硬编码 JWT 系统升级为多租户、细粒度 RBAC 的身份管理体系。

## 2. 功能范围

| 功能 | 当前 | 升级 |
| ------ | ------ | ------ |
| 用户认证 | JWT (HS256) + Argon2id | RS256 key pair + refresh rotation + token blacklist (DB 存储) |
| 角色体系 | `admin/warehouse/qc/sales` 4 个固定角色 | RBAC 多对多 (roles + permissions) |
| 组织架构 | 无 | multi-tenant (Tenants → Departments) |
| 密码策略 | 无 | 复杂度规则 + 过期 + 锁定 |
| Token | access token (mem) + refresh (cookie) | TTL + DB 黑名单 |
| 安全 | 无 MFA | 可选 TOTP / WebAuthn |

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`），所有表为普通 SQLite 表（无 schema 前缀），类型用 `INTEGER`/`TEXT`/`REAL`。

## 3. 数据模型

```sql
-- 全部表位于 SQLite 单文件数据库中（无 schema 隔离前缀）

CREATE TABLE tenants (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    name    TEXT NOT NULL,
    domain  TEXT UNIQUE,
    is_active INTEGER DEFAULT 1,
    config  TEXT,                -- JSON 文本
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    deleted_at TEXT
);

CREATE TABLE departments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id INTEGER NOT NULL REFERENCES tenants(id),
    parent_id INTEGER REFERENCES departments(id),  -- self-referencing
    name TEXT NOT NULL,
    is_active INTEGER DEFAULT 1,
    created_at, updated_at, deleted_at
);

-- 升级后的 users 表
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id INTEGER NOT NULL REFERENCES tenants(id),
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    display_name TEXT,
    email TEXT,
    phone TEXT,
    is_active INTEGER DEFAULT 1,
    password_changed_at TEXT,
    locked_until TEXT,
    login_failures INTEGER DEFAULT 0,
    totp_enabled INTEGER DEFAULT 0,
    totp_secret TEXT,
    language_pref TEXT DEFAULT 'zh',
    unit_system TEXT DEFAULT 'metric',
    created_at, updated_at, deleted_at
);

-- RBAC (多对多)
CREATE TABLE roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at, updated_at
);

CREATE TABLE permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key   TEXT UNIQUE NOT NULL,   -- e.g. 'purchase.order.write'
    description TEXT
);

INSERT INTO permissions (key, description) VALUES
    ('item.read', 'Read item data'),
    ('item.write', 'Create/Update item data'),
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
    ('manufacturing.work_order', 'Manage 工单'),
    ('workflow.design', 'Design workflows'),
    ('system.admin', 'System administration');

CREATE TABLE role_permissions (
    role_id INTEGER REFERENCES roles(id),
    permission_id INTEGER REFERENCES permissions(id),
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE user_roles (
    user_id INTEGER REFERENCES users(id),
    role_id INTEGER REFERENCES roles(id),
    PRIMARY KEY (user_id, role_id)
);

-- Refresh token
CREATE TABLE refresh_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER REFERENCES users(id),
    token_hash TEXT,
    expires_at TEXT,
    revoked INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Identity Event Log
CREATE TABLE identity_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    event_type TEXT,   -- login, logout, lock, unlock, pass_change
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);
```

## 4. API 端点

| Method | Path | Description | Permission |
| -------- | ------ | ------------- | ----------- |
| POST | `/auth/login` | Login (JWT + cookie) | public |
| POST | `/auth/logout` | Revoke refresh | any auth |
| POST | `/auth/refresh` | New token | cookie |
| GET | `/auth/me` | Current user | any auth |
| PUT | `/auth/me` | Update profile | any auth |
| GET | `/api/admin/users` | List users | admin |
| POST | `/api/admin/users` | Create user | admin |
| GET | `/api/admin/users/:id` | User detail | admin |
| PUT | `/api/admin/users/:id` | Update user | admin |
| POST | `/api/admin/users/:id/lock` | Lock user | admin |
| POST | `/api/admin/users/:id/unlock` | Unlock user | admin |
| GET | `/api/admin/roles` | List roles (per tenant) | admin |
| POST | `/api/admin/roles` | Create role | admin |
| GET | `/api/admin/permissions` | List permissions | admin |
| POST | `/api/admin/tenants` | Tenant CRUD | superadmin |
| GET | `/api/admin/tenants/:id/departments` | Department tree | admin |

## 5. 前后端模式

**后端** (跟随现有 handler/service/repo 模式):

```
backend/src/auth:
  handlers.rs        → login, refresh, logout, me, password change
  handlers.rs        → RBAC CRUD (roles/permissions/departments/tenants)
  services/identity_service.rs  → 用户/角色/权限
  repositories/
    ├── user_repo.rs
    ├── role_repo.rs
    ├── tenant_repo.rs
    └── refresh_token_repo.rs
  models/            → SQLite row structs (sqlx::FromRow)
  dto/               → Requests/Responses
```

**前端**:

- `features/auth/` → 增加 `RoleManagementPage`, `TenantPage`
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
  4. 旧的 refresh 标记 revoked（DB 黑名单，无需 Redis）

Logout:
  1. Revoke all user refresh tokens
  2. Return 204
```

## 7. 中间件

```rust
// JWT auth middleware (upgraded)
pub async fn auth_middleware(mut req: Request, next: Next) -> Response {
    // 1. extract Bearer token
    // 2. check DB blacklist (via refresh_tokens.revoked)
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
