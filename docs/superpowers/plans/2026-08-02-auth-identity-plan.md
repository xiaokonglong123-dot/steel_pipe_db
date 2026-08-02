# Auth & 身份管理 Implementation Plan (v2 — PG 迁移后修订)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use `- [ ]` syntax.

**Goal:** Upgrade from 4 hardcoded roles to multi-tenant RBAC with JWT, role/permission tables, tenant/department tree, and DB-backed token blacklist.

**Architecture (REVISED 2026-08-02 after PG migration):**
- **Single crate, modular directory** (user decision): new code lives under `backend/src/auth/` (handlers/services/repos submodules), NOT a separate `backend/crates/auth/` crate as the original spec assumed. Existing auth_handler.rs / auth_service.rs / user_repo.rs / refresh_token_repo.rs stay in place and get extended.
- **DB-backed blacklist** (user decision): token revocation via existing `refresh_tokens.revoked_at`, NO Redis dependency.
- **Tenant + Department two-level org tree** (user decision): `tenants` (one default tenant) + `departments` (self-referencing parent_id). companies/positions tables deferred.
- **RBAC tables in `public` schema** (no `auth.` prefix), matching the 20 existing migrations. users table upgraded in place via ALTER (tenant_id, password_changed_at, locked_until, login_failures) — NOT replaced by a new auth.users table.
- **role column kept for compatibility**: existing `require_role` middleware and frontend keep working during transition; new permission checks layered on top.

**Tech Stack:** Rust (Axum 0.8, jsonwebtoken 9, sqlx pg, argon2 0.5), React 19, Zustand, TanStack Query

## Global Constraints
- Follow existing handler/service/repo patterns
- Error codes: 110xx (auth)
- No hard delete — all tables have deleted_at
- No `as any`, `@ts-ignore`
- Verification: cargo check + cargo test; npx tsc --noEmit + npm run build

---

### Task 1: RBAC migration (022_create_auth_rbac.sql)

**Files:**
- Create: `backend/migrations/022_create_auth_rbac.sql`

**Tables (public schema):**
```sql
CREATE TABLE tenants (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    domain VARCHAR(100) UNIQUE,
    is_active BOOLEAN DEFAULT true,
    config JSONB,
    created_at/updated_at/deleted_at TIMESTAMPTZ
);
CREATE TABLE departments (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL REFERENCES tenants(id),
    parent_id BIGINT REFERENCES departments(id),
    name VARCHAR(200) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at/updated_at/deleted_at
);
CREATE TABLE roles (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    created_at/updated_at/deleted_at
);
CREATE TABLE permissions (
    id BIGSERIAL PRIMARY KEY,
    key VARCHAR(200) UNIQUE NOT NULL,
    description TEXT
);
CREATE TABLE role_permissions (
    role_id BIGINT REFERENCES roles(id),
    permission_id BIGINT REFERENCES permissions(id),
    PRIMARY KEY (role_id, permission_id)
);
CREATE TABLE user_roles (
    user_id BIGINT REFERENCES users(id),
    role_id BIGINT REFERENCES roles(id),
    PRIMARY KEY (user_id, role_id)
);
ALTER TABLE users ADD COLUMN tenant_id BIGINT DEFAULT 1, password_changed_at, locked_until, login_failures INT DEFAULT 0;
```

**Seed:** default tenant (id=1), permissions list from spec (pipe.read … system.admin), four default roles (admin/warehouse/qc/sales) with role_permissions mappings.

- [ ] Migration file with all tables + ALTER + seed
- [ ] Verify: migrations run clean against local PG (scripts/pg-dev.sh)

---

### Task 2: auth module skeleton + repos

**Files:**
- Create: `backend/src/auth/mod.rs`
- Create: `backend/src/auth/repositories/tenant_repo.rs` (tenant CRUD, department tree)
- Create: `backend/src/auth/repositories/role_repo.rs` (roles, role_permissions, user_roles)
- Create: `backend/src/auth/repositories/permission_repo.rs` (permission listing)
- Edit: `backend/src/lib.rs` (add `pub mod auth;`)
- Edit: `backend/src/repositories/user_repo.rs` (add tenant_id-aware list/filter)

Pattern: `SELECT * FROM users WHERE deleted_at IS NULL AND tenant_id = $1` etc.

- [ ] All repos compile (cargo check)
- [ ] Repos tested via cargo test (new auth integration test file)

---

### Task 3: Service layer — identity_service

**Files:**
- Create: `backend/src/auth/services/identity_service.rs`
- Edit: `backend/src/services/auth_service.rs` (login fetches permissions; refresh rotation revokes old token; lock/unlock)

**Functions:**
- `login(username, password)` → verify argon2, fetch permissions from DB via user_roles→roles→permissions, generate JWT with claims { sub, tenant_id, permissions[] }
- `refresh_token(claims)` → rotate refresh (revoke old, issue new)
- `list_roles(tenant_id)`, `create_role(...)`, `assign_permission(...)`, `assign_user_role(...)`
- `lock_user(user_id)`, `unlock_user(user_id)`
- `list_departments(tenant_id)`, `create_department(...)`

- [ ] Login flow returns permissions in JWT
- [ ] Locked user cannot login (locked_until check)
- [ ] Role/permission assignment works
- [ ] Tests: new identity_service tests

---

### Task 4: Handlers + routes

**Files:**
- Create: `backend/src/auth/handlers.rs` (roles, tenants, departments, permissions endpoints)
- Edit: `backend/src/handlers/auth_handler.rs` (me endpoint returns permissions)
- Edit: `backend/src/router.rs` (register new routes)

**Endpoints (all under /api/v1):**
| Method | Path | Description | Permission |
|--------|------|-------------|-----------|
| GET | /admin/roles | List roles | system.admin |
| POST | /admin/roles | Create role | system.admin |
| PUT | /admin/roles/:id | Update role | system.admin |
| DELETE | /admin/roles/:id | Soft-delete role | system.admin |
| GET | /admin/permissions | List permissions | system.admin |
| POST | /admin/roles/:id/permissions | Assign permission | system.admin |
| POST | /admin/users/:id/roles | Assign user role | system.admin |
| GET | /admin/tenants | List tenants | system.admin |
| POST | /admin/tenants | Create tenant | system.admin |
| GET | /admin/departments | List dept tree | system.admin |
| POST | /admin/departments | Create dept | system.admin |
| POST | /admin/users/:id/lock | Lock user | system.admin |
| POST | /admin/users/:id/unlock | Unlock user | system.admin |

- [ ] Routes registered, cargo check clean
- [ ] curl smoke: login → list roles

---

### Task 5: JWT middleware upgrade

**Files:**
- Edit: `backend/src/middleware/auth.rs` (AuthContext gains permissions; JWT decode extracts claims.permissions)
- Edit: `backend/src/middleware/rbac.rs` (new `require_permission(perm: &'static str)` middleware)

- [ ] AuthContext { user_id, tenant_id, username, role, permissions }
- [ ] require_permission returns 403 when permission missing
- [ ] Existing require_role still works (compat)
- [ ] Integration test: endpoint with require_permission

---

### Task 6: Frontend — Can component + role management pages

**Files:**
- Create: `frontend/src/shared/components/Can.tsx` (permission-based rendering)
- Create: `frontend/src/features/auth/api/roleApi.ts` + hooks
- Create: `frontend/src/features/auth/pages/RoleManagementPage.tsx`
- Create: `frontend/src/features/auth/pages/TenantPage.tsx` (tenant + department tree)
- Edit: `frontend/src/features/auth/api/authApi.ts` (login response includes permissions)
- Edit: `frontend/src/stores/authStore.ts` (store permissions)
- Edit: `frontend/src/routes/index.tsx` (add routes under /settings/roles, /settings/tenants)
- Edit: `frontend/src/layouts/MainLayout.tsx` (menu items for new pages)

- [ ] Can component renders children only when permission granted
- [ ] RoleManagementPage lists/creates roles, assigns permissions
- [ ] tsc --noEmit clean
- [ ] npm run build clean

---

### Task 7: Verify full flow

- [ ] cargo check + cargo test (all 12 test files green)
- [ ] Backend smoke: login admin → JWT has permissions; create role; assign permission; verify 403 on missing permission
- [ ] Frontend build clean
- [ ] Update AGENTS.md (RBAC tables, new routes, permission keys)

---
