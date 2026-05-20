# Phase 1 — 后端：系统管理与认证模块 (P0 MVP)

> 基于：`docs/需求文档.md` §3.10, §4.4；`docs/详细设计文档.md` §4.6, §5.3.15-16, §6.4, §10

## 任务清单

### 1.1 数据库迁移
- [ ] 创建 `users` 表迁移（含 username UNIQUE, password_hash, display_name, role, language_pref, unit_system, is_active, last_login_at）
- [ ] 创建 `operation_logs` 表迁移（审计日志）

### 1.2 领域层 (Domain)
- [ ] 定义 `User` 结构体
- [ ] 定义 `OperationLog` 结构体
- [ ] 定义 DTO：`LoginRequest`, `LoginResponse` (含 access_token, refresh_token, user), `RefreshRequest`, `CreateUserDto`, `UpdateUserDto`
- [ ] 定义枚举：`Role` (Admin / Warehouse / Qc / Sales)
- [ ] 定义 JWT Claims 结构体（含 sub, role, exp, iat）

### 1.3 仓库层 (Repository)
- [ ] 实现 `UserRepo`：
  - `find_by_username(username) -> Option<User>`
  - `create(dto) -> User`
  - `update(id, dto) -> User`
  - `find_by_id(id) -> Option<User>`
  - `list(filter) -> PaginatedResult<User>`
- [ ] 实现 `OperationLogRepo`：
  - `create(log) -> OperationLog`
  - `list(filter) -> PaginatedResult<OperationLog>`

### 1.4 服务层 (Service)
- [ ] 实现 `AuthService`：
  - `login(LoginRequest)`：验证用户名密码（Argon2）+ 生成 JWT（access_token + refresh_token）+ 记录最后登录时间 + 记录操作日志
  - `refresh_token(refresh_token)`：验证 refresh_token + 签发新的 token 对
  - `logout(user_id)`：记录登出日志
  - `get_current_user(user_id)`：获取当前用户信息
- [ ] 实现 `UserService`：
  - `create_user(dto)`：密码哈希 + 创建用户
  - `update_user(id, dto)`：更新信息（密码单独更新）
  - `list_users(filter)`：用户列表
  - `assign_role(user_id, role)`：分配角色

### 1.5 中间件与处理器层
- [ ] 实现认证中间件（`AuthMiddleware`）：
  - 从 `Authorization: Bearer <token>` 头解析 JWT
  - 验证 token 有效性（过期时间、签名）
  - 将用户信息注入 Request Extension
- [ ] 实现鉴权中间件（`RequireRole`）：
  - 从路由提取所需角色列表
  - 比较当前用户角色，无权限时返回 403
  - 可通过 `axum::middleware::from_extractor` 或自定义 Layer 实现
- [ ] 实现 Auth 端点：
  - `POST /api/v1/auth/login` — 登录
  - `POST /api/v1/auth/refresh` — 刷新 token
  - `POST /api/v1/auth/logout` — 登出
  - `GET /api/v1/auth/me` — 获取当前用户（需认证）
- [ ] 实现用户管理端点（需 admin 权限）：
  - `GET /api/v1/users` — 用户列表
  - `POST /api/v1/users` — 创建用户
  - `PUT /api/v1/users/{id}` — 更新用户
  - `PUT /api/v1/users/{id}/role` — 分配角色
- [ ] 实现操作日志端点：
  - `GET /api/v1/operation-logs` — 操作日志查询（需 admin）
- [ ] 实现 CORS 中间件配置（允许前端跨域请求）

### 1.6 安全相关
- [ ] 密码强度校验（最少 8 位，字母+数字）
- [ ] 登录失败次数限制（可选，防止暴力破解）
- [ ] JWT 密钥配置（从环境变量读取，生产环境建议 256 位随机密钥）
- [ ] JWT 过期时间：access_token 30min, refresh_token 7d

### 1.7 测试
- [ ] 测试登录成功/失败流程
- [ ] 测试 JWT 过期刷新流程
- [ ] 测试权限拦截（无 token / 过期 token / 角色不足）
- [ ] 测试用户 CRUD + 角色分配

> **依赖**: 无（基础模块，其他模块依赖其鉴权中间件）
