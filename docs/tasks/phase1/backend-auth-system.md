# Phase 1 — Backend: Auth & System Management (P0 MVP)

> Based on: `docs/requirements.en.md` §3.10, §4.4; `docs/detailed-design.en.md` §4.6, §5.3.15-16, §6.4, §10
> Architecture: backend crate `erp-server` (Rust + Axum + SQLx 0.8 **sqlite** feature); DB = SQLite3 at `sqlite://data/erp.db?mode=rwc` (WAL). DI 走 `Extension<SqlitePool>` / `Extension<JwtSecret>`；service 为 unit struct + 静态方法；响应形状 `ApiResponse<T>` / `PaginatedResponse<T>`；错误码数值化（100xx 通用 / 110xx 认证）。术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`。

## Tasks

### 1.1 DB Migration

- [ ] Create `users` table migration (username UNIQUE, password_hash, display_name, role, language_pref, unit_system, is_active, last_login_at)
- [ ] Create `operation_logs` table migration (audit log)

### 1.2 Domain Layer

- [ ] Define `User` struct
- [ ] Define `OperationLog` struct
- [ ] Define DTOs: `LoginRequest`, `LoginResponse` (with access_token, refresh_token, user), `RefreshRequest`, `CreateUserDto`, `UpdateUserDto`
- [ ] Define enums: `Role` (Admin / Warehouse / Qc / Sales)
- [ ] Define JWT Claims struct (sub, role, exp, iat)

### 1.3 Repository Layer

- [ ] Implement `UserRepo`:
  - `find_by_username(username) -> Option<User>`
  - `create(dto) -> User`
  - `update(id, dto) -> User`
  - `find_by_id(id) -> Option<User>`
  - `list(filter) -> PaginatedResult<User>`
- [ ] Implement `OperationLogRepo`:
  - `create(log) -> OperationLog`
  - `list(filter) -> PaginatedResult<OperationLog>`

### 1.4 Service Layer

- [ ] Implement `AuthService`:
  - `login(LoginRequest)`: verify username/password (Argon2) + generate JWT (access_token + refresh_token) + update last_login + write operation log
  - `refresh_token(refresh_token)`: validate refresh token + issue new token pair
  - `logout(user_id)`: log logout action
  - `get_current_user(user_id)`: fetch current user info
- [ ] Implement `UserService`:
  - `create_user(dto)`: hash password + create user
  - `update_user(id, dto)`: update info (password handled separately)
  - `list_users(filter)`: user list
  - `assign_role(user_id, role)`: assign role

### 1.5 Middleware & Handler Layer

- [ ] Implement auth middleware (`AuthMiddleware`):
  - Parse JWT from `Authorization: Bearer <token>` header
  - Validate token (expiry, signature)
  - Inject user info into Request Extension
- [ ] Implement role middleware (`RequireRole`):
  - Extract allowed roles from route config
  - Compare against current user role, return 403 on mismatch
  - Use `axum::middleware::from_extractor` or custom Layer
- [ ] Implement Auth endpoints:
  - `POST /api/v1/auth/login` — login
  - `POST /api/v1/auth/refresh` — refresh token
  - `POST /api/v1/auth/logout` — logout
  - `GET /api/v1/auth/me` — get current user (requires auth)
- [ ] Implement user management endpoints (admin only):
  - `GET /api/v1/users` — list users
  - `POST /api/v1/users` — create user
  - `PUT /api/v1/users/{id}` — update user
  - `PUT /api/v1/users/{id}/role` — assign role
- [ ] Implement operation log endpoints:
  - `GET /api/v1/operation-logs` — query operation logs (admin)
- [ ] Configure CORS middleware (allow frontend cross-origin requests)

### 1.6 Security Stuff

- [ ] Password strength validation (min 8 chars, letters + digits)
- [ ] Login failure rate limiting (nice-to-have, prevents brute force)
- [ ] JWT secret from env var (256-bit random key in production)
- [ ] JWT expiry: access_token 30min, refresh_token 7d

### 1.7 Tests

- [ ] Test login success / failure flows
- [ ] Test JWT expiry and refresh flow
- [ ] Test auth guard (no token / expired token / insufficient role)
- [ ] Test user CRUD + role assignment

> **Deps**: None (base module — everything else depends on its auth middleware)
