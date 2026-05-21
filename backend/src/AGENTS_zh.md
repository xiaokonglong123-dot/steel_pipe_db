# `backend/src/` — 模块连接与路由

此目录连接所有后端源码模块。**不是添加新特性文件的位置** —— 那些应放在 `handlers/`、`services/`、`repositories/` 等目录中。

## 模块注册

**添加新后端模块的步骤：**
1. 在适当的子目录中创建文件（例如 `handlers/new_thing_handler.rs`）
2. 在子目录的 `mod.rs` 中添加 `pub mod new_thing_handler;`
3. 在 `router.rs` 中添加路由
4. 在处理器 `mod.rs` 中添加处理器

## `main.rs` — 入口点
- `#![allow(dead_code)]` 在 crate 根级别（压制未使用代码警告）
- `#[tokio::main]` 异步入口
- 初始化：日志（tracing）、DB 池（SqlitePool）、来自环境变量的配置
- 调用 `router::create_app(pool, jwt_secret)` 构建路由器
- 绑定 `0.0.0.0:3000`，开始服务
- **不存在 `AppState` 结构体** —— 依赖注入通过 `Extension<>` 层实现

## 共享状态模式 — `Extension<>` 而非 `State<>`
**此项目使用 Axum `Extension<>` 层进行依赖注入，而非 `State<Arc<AppState>>`。**
```rust
// router.rs（第 423-426 行）
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
    // ~50 个端点，按实体通过 .merge() 分组
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

## `error.rs` — 错误处理
```rust
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    BadRequest(String),
    Internal(String),
}
impl IntoResponse for AppError { ... }
```
- 所有服务层错误通过 `IntoResponse` 转换为 HTTP 4xx/5xx
- 使用 `map_err(AppError::from)` 或 `?` 运算符配合 `From` 实现

## `auth.rs` — JWT Claims
- `Claims` 结构体（带有 `sub`、`role`、`scope`、`exp` 的 JWT 载荷）
- 令牌生成和验证函数
- `AuthUser` 结构体由中间件提取
