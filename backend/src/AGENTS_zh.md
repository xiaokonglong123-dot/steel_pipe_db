# `backend/src/` — 模块装配与路由

以下所有模块都属于单一后端 crate **`erp-server`**（代码阶段实施目标）。

本目录负责把后端所有源码模块装配在一起。不要把新功能文件丢在这里——它们应放在功能模块目录：资源域（`items/`、`inventory/`、`orders/`、`contracts/`、`parties/`、`reports/`、`data_io/`）与业务域（`auth/`、`workflow/`、`hr/`、`finance/`、`procurement/`、`sales_crm/`、`inventory_atp/`、`manufacturing/`、`project/`、`assets/`、`notification/`、`portal/`、`bi/`）。顶层单文件：`config.rs`、`error.rs`、`response.rs`、`router.rs`、`macros.rs`、`health.rs`、`utils.rs`、`operation_log.rs`、`cache.rs`。

## 模块注册

**要新增一个模块？步骤如下：**

1. 在正确的模块目录创建文件（如 `inventory/handlers.rs` 或 `orders/services.rs`）
2. 在该模块的 `mod.rs` 中添加声明
3. 在 `router.rs` 中挂载路由（`use crate::<模块>::<文件>;`）

功能模块是自包含的：每个模块有自己的 `mod.rs` + `handlers.rs` + `repos.rs` + `services.rs`，并从 `lib.rs` 注册（如 `pub mod workflow;`）。

## `main.rs` — 入口点

- **文件**：`src/main.rs`（不是 `src/bin/main.rs`）
- `#![allow(dead_code)]` 位于 crate 根（压制合理的未用代码警告）
- `#[tokio::main]` 异步入口
- 初始化：tracing（日志）、DB 连接池（SqlitePool）、从环境变量读取 Config
- 调用 `router::create_app(pool, jwt_secret)` 构建路由
- 绑定 `0.0.0.0:3000` 开始服务
- **没有 `AppState` 结构体** — 使用 `Extension<>` 层做 DI

## 共享状态模式 — `Extension<>` 而非 `State<>`

**本项目使用 Axum `Extension<>` 层做依赖注入，而非 `State<Arc<AppState>>`。**

```rust
// router.rs
.layer(CorsLayer::permissive())
.layer(TraceLayer::new_for_http())
.layer(Extension(pool))
.layer(Extension(JwtSecret(jwt_secret)))
```

- `Extension(SqlitePool)` — 裸连接池，无包装结构体
- `Extension(JwtSecret)` — JWT 密钥 newtype，`Debug` 输出脱敏；缺少该 Extension 时直接失败关闭，而不是用空回退
- handler 通过 `Extension(pool): Extension<SqlitePool>` 取所需依赖

## `router.rs` — 路由挂载

```rust
pub fn create_app(pool: SqlitePool, jwt_secret: String) -> Router {
    // ~190 个路由（~170 个唯一路径），按实体分组并通过 .merge() 组装
    Router::new()
        .route("/api/v1/auth/login", post(handlers::auth_handler::login))
        .route("/api/v1/items", get(handlers::item_handler::list))
        // ...
        .merge(item_routes)
        .merge(inventory_routes)
        // ...
        .layer(CorsLayer::permissive())
        .layer(Extension(pool))
        .layer(Extension(JwtSecret(jwt_secret)))
}
```

- 路由函数**现场创建**路由——不接受预构建的服务
- handler 通过 `Extension<SqlitePool>` 获得 DI，认证 handler 使用 `Extension<JwtSecret>`
- 认证中间件包裹单个子路由，而非整个应用
- Request ID 中间件设置/透传 `x-request-id`；CORS 向浏览器暴露该头
- 路由按域分组：auth、商品（商品/SKU）、库存（入库/出库/库位/盘点/ATP）、采购（采购订单）、销售（销售订单）、合同、客户、供应商、审批流、HR、财务、采购管理、销售 CRM、制造、项目、固定资产、通知、门户、BI、data-io

## `error.rs` — 错误处理（数字错误码）

- `AppError` 枚举有 **约 20 个变体**，按域前缀分组（100xx–50001）
- 每个变体映射到**数字 `error_code()`**（如 `Validation` → 10002）和 **HTTP `status_code()`**
- 使用 `thiserror::Error` 派生 `Display`
- 实现 `IntoResponse`，序列化为 `ApiErrorResponse { success: false, code, request_id, message, details }`
- `From<sqlx::Error>` 将 DB 错误转换为 `AppError::Database`
- 服务错误通过 `?` 操作符 + `From` 实现转换——干净简单

域划分：

| 范围 | 域 |
| --------- | -------------- |
| 100xx | General（Internal、Validation、NotFound、BadRequest） |
| 110xx | Auth（Unauthorized、TokenExpired、Forbidden） |
| 120xx | Item 商品（NotFound、Duplicate、StatusConflict） |
| 130xx | Inventory（InsufficientStock、LocationNotFound） |
| 140xx | Orders（CannotModify、NotFound） |
| 150xx | Inspection 质检（NotFound、StatusConflict） |
| 160xx | Supplier（NotFound、CodeDuplicate） |
| 170xx | Customer（NotFound、CodeDuplicate） |
| 180xx | Data IO（ImportError、ExportError） |
| 50001 | Database |

## `response.rs` — 响应类型

- `ApiResponse<T>` 包含 `success: bool`、`request_id: String`、`data: T`
- `PaginatedResponse<T>` 包含 `success`、`request_id`、`meta: Meta`、`data: PaginatedData<T>`
- `ApiResponse::created(data)` 返回 201
- `no_content()` 返回 204

## `middleware/auth.rs` — JWT 中间件

- **`Claims`** 结构体 — JWT 载荷（`sub` user_id、`username`、`role`、`exp`、`iat`）
- **`AuthContext`** 提取器 — 从已验证的 JWT 令牌中提取（包含 `user_id`、`username`、`role`）
- **`auth_middleware`** — 校验 `Authorization` 头 Bearer 令牌的 Axum 中间件层
  - 从请求扩展中读取 JWT 密钥
  - 缺少 `JwtSecret` 返回 500 `Authentication is not configured`，而不是静默使用空密钥
  - 通过 `jsonwebtoken` 以 HS256 解码令牌
  - 成功：将 `AuthContext` 注入请求扩展
  - 失败：返回 401 + `ApiErrorResponse`（含 `success: false`、`request_id`、数字错误码 11001/11002）
- 中间件包裹单个子路由（而非整个应用）
- 令牌生成在 **handler/service 层**，不在中间件中

## RBAC 速查（`router.rs` 中的路由分组）

| 域     | 读（任意已认证） | 写（角色）                    |
|------------|:---------------:|----------------------------------|
| Users      | admin           | admin                            |
| Items      | ✅              | admin、warehouse                 |
| Inbound    | ✅              | admin、warehouse                 |
| Outbound   | ✅              | admin、warehouse                 |
| Inventory  | ✅              | admin、warehouse                 |
| Sales      | ✅              | admin、sales                     |
| Purchases  | ✅              | admin、warehouse、sales          |
| Suppliers  | ✅              | admin、warehouse、sales          |
| Customers  | ✅              | admin、warehouse、sales          |
| Contracts  | ✅              | admin、warehouse、sales          |
| Data IO    | templates       | admin（导入/日志）、admin/warehouse/sales（导出） |
| Reports    | ✅              | —（只读）                    |
