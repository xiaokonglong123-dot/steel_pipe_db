# PostgreSQL 迁移 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use `- [ ]` syntax.

**Goal:** 将后端从 SQLite 迁移到 PostgreSQL 18，使后续 ERP 模块（auth RBAC schema 隔离等）的设计可落地。

**Architecture:** 保持单 crate 结构（不引入 workspace），仅切换 sqlx 驱动 feature。所有表迁入 `public` schema（原样），auth 模块的 `auth` schema 在 001 计划中添加。测试改为连接本地 PG 测试库（`steel_pipe_test`）。

**Tech Stack:** Rust (sqlx 0.8 postgres, runtime-tokio-rustls), PostgreSQL 18.4

## Global Constraints

- 保持现有 handler/service/repo 分层模式不变
- **不要创建 workspace 结构**（`backend/crates/` 在后续 auth 计划中由用户另行决策）
- PostgreSQL 参数占位符是 `$1, $2, ...`（NOT `?`）——所有 SQL 必须重排
- `AUTOINCREMENT` → `GENERATED ALWAYS AS IDENTITY`（或 `BIGSERIAL`）
- `REAL` → `DOUBLE PRECISION`；金额字段用 `NUMERIC`（如需精确）
- `datetime('now')` → `NOW()`；`datetime(..., 'localtime')` → `NOW() AT TIME ZONE 'UTC'`
- `INSERT OR IGNORE` → `ON CONFLICT DO NOTHING`
- `strftime('%Y-%m-%d', ...)` → `TO_CHAR(..., 'YYYY-MM-DD')`
- `datetime(?)` 参数绑定 → `$1` 类型化传参（chrono::NaiveDateTime / String 由 sqlx 处理）
- `IF NOT EXISTS` 在 PG 中同样支持（保留）
- `INTEGER` 布尔列（`is_active INTEGER NOT NULL DEFAULT 1`）→ `BOOLEAN NOT NULL DEFAULT TRUE`，且 **Rust 侧字段类型从 i64 改为 bool**（sqlx postgres 的 i64↔bool 不兼容）
- 软删除语义不变：`deleted_at TIMESTAMPTZ`
- 验证命令：`cargo check`（用 CARGO_HTTP_CAINFO 环境变量）+ `cargo test`（连本地 PG）
- 测试使用 `steel_pipe_test` 库，连接串 `postgres://postgres@localhost:5432/steel_pipe_test`（/tmp socket 由 PGHOST 指定）

---

### Task 1: 基础设施切换 (Cargo.toml + config.rs + main.rs)

**Files:**
- Modify: `backend/Cargo.toml`
- Modify: `backend/src/config.rs`
- Modify: `backend/src/main.rs`

**Interfaces:**
- Produces: `Config.database_url` 默认值改为 `postgres://postgres@localhost:5432/steel_pipe_erp`；`main.rs` 使用 `PgPoolOptions`；bootstrap_admin 用 `&sqlx::PgPool`

- [ ] **Step 1: 修改 Cargo.toml**

```toml
# 替换 sqlite feature 为 postgres
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"] }
```

- [ ] **Step 2: 修改 config.rs database_url 默认值**

```rust
database_url: env::var("DATABASE_URL")
    .unwrap_or_else(|_| "postgres://postgres@localhost:5432/steel_pipe_erp".to_string()),
```

（同时更新 doc comment 提及 PostgreSQL）

- [ ] **Step 3: 修改 main.rs 连接池**

```rust
use sqlx::postgres::PgPoolOptions;
// ...
let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&cfg.database_url)
    .await
    .expect("Failed to connect to database");
// bootstrap_admin 签名改为 pool: &sqlx::PgPool
```

- [ ] **Step 4: 验证编译**

```bash
export CARGO_HTTP_CAINFO="/home/yzp/.local/share/Steam/steamrt64/pv-runtime/steam-runtime-steamrt/steamrt3c_platform_3c.0.20260618.246540/files/etc/ssl/certs/ca-certificates.crt"
cargo check 2>&1 | grep -E "^error" | head -20
```
Expected: 大量 SqlitePool 类型错误（预期，后续任务逐个修复）。**此任务不要求全绿**，只验证 main.rs 自身改动无误。

- [ ] **Step 5: Commit**

```bash
git add backend/Cargo.toml backend/src/config.rs backend/src/main.rs
git commit -m "feat: 切换到 PostgreSQL 驱动基础设施"
```

---

### Task 2: 迁移脚本重写（20 个文件）

**Files:**
- Modify: `backend/migrations/001_*.sql` ~ `021_*.sql`（全部）

**Interfaces:**
- Produces: PostgreSQL 兼容的迁移脚本（public schema）

- [ ] **Step 1: 逐文件重写迁移脚本**

每个文件应用以下转换规则（示例：001_create_users.sql）：

```sql
-- SQLite 原文:
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    ...
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    ...
);

-- PostgreSQL 目标:
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    ...
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ...
);
```

转换清单（逐文件检查）：
- [ ] `INTEGER PRIMARY KEY AUTOINCREMENT` → `BIGSERIAL PRIMARY KEY`
- [ ] `INTEGER NOT NULL DEFAULT 1`（布尔语义）→ `BOOLEAN NOT NULL DEFAULT TRUE`
- [ ] `REAL` → `DOUBLE PRECISION`
- [ ] `TEXT NOT NULL DEFAULT (datetime('now'))` → `TIMESTAMPTZ NOT NULL DEFAULT NOW()`
- [ ] `TEXT` 日期列（无默认值，如 production_date）→ `TIMESTAMPTZ`（或保留 TEXT 若代码以字符串处理——**优先 TIMESTAMPTZ**，同步改模型）
- [ ] `INSERT OR IGNORE` → `ON CONFLICT DO NOTHING`
- [ ] 保留 `IF NOT EXISTS`、`CREATE INDEX`、`CHECK` 约束
- [ ] 无 `sqlite_master`、`PRAGMA` 引用

- [ ] **Step 2: 在 PG 上验证迁移**

```bash
export PATH=~/.local/pgsql/bin:$PATH LD_LIBRARY_PATH=~/.local/pgsql/lib:$LD_LIBRARY_PATH
# 用 sqlx-cli 或临时 Rust 测试验证迁移可在 steel_pipe_test 库跑通
```

- [ ] **Step 3: Commit**

```bash
git add backend/migrations/
git commit -m "feat: 迁移脚本重写为 PostgreSQL 语法"
```

---

### Task 3: 测试基建切换 (tests/common)

**Files:**
- Modify: `backend/tests/common/mod.rs`

**Interfaces:**
- Produces: `test_pool()` 返回 `PgPool`，连接 `postgres://postgres@localhost:5432/steel_pipe_test`，每次运行前清理旧表并重跑迁移

- [ ] **Step 1: 重写 test_pool**

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};

pub const TEST_DATABASE_URL: &str = "postgres://postgres@localhost:5432/steel_pipe_test";

pub async fn test_pool() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(1)
        .connect(TEST_DATABASE_URL)
        .await
        .expect("failed to connect to test database");

    // 清理所有表（每次测试运行从干净状态开始）
    sqlx::query("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
        .execute(&pool)
        .await
        .expect("failed to reset schema");

    let migrator = sqlx::migrate!("./migrations");
    migrator.run(&pool).await.expect("failed to run migrations");
    pool
}
```

- [ ] **Step 2: 处理 temp_file_pool 等辅助函数**（改为也返回 PgPool 或删除，视使用情况）

- [ ] **Step 3: 验证编译**

```bash
export CARGO_HTTP_CAINFO="..."
cargo check --tests 2>&1 | grep -E "^error" | head -10
```
Expected: tests/ 中所有 `.await` 测试的 SqlitePool 类型错误。**此任务仅验证 common 模块自身**。

- [ ] **Step 4: Commit**

```bash
git add backend/tests/common/
git commit -m "feat: 测试基建切换为 PostgreSQL"
```

---

### Task 4: 共享层类型改造（models + dto + error + response）

**Files:**
- Modify: `backend/src/models/*.rs`（11 个文件）
- Modify: `backend/src/dto/*.rs`（如需）
- Modify: `backend/src/error.rs`、`backend/src/response.rs`

**Interfaces:**
- Produces: 所有 `sqlx::FromRow` 结构体字段类型与 PG 列类型匹配（i64 ↔ BIGSERIAL, bool ↔ BOOLEAN, chrono::DateTime<Utc> ↔ TIMESTAMPTZ）

- [ ] **Step 1: 转换模型字段类型**

规则：
- `id: i64` → 保持 `i64`（BIGSERIAL 映射 i64 兼容）
- `is_active: i64` → `bool`
- `created_at: String` → `chrono::DateTime<Utc>`（若迁移改为 TIMESTAMPTZ）
- 若某列保留 TEXT（如 date 字段），保持 String

- [ ] **Step 2: 检查 error.rs / response.rs** 是否有 sqlx::sqlite 类型引用，替换为 postgres 类型

- [ ] **Step 3: 验证**

```bash
export CARGO_HTTP_CAINFO="..."
cargo check 2>&1 | grep -E "^error" | head -20
```

- [ ] **Step 4: Commit**

```bash
git add backend/src/models/ backend/src/dto/ backend/src/error.rs backend/src/response.rs
git commit -m "feat: 模型与共享层类型适配 PostgreSQL"
```

---

### Task 5: 仓储层 SQL 改造（21 个文件，最大工作量）

**Files:**
- Modify: `backend/src/repositories/*.rs`（全部）

**Interfaces:**
- Produces: 所有 repo 方法接受 `&PgPool`，SQL 用 `$N` 占位符

- [ ] **Step 1: 批量替换类型**：`&SqlitePool` → `&PgPool`，`SqliteQueryResult` → `PgQueryResult`，`sqlx::Sqlite` → `sqlx::Postgres`

- [ ] **Step 2: 占位符重排**：每个 SQL 字符串中的 `?` 按出现顺序改为 `$1, $2, ...`

示例：
```rust
// SQLite:
sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ? AND deleted_at IS NULL")
    .bind(username)
// PostgreSQL:
sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1 AND deleted_at IS NULL")
    .bind(username)
```

- [ ] **Step 3: 日期函数替换**：`datetime('now')` → `NOW()`；`strftime(...)` → `TO_CHAR(...)` 或 `date_trunc(...)`；`datetime(?, 'localtime')` 相关绑定改为直接传 chrono 值

- [ ] **Step 4: 每完成一个文件立即编译验证**

```bash
export CARGO_HTTP_CAINFO="..."
cargo check 2>&1 | grep -E "^error" | head -5
```

- [ ] **Step 5: 全部完成后提交**

```bash
git add backend/src/repositories/
git commit -m "feat: 仓储层 SQL 适配 PostgreSQL"
```

---

### Task 6: 服务层 + 处理器层适配

**Files:**
- Modify: `backend/src/services/*.rs`（19 个文件）
- Modify: `backend/src/handlers/*.rs`（16 个文件）

**Interfaces:**
- Produces: 服务方法接受 `&PgPool`，处理器 extract `Extension<PgPool>`

- [ ] **Step 1: 服务层类型替换**：`&SqlitePool` → `&PgPool`；修复因模型类型变化（bool/DateTime）引发的编译错误

- [ ] **Step 2: 处理器层**：`Extension<SqlitePool>` → `Extension<PgPool>`；`Extension(pool): Extension<SqlitePool>` → `Extension<PgPool>`

- [ ] **Step 3: 编译验证到全绿**

```bash
export CARGO_HTTP_CAINFO="..."
cargo check 2>&1 | grep -cE "^error"
```
Expected: 0

- [ ] **Step 4: Commit**

```bash
git add backend/src/services/ backend/src/handlers/
git commit -m "feat: 服务层与处理器层适配 PostgreSQL"
```

---

### Task 7: 集成测试修复

**Files:**
- Modify: `backend/tests/*.rs`（12 个测试文件）

- [ ] **Step 1: 修复测试文件类型错误**（SqlitePool → PgPool 引用、seed 函数签名）

- [ ] **Step 2: 运行完整测试套件**

```bash
export CARGO_HTTP_CAINFO="..." PATH=~/.local/pgsql/bin:$PATH LD_LIBRARY_PATH=~/.local/pgsql/lib:$LD_LIBRARY_PATH
cargo test 2>&1 | tail -30
```
Expected: 所有测试通过（若个别测试逻辑因 PG 行为差异失败，修复测试断言——但**不删除测试**）

- [ ] **Step 3: Commit**

```bash
git add backend/tests/
git commit -m "feat: 集成测试适配 PostgreSQL"
```

---

### Task 8: 全量验证 + 手动运行冒烟

- [ ] **Step 1: 全量验证**

```bash
export CARGO_HTTP_CAINFO="..."
cargo check && cargo test 2>&1 | grep -E "test result" | tail -15
```
Expected: 全部 pass

- [ ] **Step 2: 手动冒烟**（连接开发库 steel_pipe_erp）

```bash
export PATH=~/.local/pgsql/bin:$PATH LD_LIBRARY_PATH=~/.local/pgsql/lib:$LD_LIBRARY_PATH
export DATABASE_URL="postgres://postgres@localhost:5432/steel_pipe_erp"
cargo run 2>&1 | tail -5
# 另一个终端: curl localhost:3000/api/v1/auth/login -d '{"username":"admin","password":"admin123"}'
```
Expected: 服务器启动，登录返回 token

- [ ] **Step 3: 更新 AGENTS.md 与 .env.example**（数据库相关说明改为 PostgreSQL）

- [ ] **Step 4: 最终提交**

```bash
git add -A
git commit -m "docs: 更新数据库说明为 PostgreSQL"
```

---

**迁移完成标准**：cargo check 全绿、cargo test 全绿、服务器可启动、登录 API 返回 token。
