# 后端 — Rust 包

## 技术
- **Rust** nightly-2024-02-08，版次 2021
- **Cargo 工作区**（单个 crate `db_backend`，无工作区成员子目录）
- **SQLx** 0.8 使用 SQLite，通过 `sqlx-data.json` 实现编译时查询检查

## 关键依赖（来自 Cargo.toml）
- `axum` 0.7 — HTTP 路由
- `sqlx` 0.8 — SQL（sqlite、runtime-tokio、derive 特性）
- `serde` / `serde_json` — JSON
- `jsonwebtoken` — JWT 认证
- `bcrypt` — 密码哈希
- `validator` — 请求校验（derive 特性）
- `rust_decimal` — 货币/数量计算
- `bigdecimal` — 精度十进制
- `chrono` — 日期/时间（serde 特性）
- `tokio` — 异步运行时（完整特性）
- `tower-http` — CORS、TraceLayer
- `backpack` — 额外校验功能

## 构建与测试
```bash
cd backend
cargo build            # 调试构建（~30 秒）
cargo build --release  # 发布构建
cargo test             # 运行所有测试
cargo sqlx prepare     # 查询变更后重新生成 sqlx-data.json
```

## 数据库
- **SQLite** 文件位于 `backend/db.db`（自动创建）
- **迁移**：`backend/migrations/` — SQLx 时间戳前缀文件
- 启动时自动运行迁移（embed_migrations! / sqlx::migrate!）
- 无需外部数据库服务器

## 模块结构

```
src/
├── bin/main.rs         ← 入口点：构建路由器，启动服务器
├── lib.rs              ← 模块声明
├── app_state.rs        ← 共享 AppState（DB 池、AppConfig）
├── error.rs            ← AppError 枚举、IntoResponse 实现
├── router.rs           ← 构建路由器，挂载所有处理器路由
├── auth.rs             ← JWT 中间件、claims 结构体
├── handlers/           ← 13 个文件，110+ 个处理函数
│   ├── mod.rs          ← pub mod 声明
│   ├── auth_handler.rs
│   ├── pipe_handler.rs
│   ├── inventory_handler.rs
│   └── ...             ← 每个实体一个文件
├── services/           ← 13 个文件，业务逻辑
│   ├── mod.rs
│   ├── auth_service.rs
│   ├── pipe_service.rs
│   ├── inventory_service.rs  ← 764 行（最大的服务）
│   └── ...
├── repositories/       ← 14 个文件，SQL 层
│   ├── mod.rs
│   ├── pipe_repo.rs
│   ├── inventory_repo.rs     ← 755 行（最大的仓库）
│   ├── report_repo.rs        ← 586 行
│   └── ...
├── models/             ← 11 个文件，数据库行结构体
│   ├── mod.rs
│   ├── pipe.rs
│   ├── inventory.rs
│   └── ...
├── dto/                ← 14 个文件，请求/响应结构体
│   ├── mod.rs
│   ├── pipe_dto.rs
│   └── ...
├── domain/             ← 4 个文件，枚举/领域类型
│   └── mod.rs
└── middleware/         ← 3 个文件，认证 + RBAC
    ├── mod.rs
    └── auth_middleware.rs
```

## 关键文件
- `Cargo.toml` — 包清单
- `build.rs` — Tauri/Vite 构建钩子（控制 `FRONTEND_DIST` 用于内嵌前端）
- `sqlx-data.json` — 预编译查询缓存（用于编译时检查）
- `db.db` — SQLite 数据库文件（gitignore）

## Rust 约定
- 函数/变量使用 `snake_case`，类型使用 `PascalCase`
- `use` 语句：`use crate::{handlers, models, ...}` 模式
- `mod.rs` 文件重新导出公开项：`pub use pipe_handler::*;`
- 公开 API 函数为 `pub async fn` 并带有显式返回类型
- 内部辅助函数为 `pub(crate) fn` 或 `async fn`
- 所有处理器返回 `impl IntoResponse`（Axum 模式）
- 服务返回 `Result<T, AppError>`
- 仓库接受 `&Pool<Sqlite>` 并返回 `Result<Vec<T>, sqlx::Error>`
