# 000 — ERP: 审计修复 & 基础设施加固 (Phase 0)

> **版本**: v1.0
> **日期**: 2026-08-02
> **状态**: Draft
> **优先级**: P0 — 地基加固

---

## 目标

在开始 ERP 模块开发之前，修复当前代码库中所有已知缺陷，完成迁移脚本的 SQLite 重写，并通过 Docker Compose 支持一键启动。

---

## 2. 缺陷修复清单

### 2.1 ⚠️ P0 — UserManagementPage 无路由注册

**问题**: `frontend/src/features/auth/pages/UserManagementPage.tsx` 331 行功能完整，但无路由/no 菜单

**修复**:

1. 在 `routes/index.tsx` 添加：

```tsx
{ path: 'settings/users', element: route(<UserManagementPage />), handle: { roles: ['admin'] } }
```
1. 在 `MainLayout.tsx` 添加菜单项

### 2.2 ⚠️ P1 — 审批评论死代码

**问题**: `ApproveRequest.reason` 字段但从不用使用

**修复**:

1. `InventoryService::approve_inbound` 签名加 `reason: Option<&str>`, `user_id: i64`
2. 将 reason 写入 operation_log、并于 inbound_records 增加 `approval_reason` 列

### 2.3 ⚠️ P1 — 报表子页面路由缺失 (2 个)

**修复**: 在 `routes/index.tsx` 添加:

```tsx
{ path: 'reports/inventory', element: route(<InventoryReportPage />) },
{ path: 'reports/orders', element: route(<OrderReportPage />) },
```

### 2.4 ⚠️ P1 — Data IO 前端模块缺失

**修复**: 创建完整的 `features/data-io/` 模块（参考 `docs/fullstack-audit-report.md` 第 87-156 行的生产力修复代码）

### 2.5 ⚠️ P2 — 订单总价客户端可覆盖

**修复**: `UpdatePurchaseItemRequest.total_price` 移除 → 服务层无人重建: `total_price = quantity * unit_price`

### 2.6 ⚠️ P2 — Domain Enums 未在 Models 中什么

**修复**: 在 `PurchaseOrder.status` 上与迁移脚本 SQLite 重写一起修改为 `enum` 类型

### 2.7 🟡 P3 — 进出方 / 供应商 / 客户详情页路线失效

**修复**: 在 `routes/index.tsx` 添加这些路由

> **注**: 原「质量附件查询参数不匹配」「报表质量子页面」「按管号追溯」等缺陷随钢管专属模块（quality、trace）一并废弃，见 `009-pipe-threading.md` 归档说明。

---

## 3. 数据库迁移策略 (SQLite)

### 3.1 迁移目标

数据库为 **SQLite3**，连接串 `sqlite://data/erp.db?mode=rwc`（sqlx 0.8 `sqlite` feature），单文件、零外部数据库依赖。

### 3.2 迁移步骤

1. **37 个遗留迁移文件重写为 SQLite 语法**
   - `INTEGER PRIMARY KEY AUTOINCREMENT` 保留（SQLite 原生）
   - `TEXT NOT NULL DEFAULT (datetime('now'))` 保留
   - 删除原数据库专属类型：`BIGSERIAL`、`TIMESTAMPTZ`、`JSONB`、`INET`、`NUMERIC(p,s)` → 统一 `INTEGER`/`TEXT`/`REAL`
   - 删除钢管行业专属表（管材、标签、质检证书、参考数据等；完整清单见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`）

2. **商品化改造**
   - 新增 `items` 表（sku/名称/分类/单位/规格），作为全系统唯一商品实体
   - 库存、订单、合同等引用 `item_id`（原管材表引用迁移至 `items.id`）

3. **数据迁移脚本** (`backend/migrations/`)
   - 复用原 SQLite 数据（无跨库导入）
   - 所有表重新定义 + 重索引重建

### 3.3 Cargo.toml 约定

```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "chrono"] }
```

不引入外部数据库驱动与中间件依赖。

---

## 4. Docker 容器化

### 4.1 Backend Dockerfile

```dockerfile
# /backend/Dockerfile
FROM rust:1.80-slim-bookworm AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/erp-server /usr/local/bin/erp-server
EXPOSE 3000
CMD ["/usr/local/bin/erp-server"]
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

见 `015-architecture-overview.md` §7。目标：`docker compose up` → 一键启动 → `http://localhost:80` 登录。数据库为挂载卷中的 SQLite 单文件（`data/erp.db`），无需数据库容器。

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
      - run: docker build -t erp-server:latest -f backend/Dockerfile .
      - run: docker build -t erp-ui:latest -f frontend/Dockerfile .
```

---

## 6. 验收标准

- [ ] 所有缺陷修复完成（不含已废弃的钢管专属缺陷）
- [ ] 37 个迁移文件重写为 SQLite 语法，钢管表已删除，`items` 商品表就绪
- [ ] `cargo check` + `cargo test` + `cargo clippy` 全部通过
- [ ] `npx tsc --noEmit` + `npm run build` 全部通过
- [ ] `docker compose up` 一键启动
- [ ] 登录 `admin / admin123` 正常工作
- [ ] 用户管理页面可以正常访问

---

> **推测**: `006-sales-crm.md` · 每个子模块 spec 的 deploy 部分都参照本指南
