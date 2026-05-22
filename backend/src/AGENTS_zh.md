# `backend/src/` — 模块连接与路由

此目录连接所有后端源码模块。**不是添加新特性文件的位置** —— 那些应放在 `handlers/`、`services/`、`repositories/` 等目录中。

## 模块注册

**添加新后端模块的步骤：**
1. 在适当的子目录中创建文件（例如 `handlers/new_thing_handler.rs`）
2. 在子目录的 `mod.rs` 中添加 `pub mod new_thing_handler;`
3. 在 `router.rs` 中添加路由
4. 在处理器 `mod.rs` 中添加处理器

## `main.rs` — 入口点
- **文件位置**：`src/main.rs`（而非 `src/bin/main.rs`）
- `#![allow(dead_code)]` 在 crate 根级别（压制未使用代码警告）
- `#[tokio::main]` 异步入口
- 初始化：日志（tracing）、DB 池（SqlitePool）、来自环境变量的配置
- 调用 `router::create_app(pool, jwt_secret)` 构建路由器
- 绑定 `0.0.0.0:3000`，开始服务
- **不存在 `AppState` 结构体** —— 依赖注入通过 `Extension<>` 层实现

## 共享状态模式 — `Extension<>` 而非 `State<>`
**此项目使用 Axum `Extension<>` 层进行依赖注入，而非 `State<Arc<AppState>>`。**
```rust
// router.rs
.layer(CorsLayer::permissive())
.layer(TraceLayer::new_for_http())
.layer(Extension(pool))
.layer(Extension(jwt_secret))
```
- `Extension(SqlitePool)` — 原始池（无包装结构体）
- `Extension(String)` — JWT 密钥作为裸 `String`（无 newtype）
- 处理器通过 `Extension(pool): Extension<SqlitePool>` 提取

## `router.rs` — 路由挂载
```rust
pub fn create_app(pool: SqlitePool, jwt_secret: String) -> Router {
    // ~70 个端点，按实体通过 .merge() 分组
    Router::new()
        .route("/api/v1/auth/login", post(handlers::auth_handler::login))
        .route("/api/v1/pipes", get(handlers::pipe_handler::list))
        // ...
        .merge(pipe_routes)
        .merge(inventory_routes)
        // ...
        .layer(CorsLayer::permissive())
        .layer(Extension(pool))
        .layer(Extension(jwt_secret))
}
```
- 路由器函数**创建**路由，不接受预先构建的服务
- 每个处理器通过 `Extension<SqlitePool>` 和 `Extension<String>` 提取器接收依赖注入
- 认证中间件包裹单个子路由器，而非全局

## `error.rs` — 错误处理（数字错误码）
- `AppError` 枚举包含 **约 20 个变体**，按领域前缀分组（100xx–50001）
- 每个变体对应一个 **数字 `error_code()`**（例如 `Validation` → 10002）和一个 **HTTP `status_code()`**
- 使用 `thiserror::Error` 派生 `Display`
- 实现 `IntoResponse`，序列化为 `ApiErrorResponse { success: false, code, request_id, message, details }`
- `From<sqlx::Error>` 实现将 DB 错误转换为 `AppError::Database`
- 所有服务层错误通过 `?` 运算符配合 `From` 实现自动转换

领域划分：

| 范围   | 领域         |
|--------|-------------|
| 100xx  | 通用（Internal, Validation, NotFound, BadRequest） |
| 110xx  | 认证（Unauthorized, TokenExpired, Forbidden） |
| 120xx  | 钢管（NotFound, Duplicate, StatusConflict） |
| 130xx  | 库存（InsufficientStock, LocationFull） |
| 140xx  | 订单（CannotModify, NotFound） |
| 150xx  | 质量（CertNotFound, AttachmentNotFound） |
| 160xx  | 供应商（NotFound, CodeDuplicate） |
| 170xx  | 客户（NotFound, CodeDuplicate） |
| 180xx  | 数据导入导出（ImportError, ExportError） |
| 50001  | 数据库 |

## `response.rs` — 响应类型
- `ApiResponse<T>` 包含 `success: bool`、`request_id: String`、`data: T`
- `PaginatedResponse<T>` 包含 `success`、`request_id`、`meta: Meta`、`data: PaginatedData<T>`
- `ApiResponse::created(data)` 返回 201
- `no_content()` 函数返回 204

## `middleware/auth.rs` — JWT 中间件
- **`Claims`** 结构体 — JWT 载荷（`sub` 用户ID, `username`, `role`, `exp`, `iat`）
- **`AuthContext`** 提取器 — 从验证后的 JWT 中提取（包含 `user_id`, `username`, `role`）
- **`auth_middleware`** — Axum 中间件层，从 `Authorization` 头验证 Bearer 令牌
  - 从请求扩展中读取 JWT 密钥
  - 使用 HS256 通过 `jsonwebtoken` 解码令牌
  - 成功时：将 `AuthContext` 插入请求扩展
  - 失败时：返回包含 `success: false`、`request_id` 及数字错误码（11001/11002）的 `ApiErrorResponse`
- 中间件包裹单个子路由器，而非全局
- 令牌生成逻辑位于 **handler/service 层**（不在中间件中）
