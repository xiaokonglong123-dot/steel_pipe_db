# Backend — Rust 包（erp-server）

> 历史沿革：本系统由钢管行业系统重构而来，旧模块与旧术语一律废弃。

## 技术栈

- **Rust** stable channel（`rust-toolchain.toml`），edition 2021
- **单一 crate** `erp-server`（无 workspace，无 monorepo）
- **SQLx** 0.8 + SQLite（runtime-tokio），启动时自动执行迁移

## 关键依赖（来自 Cargo.toml）

- `axum` 0.8 — HTTP 路由（macros + multipart features）
- `sqlx` 0.8 — SQL（sqlite、runtime-tokio、chrono features）
- `serde` / `serde_json` — JSON
- `jsonwebtoken` 9 — JWT 认证
- `argon2` 0.5 — 密码哈希（NOT bcrypt）
- `validator` 0.19 — 请求校验（derive feature）
- `chrono` 0.4 — 日期/时间（serde feature）
- `tokio` 1 — 异步运行时（full features）
- `tower-http` 0.6 — CORS、TraceLayer、request-id
- `tower` 0.5 — 工具
- `uuid` 1 — UUID 生成（v4 feature）
- `dotenvy` 0.15 — .env 加载
- `thiserror` 2 — 错误派生宏
- `calamine` 0.26 — Excel 导入
- `rust_xlsxwriter` 0.80 — Excel 导出
- `csv` 1.3 — CSV 导入/导出
- `tracing` / `tracing-subscriber` — 结构化日志（env-filter、json）

**注意：** 金额输入层已引入 `rust_decimal` + `rust_decimal_macros`（DTO、`domain/money.rs`）；库表与计算仍用 `f64`。这里没有 `bigdecimal`、`backpack` 或 `bcrypt`，不要去找它们。

## 构建与测试

```bash
cd backend
cargo check          # 仅类型检查（比 build 快，CI 使用）
cargo build          # Debug 构建
cargo build --release # Release 构建
cargo test           # 运行全部测试
```

## 数据库

- **SQLite** 文件，连接串 `sqlite://data/erp.db?mode=rwc`（由 `DATABASE_URL` 环境变量指定，首次运行自动创建）
- **迁移**：`backend/migrations/` — SQLx 时间戳前缀文件。37 个历史迁移文件将重写为 SQLite 语法，并删除钢管时代的遗留表。
- 启动时通过 `sqlx::migrate!("./migrations")` 自动迁移
- 无需外部数据库服务器——就是一个文件
- 启用 WAL 模式，软删除通过 `deleted_at` 列实现

## 模块结构

```
src/
├── main.rs              ← 入口：tracing、DB 连接池、迁移、启动服务
├── lib.rs               ← 模块声明，#![allow(dead_code)]
├── config.rs            ← 环境变量配置（DATABASE_URL、JWT_SECRET 等）
├── error.rs             ← AppError 枚举，数字错误码（10001-50001）
├── response.rs          ← ApiResponse<T>、PaginatedResponse<T>
├── router.rs            ← ~190 个路由（~170 个唯一路径），通过 .merge() 组装
├── cache.rs             ← 响应缓存 + cache_invalidator.rs
├── domain/              ← 通用领域类型（商品、库存、单据、金额）
├── dto/                 ← 请求/响应结构体（每实体一个文件）
├── models/              ← 数据库行结构体（sqlx::FromRow）
├── repositories/        ← 纯 SQL，软删除感知
├── services/            ← 业务逻辑（unit struct + 静态方法）
├── handlers/            ← 薄 HTTP 处理器（提取 → 调服务 → 响应）
├── middleware/          ← auth、rbac、rate_limit、security_headers
├── auth/                ← RBAC：角色 / 权限 / 部门 / 租户
├── workflow/            ← 审批引擎：审批流定义 / 实例 / 任务
├── hr/                  ← 员工 / 考勤 / 薪资 / 劳动合同
├── finance/             ← 会计科目 / 日记账 / 发票 / 付款 / 试算平衡
├── procurement/         ← 采购申请 / 收货 / 报价 / 供应商评分
├── sales_crm/           ← 发货 / 报价 / 客户信用
├── inventory_atp/       ← 商品/SKU 库存：预留 / 调拨 / 盘点
├── manufacturing/       ← BOM / 工单 / 质检 / 不合格品单
├── project/             ← 项目 / WBS / 预算
├── assets/              ← 固定资产：登记 / 折旧 / 处置
├── notification/        ← 收件箱 / 模板 / 偏好
├── portal/              ← 门户账户 / 当事人 JWT / 采购订单确认 / 销售订单回执
└── bi/                  ← 销售趋势 / 库存价值 / 财务汇总 / 供应商绩效
```

每个功能模块（`auth/`、`workflow/`、`hr/`、…）遵循相同的布局：`mod.rs` + `handlers.rs` + `repos.rs` + `services.rs`（`bi/` 无 `repos.rs`——只读分析，复用共享仓储）。

核心分层明细：

```
├── domain/              ← 通用领域类型（item、inventory、order、money）
├── dto/                 ← auth_dto、item_dto、inventory_dto、purchase_dto、sales_dto、
│                          contract_dto、customer_dto、supplier_dto、report_dto、data_io_dto、common、…
├── models/              ← 数据库行结构体：user、rbac、item、inventory、purchase_order、
│                          sales_order、contract、customer、supplier、workflow、hr、finance、
│                          procurement、sales_crm、inventory_atp、manufacturing、project、
│                          assets、notification、portal
├── repositories/        ← item_repo、inventory_repo、location_repo、inbound_repo、outbound_repo、
│                          inventory_log_repo、check_repo、purchase_order_repo、sales_order_repo、
│                          contract_repo、customer_repo、supplier_repo、report_repo、data_io_repo、
│                          user_repo、operation_log_repo、refresh_token_repo
├── services/            ← auth_service、item_service、inbound_service、outbound_service、
│                          check_service、inventory_query_service、location_service、
│                          purchase_service、sales_service、contract_service、customer_service、
│                          supplier_service、report_service、data_io_service
├── handlers/            ← auth_handler、item_handler、inbound_handler、outbound_handler、
│                          location_handler、check_handler、inventory_handler、purchase_handler、
│                          sales_handler、contract_handler、customer_handler、supplier_handler、
│                          report_handler、data_io_handler、atp_handler、health_handler
└── middleware/          ← auth.rs（JWT）、rbac.rs、rate_limit.rs、security_headers.rs
```

库存已泛化为**商品/SKU**：商品主数据表承载 `sku` / 名称 / 分类 / 单位 / 可选规格——无任何行业专属字段。预留、调拨、盘点位于 `inventory_atp/`。

## 关键文件

- `Cargo.toml` — 包清单（crate `erp-server`）
- `.env.example` — 环境变量模板（DATABASE_URL、JWT_SECRET 等）
- `migrations/` — SQLx 时间戳前缀迁移文件（已重写为 SQLite 语法；钢管表已删除）

## Rust 约定

- 函数/变量用 `snake_case`，类型用 `PascalCase`
- `use` 语句遵循 `use crate::{handlers, models, ...}` 模式
- `mod.rs` 重导出公共项：`pub use item_handler::*;`
- 公共 API 函数为 `pub async fn`，返回类型显式
- 内部辅助函数为 `pub(crate) fn` 或 `async fn`
- **所有 handler 返回 `Result<Json<...>, AppError>`**（不是 `impl IntoResponse`）
- Service 是 **unit struct + 静态方法**（无构造器 DI）：`ItemService::list(...)`
- Service 返回 `Result<T, AppError>`；仓储接受 `&SqlitePool`，返回 `Result<Vec<T>, sqlx::Error>`
- 库存服务层按职责拆分：
  - `inbound_service.rs` — 入库（创建/审批/批量执行）
  - `outbound_service.rs` — 出库（创建/审批/库存扣减）
  - `check_service.rs` — 盘点（创建/提交/完成）
  - `inventory_query_service.rs` — 只读查询（列表/统计）
  - `location_service.rs` — 库位 CRUD、分配、调拨
- 采购与销售拆分：
  - `purchase_service.rs` — 采购订单生命周期、审批、拒绝
  - `sales_service.rs` — 销售订单生命周期、ATP 验证、审批
- ATP 计算位于 `sales_service.rs` 和 `atp_handler.rs`

## DI 模式：Extension 层，而非 State<Arc<AppState>>

```rust
// router.rs layers:
.layer(CorsLayer::permissive())
.layer(TraceLayer::new_for_http())
.layer(Extension(pool))       // Extension<SqlitePool>
.layer(Extension(JwtSecret(jwt_secret))) // Extension<JwtSecret>

// Handler 提取:
pub async fn list_items(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<ItemFilterParams>,
) -> Result<Json<PaginatedResponse<Item>>, AppError> {
```

不存在 `AppState` 结构体。DB 连接池直接注入；JWT 密钥包装为 `JwtSecret`，类型安全、`Debug` 输出脱敏，且不会与任意字符串 Extension 混淆。

## 响应形状

```json
// 成功:    { "success": true, "request_id": "req_...", "data": T }
// 分页:    { "success": true, "request_id": "req_...", "meta": { "total": N, "page": P, "page_size": S, "total_pages": N }, "data": { "items": [], ... } }
// 错误:    { "success": false, "code": 11001, "request_id": "req_...", "message": "...", "details": null }
```

`tower-http` 同时设置/透传 `x-request-id` 头，CORS 向浏览器暴露该头。

## 错误码（数字、按域前缀）

| 范围 | 域 |
| ------- | -------- |
| 100xx | General（Internal、Validation、NotFound） |
| 110xx | Auth（Unauthorized、TokenExpired、Forbidden） |
| 120xx | Item 商品（NotFound、Duplicate、StatusConflict） |
| 130xx | Inventory（InsufficientStock、LocationNotFound） |
| 140xx | Orders（CannotModify、NotFound） |
| 150xx | Inspection 质检（NotFound、StatusConflict） |
| 160xx | Supplier（NotFound、CodeDuplicate） |
| 170xx | Customer（NotFound、CodeDuplicate） |
| 180xx | Data IO（ImportError、ExportError） |
| 50001 | Database |
