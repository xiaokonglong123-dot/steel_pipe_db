# 技术债务跟踪 / Tech Debt Tracking

> 本文档记录已识别但**尚未修复**的后端代码与架构问题。文档类问题已在
> README.md / detailed-design.en.md / 各 AGENTS.md 中修复完毕，不在此列。
>
> **环境说明**：本仓库后端在当前开发环境已可通过 `cargo check` 编译（需设置
> `CARGO_HTTP_CAINFO=/tmp/trust-bundle.crt`）。前端 `tsc --noEmit` 在本环境正常。
>
> **历史沿革**：本系统由钢管行业系统重构而来，现为通用 ERP；本文档跟踪
> `erp-server`（代码阶段实施）的架构与技术债。

最后更新：2026-05-30

---

## 待修复

| # | 问题 | 类型 | 风险 | 说明 |
|---|------|------|------|:---:|
| C3 | 金额/小数用 `f64`（精度丢失） | 正确性 | 中 | **阻塞**（需联网装 crate） |

---

## C3 — 金额/小数用 f64（精度丢失）

**位置**：后端涉及金额/数量小数的字段（合同金额、单价、付款等）；详见 `models/` 与 `dto/` 中相关 f64 字段。

**问题**：货币与精确小数用 `f64`，存在浮点精度误差，财务场景不可接受。

**建议修法**：引入 `rust_decimal`（或 `bigdecimal`），金额字段改用 `Decimal`，并调整 SQLx 映射、DTO 序列化与前端展示。

**环境阻塞**：当前环境**无法联网安装 crate**，本项无法在此完成，必须在联网机器上进行。

**验证**：`cargo check` + `cargo test`（金额计算精度用例）。

---

## 已修复

### C1 — ATP 校验 TOCTOU 竞态 → 已修复

**修复**：`sales_service.rs::approve_sales_order` 改用 `BEGIN IMMEDIATE` 事务，
将 ATP 查询与状态更新放在单个串行化事务中。相关方法 `InventoryRepo::find_atp` 改为接受
泛型 executor，可在事务内部使用。

**修法**：方案 1 — 先 BEGIN IMMEDIATE，ATP 读取 + 库存占用全部放进同一事务。

**验证**：`cargo check` 通过。

### B1 — JWT 密钥 fail-open → 已修复

**修复**：`config.rs::resolve_jwt_secret` 新增 fail-closed 语义：

- 生产环境（`APP_ENV=production`）：JWT_SECRET 缺失、为占位值或 < 32 字节时 panic；
- 开发环境：warn 并允许使用 placeholder（保留 `cp .env.example .env && cargo run` 体验）。

**验证**：`cargo check` 通过；需手动验证 APP_ENV=production 未设 JWT_SECRET 时启动失败。

### B2 — 默认管理员种子凭证 → 已修复

**修复**：

- `migrations/001_create_users.sql`：移除硬编码的 INSERT 种子语句，仅保留建表；
- `main.rs`：新增 `bootstrap_admin()` 函数，启动时若 users 表为空，从 `ADMIN_USERNAME` /
  `ADMIN_PASSWORD` 环境变量创建首个管理员（Argon2id 哈希）。

**验证**：`cargo check` 通过；需验证全新 DB 迁移后首次登录流程。

### B3 / D1 — 前端 JWT 存 localStorage → 已修复

**修复**：

- 后端：添加 `axum-extra` cookie 依赖；login/refresh/logout handler 设置 httpOnly refresh_token cookie
- 前端：移除 localStorage 持久化，access token 仅存内存；axios 启用 `withCredentials`；页面加载时自动调用 `/auth/refresh` 恢复会话

**验证**：`cargo check` + `cargo test` + `tsc --noEmit` 均通过

### B4 — 无真正 refresh token / logout 不失效 → 已实现

**现状**：代码已实现完整的 refresh token rotation：

- `refresh_tokens` 表存储 SHA-256 哈希
- `/auth/refresh` 轮换旧 token（revoke + create）
- `/auth/logout` 撤销用户所有 refresh tokens

**验证**：`cargo test` 通过

---

## 已完成（文档侧，供参照）

- README.md：数据模型节重写为 ERP 通用商品（Item/SKU）/采购·销售订单/合同/财务等表；Security 节如实描述无状态 JWT / logout 不撤销 / refresh 7 天宽限续期。
- docs/detailed-design.en.md：补 `contract_items`、`contract_payments` 表结构与 ER 图关系。
- backend/AGENTS.md、AGENTS_zh.md：middleware 2→4（补 `rate_limit.rs`）。
- backend/src/services/AGENTS.md、AGENTS_zh.md：services 按保留模块组织，替换过时的 `inventory_service.rs` 为拆分服务。
- backend/src/handlers/auth_handler.rs：登录/刷新文档注释纠正（不再声称 access+refresh tokens / rotation）。
