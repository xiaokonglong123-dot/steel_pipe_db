# 后端 — Rust 包 (steel-pipe-db)

## 技术栈
- **Rust** nightly-2024-02-08，版次 2021
- **单 crate** `steel-pipe-db`（无工作区）
- **SQLx** 0.8 使用 SQLite（runtime-tokio-rustls），启动时自动迁移

## 关键依赖（来自 Cargo.toml）
- `axum` 0.8 — HTTP 路由（macros、multipart 特性）
- `sqlx` 0.8 — SQL（sqlite、runtime-tokio-rustls、chrono 特性）
- `serde` / `serde_json` — JSON
- `jsonwebtoken` 9 — JWT 认证
- `argon2` 0.5 — 密码哈希（不是 bcrypt）
- `validator` 0.19 — 请求校验（derive 特性）
- `chrono` 0.4 — 日期/时间（serde 特性）
- `tokio` 1 — 异步运行时（full 特性）
- `tower-http` 0.6 — CORS、TraceLayer、request-id
- `tower` 0.5 — 工具函数
- `uuid` 1 — UUID 生成（v4 特性）
- `dotenvy` 0.15 — .env 加载
- `thiserror` 2 — 错误 derive 宏
- `calamine` 0.26 — Excel 导入
- `rust_xlsxwriter` 0.80 — Excel 导出
- `csv` 1.3 — CSV 导入/导出
- `tracing` / `tracing-subscriber` — 结构化日志（env-filter、json）

**重要：** 不包含 `rust_decimal`、`bigdecimal`、`backpack` 或 `bcrypt`。

## 构建与测试
```bash
cd backend
cargo check          # 仅类型检查（比构建快，CI 使用）
cargo build          # 调试构建
cargo build --release # 发布构建
cargo test           # 运行所有测试
```

## 数据库
- **SQLite** 文件路径来自 `DATABASE_URL` 环境变量（默认：`./data/steel_pipe.db`）
- **迁移文件**：`backend/migrations/` — SQLx 时间戳前缀文件
- 启动时自动运行迁移（`sqlx::migrate!("./migrations")`）
- 无需外部数据库服务器
- WAL 模式，通过 `deleted_at` 列实现软删除

## 模块结构

```
src/
├── main.rs              ← 入口点：tracing、DB 池、迁移、启动服务器
├── lib.rs               ← 模块声明、#![allow(dead_code)]
├── config.rs            ← 基于环境变量的配置（DATABASE_URL、JWT_SECRET 等）
├── error.rs             ← AppError 枚举，含数字错误码（10001-50001）
├── response.rs          ← ApiResponse<T>、PaginatedResponse<T>
├── router.rs            ← ~70 个端点，通过 .merge() 组合
├── domain/              ← 3 个文件（pipe.rs、inventory.rs、order.rs）— 枚举/领域类型
│   └── mod.rs
├── dto/                 ← 14 个文件，请求/响应结构体
│   ├── mod.rs
│   ├── auth_dto.rs
│   ├── pipe_dto.rs
│   ├── inventory_dto.rs
│   ├── purchase_dto.rs
│   ├── sales_dto.rs
│   ├── quality_dto.rs
│   ├── contract_dto.rs
│   ├── customer_dto.rs
│   ├── supplier_dto.rs
│   ├── label_dto.rs
│   ├── report_dto.rs
│   ├── data_io_dto.rs
│   └── common.rs
├── models/              ← 11 个文件，DB 行结构体（sqlx::FromRow）
│   ├── mod.rs
│   ├── user.rs
│   ├── seamless_pipe.rs
│   ├── screen_pipe.rs
│   ├── inventory.rs
│   ├── purchase_order.rs
│   ├── sales_order.rs
│   ├── quality.rs
│   ├── contract.rs
│   ├── customer.rs
│   └── supplier.rs
├── repositories/        ← 13 个文件，纯 SQL，软删除感知
│   ├── mod.rs
│   ├── pipe_repo.rs
│   ├── inventory_repo.rs
│   ├── purchase_order_repo.rs
│   ├── sales_order_repo.rs
│   ├── quality_repo.rs
│   ├── contract_repo.rs
│   ├── customer_repo.rs
│   ├── supplier_repo.rs
│   ├── label_repo.rs
│   ├── report_repo.rs
│   ├── data_io_repo.rs
│   ├── user_repo.rs
│   └── operation_log_repo.rs
├── services/            ← 12 个文件，业务逻辑（单元结构体 + 静态方法）
│   ├── mod.rs
│   ├── auth_service.rs
│   ├── pipe_service.rs
│   ├── inventory_service.rs
│   ├── purchase_sales_service.rs
│   ├── quality_service.rs
│   ├── contract_service.rs
│   ├── customer_service.rs
│   ├── supplier_service.rs
│   ├── label_service.rs
│   ├── report_service.rs
│   ├── data_io_service.rs
│   └── trace_service.rs
├── handlers/            ← 12 个文件，薄处理器（提取参数 → 调用服务 → 响应）
│   ├── mod.rs
│   ├── auth_handler.rs
│   ├── pipe_handler.rs
│   ├── inventory_handler.rs
│   ├── purchase_handler.rs
│   ├── sales_handler.rs
│   ├── quality_handler.rs
│   ├── contract_handler.rs
│   ├── customer_handler.rs
│   ├── supplier_handler.rs
│   ├── report_handler.rs
│   ├── label_handler.rs
│   └── data_io_handler.rs
└── middleware/          ← 2 个文件，认证 + RBAC
    ├── mod.rs
    ├── auth.rs          ← JWT 验证、Claims、AuthContext、auth_middleware
    └── rbac.rs          ← 基于角色的访问控制辅助函数
```

## 关键文件
- `Cargo.toml` — 包清单
- `.env.example` — 环境变量模板（DATABASE_URL、JWT_SECRET 等）
- `migrations/` — SQLx 时间戳前缀迁移文件

## Rust 约定
- 函数/变量使用 `snake_case`，类型使用 `PascalCase`
- `use` 语句：`use crate::{handlers, models, ...}` 模式
- `mod.rs` 文件重新导出公开项：`pub use pipe_handler::*;`
- 公开 API 函数为 `pub async fn` 并带有显式返回类型
- 内部辅助函数为 `pub(crate) fn` 或 `async fn`
- **所有处理器返回 `Result<Json<...>, AppError>`**（不是 `impl IntoResponse`）
- 服务是**带静态方法的单元结构体**（无构造函数 DI）：`PipeService::list(...)`
- 服务返回 `Result<T, AppError>`
- 仓库接受 `&SqlitePool` 并返回 `Result<Vec<T>, sqlx::Error>`

## DI 模式：Extension 层，而非 State<Arc<AppState>>
```rust
// router.rs 中的层：
.layer(CorsLayer::permissive())
.layer(TraceLayer::new_for_http())
.layer(Extension(pool))       // Extension<SqlitePool>
.layer(Extension(jwt_secret)) // Extension<String>

// 处理器提取：
pub async fn list_pipes(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<PipeFilterParams>,
) -> Result<Json<PaginatedResponse<Pipe>>, AppError> {
```
不存在 `AppState` 结构体。连接池和 JWT 密钥作为原始类型注入。

## 响应格式
```json
// 成功：    { "success": true, "data": T }
// 分页：    { "success": true, "data": { "items": [], "total": N, "page": P, "page_size": S, "total_pages": N } }
// 错误：    { "code": 11001, "message": "..." , "details": null }
```

## 错误码（数字，按领域前缀）
| 范围 | 领域 |
|-------|--------|
| 100xx | 通用（内部错误、校验、未找到） |
| 110xx | 认证（未授权、令牌过期、禁止） |
| 120xx | 钢管（未找到、重复、状态冲突） |
| 130xx | 库存（库存不足、位置已满） |
| 140xx | 订单（无法修改、未找到） |
| 150xx | 质量（证书未找到、附件未找到） |
| 160xx | 供应商（未找到、编码重复） |
| 170xx | 客户（未找到、编码重复） |
| 180xx | 数据 IO（导入错误、导出错误） |
| 50001 | 数据库 |
