# Auth & 身份管理 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use `- [ ]` syntax.

**Goal:** Upgrade from 4 hardcoded roles to multi-tenant RBAC with RS256 JWT, role/permission tables, company/department trees, and token blacklist.

**Architecture:** New `auth` crate under `backend/crates/auth/` following handler/service/repo layers. All tables live in new PostgreSQL schema `auth`. Token flow upgraded: RBAC permissions extracted from DB at login, refresh token rotated on each use, old refresh tokens blacklisted in Redis. Nuanced: frontend adds Can component, RoleManagementPage, TenantPage, person page.

**Tech Stack:** Rust (Axum 0.8, jsonwebtoken 9, sqlx pg, argon2 0.5), React 19, Zustand, TanStack Query

## Global Constraints
- Follow existing handler/service/repo patterns
- Error codes: 110xx (auth)
- No hard delete — all tables have deleted_at
- No `as any`, `@ts-ignore`
- Verification: cargo check + cargo test; npx tsc --noEmit + npm run build

---

### Task 1: Create auth crate skeleton

**Files:**
- Create: `backend/crates/auth/Cargo.toml`
- Create: `backend/crates/auth/src/mod.rs`, `lib.rs`, `routes.rs`

- [ ] Create Cargo.toml with deps on core, jsonwebtoken, argon2
- [ ] Create modules: handlers, services, repos, models, dto, events

---

### Task 2: Create DB migration tables for RBAC

**Files:**
- Create: `backend/migrations/023_create_auth_rbac.sql`

```sql
CREATE SCHEMA IF NOT EXISTS auth;
CREATE TABLE auth.tenants (id BIGSERIAL PRIMARY KEY, name TEXT NOT NULL, ...);
CREATE TABLE auth.roles (id BIGSERIAL PRIMARY KEY, name TEXT NOT NULL, tenant_id BIGINT);
CREATE TABLE auth.permissions (id BIGSERIAL PRIMARY KEY, key TEXT UNIQUE NOT NULL);
CREATE TABLE auth.role_permissions (role_id BIGINT REFERENCES auth.roles(id), permission_id BIGINT REFERENCES auth.permissions(id));
CREATE TABLE auth.tenant_members (user_id BIGINT, tenant_id BIGINT, role_id BIGINT);
```

---

### Task 3: Service layer — auth_service + identity_service

**Files:**
- Create: `backend/crates/auth/src/services/auth_service.rs`
- Create: `backend/crates/auth/src/services/identity_service.rs`

- [ ] `login(username, password)` → verify argon2, fetch permissions from DB, generate JWT with claims { sub, tenant_id, permissions[] }
- [ ] `refresh_token(claims)` → rotate refresh
- [ ] `list_roles(tenant_id)`, `create_role(...)`, `assign_permission(...)`, `assign_user_role(...)`

---

### Task 4: Repositories layer

**Files:**
- Create: `backend/crates/auth/src/repositories/tenant_repo.rs`
- Create: `backend/crates/auth/src/repositories/user_repo.rs`
- Create: `backend/crates/auth/src/repositories/role_repo.rs`

Pattern: `SELECT * FROM auth.users WHERE deleted_at IS NULL AND username = ?` etc.

---

### Task 5: Handlers — auth + roles

List of endpoints to create: login, logout, refresh, me + all CRUD routes for roles, tenants, users, permissions.

---

### Task 6: JWT middleware upgrade

- Read custom JWT middleware → update to parse permissions from claims
- Create `PermissionMiddleware` checking `req.extensions()->has_perm("...")`

### Task 7: Frontend — Per-route auth control

- Create `frontend/src/shared/components/Can.tsx`
- Create `frontend/src/features/auth/api/roleApi.ts` + hooks + pages

### Task 8: Verify full flow

- login → check permissions → apply Can component → test RBAC

---