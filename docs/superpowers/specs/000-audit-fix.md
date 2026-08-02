# 000 — Steel Pipe ERP: 审计修复 & 基础设施迁移 (Phase 0)

> **版本**: v1.0  
> **日期**: 2026-08-02  
> **状态**: Draft  
> **优先级**: P0 — 地基加固  

---

## 目标

在开始 ERP 模块开发之前，修复当前代码库中所有已知缺陷，完成从 SQLite 到 PostgreSQL 的迁移，并通过 Docker Compose 代替裸启动。

---

## 2. 缺陷修复清单

### 2.1 🔴 P0 — 质量附件查询参数不匹配

**问题**: 前端发送 `cert_id`，后端期望 `pipe_type` + `pipe_id` → 附件永远为空

**修复** (`backend/src/handlers/quality_handler.rs`):
```rust
pub async fn list_attachments_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<PipeAttachment>>>, AppError> {
    // 优先按 cert_id 查询：(done)
    if let Some(cert_id) = params.get("cert_id").and_then(|s| s.parse::<i64>().ok()) {
        let cert = QualityService::get_cert_by_id(&pool, cert_id).await?;
        return QualityService::list_attachments(&pool, &cert.pipe_type, cert.pipe_id);
    }
    // 备选 pipe_type + pipe_id
    let pipe_type = params.get("pipe_type").map(|s| s.as_str());
    let pipe_id = params.get("pipe_id").and_then(|s| s.parse::<i64>().ok());
    QualityService::list_attachments(&pool, pipe_type, pipe_id)
}
```

### 2.2 ⚠️ P0 — UserManagementPage 无路由注册

**问题**: `frontend/src/features/auth/pages/UserManagementPage.tsx` 331 行功能完整，但无路由/no 菜单

**修复**:
1. 在 `routes/index.tsx` 添加：
```tsx
{ path: 'settings/users', element: route(<UserManagementPage />), handle: { roles: ['admin'] } }
```
2. 在 `MainLayout.tsx` 添加菜单项

### 2.3 ⚠️ P1 — 审批评论死代码

**问题**: `ApproveRequest.reason` 字段但从不用使用

**修复**:
1. `InventoryService::approve_inbound` 签名加 `reason: Option<&str>`, `user_id: i64`
2. 将 reason 写入 operation_log、并于 inbound_records 增加 `approval_reason` 列

### 2.4 ⚠️ P1 — 报表子页面路由缺失 (3 个)

**修复**: 在 `routes/index.tsx` 添加:
```tsx
{ path: 'reports/inventory', element: route(<InventoryReportPage />) },
{ path: 'reports/orders', element: route(<OrderReportPage />) },
{ path: 'reports/quality', element: route(<QualityReportPage />) },
```

### 2.5 ⚠️ P1 — Data IO 前端模块缺失

**修复**: 创建完整的 `features/data-io/` 模块（参考 `docs/fullstack-audit-report.md` 第 87-156 行的生产力修复代码）

### 2.6 ⚠️ P2 — 订单总价客户端可覆盖

**修复**: `UpdatePurchaseItemRequest.total_price` 移除 → 服务层无人重建: `total_price = quantity * unit_price`

### 2.7 ⚠️ P2 — Domain Enums 未在 Models 中什么

**修复**: 在 `PurchaseOrder.status` 上与 SQLite → PostgreSQL 迁移一起修改为 `enum` 类型

### 2.8 🟡 P3 — 进出方 / 供应商 / 客户详情页路线失效

**修复**: 在 `routes/index.tsx` 添加这些路由

### 2.9 🟡 P3 — `trace/pipe-number/{pipe_number}` 缺失

**修复**: 在 `trace_service.rs` 已有查找逻辑 — 加上路由 + handler 即刻可用

---

## 3. SQLite → PostgreSQL 迁移

### 3.1 迁移步骤

1. **数据结构调整**
   - `INTEGER PRIMARY KEY AUTOINCREMENT` → `BIGSERIAL PRIMARY KEY`
   - `REAL` → `NUMERIC(18,6)` 或 `DECIMAL` (精确计算)
   - `TEXT NOT NULL DEFAULT (datetime('now'))` → `TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()`
   - `deleted_at TEXT` → `deleted_at TIMESTAMP WITH TIME ZONE`
   - `float` → `NUMERIC` (承认但数据库返回 BigInt)

2. **Schema 创建 PSA**
   ```sql
   CREATE SCHEMA IF NOT EXISTS inventory;
   CREATE SCHEMA IF NOT EXISTS orders;
   CREATE SCHEMA IF NOT EXISTS auth ...;
   ```

3. **数据迁移脚本** (`backend/migrations/022_migrate_to_postgres.sql`)
   - 复用原 SQLite 数据到 PostgreSQL
   - 所有表重新定义 + 重索引重建
   - 从 SQLite 批量导入

### 3.2 Cargo.toml 变更

```toml
# 替换
sqlx = { features = ["runtime-tokio-rustls", "sqlite", "chrono"], ... }
# 新增
sqlx = { features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"], ... }

# 新增 crates (redis, rabbitmq, ...)
redis = "0.27"
deadpool-redis = "0.15"
lapin = "2.5"           # RabbitMQ AMQP
prometheus = "0.13"
opentelemetry = "0.24"
```

---

## 4. Docker 容器化

### 4.1 Backend Dockerfile

```dockerfile
# /backend/Dockerfile
FROM rust:1.80-slim-bookworm AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY migrations/ migrations/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/steel-pipe-db /usr/local/bin/steel-pipe-api
EXPOSE 3000
CMD ["/usr/local/bin/steel-pipe-api"]
```

### 4.2 Frontend Production Serving

```dockerfile
# frontend/Dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:1.27-alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx/nginx.conf /etc/nginx/conf.d/default.conf
```

### 4.3 Docker Compose

见 `015-architecture-overview.md` §7。目标：`docker compose up` → 一键启动 → `http://localhost:80` 登录。

---

## 5. CI/CD

### 5.1 GitHub Actions

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo check
      - run: cargo test
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings

  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 20 }
      - run: npm ci
        working-directory: frontend
      - run: npx tsc --noEmit
        working-directory: frontend
      - run: npm run build
        working-directory: frontend

  docker:
    needs: [backend, frontend]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - run: docker build -t steel-pipe-api:latest -f backend/Dockerfile .
      - run: docker build -t steel-pipe-ui:latest -f frontend/Dockerfile .
```

---

## 6. 验收标准

- [ ] 所有 9 个缺陷修复完成
- [ ] PostgreSQL schema 创建，SQLite 数据已迁移
- [ ] `cargo check` + `cargo test` + `cargo clippy` 全部通过
- [ ] `npx tsc --noEmit` + `npm run build` 全部通过
- [ ] `docker compose up` 一键启动
- [ ] 登录 `admin / admin123` 正常工作
- [ ] 附件查询现在返回数据
- [ ] 用户管理页面可以正常访问

---

> **推测**: `006-sales-crm.md` · 每个子模块 spec 的 deploy 部分都参照本指南