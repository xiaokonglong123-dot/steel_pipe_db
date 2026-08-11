# Ikari_Shinji — ERP

<div align="center">

> 通用 ERP（企业资源计划）系统。Rust + Axum 后端，Vue 3 + Element Plus 前端，SQLite 单文件存储。

![Rust](https://img.shields.io/badge/Rust-Axum_0.8-000000?style=flat-square&logo=rust&logoColor=white)
![Vue](https://img.shields.io/badge/Vue-3-4FC08D?style=flat-square&logo=vue.js&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-3-003B57?style=flat-square&logo=sqlite&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript&logoColor=white)
![Element Plus](https://img.shields.io/badge/Element_Plus-409EFF?style=flat-square&logo=element&logoColor=white)

![Tests](https://img.shields.io/badge/后端测试-121%20通过-brightgreen?style=flat-square)
![Build](https://img.shields.io/badge/前端构建-通过-brightgreen?style=flat-square)

</div>

---

## 概览

一套面向单厂、单实例部署的模块化 ERP，覆盖核心交易闭环：
商品（SKU 主数据）→ 库存（入出库 + 审计轨迹）→ 采购（采购订单）→ 销售（销售订单 + ATP 预留）→ 财务（科目/日记账/发票/付款）→ 报表 —— 全程由数据驱动审批流和 JWT+RBAC 鉴权守护。

本系统是 **erp-v2 重写时代**，由原 React 栈 ERP 重写而来（旧栈已归档至 `legacy/steel-pipe-react` 分支）。单厂、单实例部署，SQLite 作为唯一数据源——面向小团队、零基础设施运维。

---

## 技术栈

### 后端 — Rust (Axum 0.8)

| 层级      | 技术                                                            |
|-----------|-----------------------------------------------------------------|
| 框架      | Axum 0.8 with macros + multipart                                |
| ORM       | SQLx 0.8（SQLite，`sqlite` + `chrono` + `uuid` feature）         |
| 鉴权      | JWT（`jsonwebtoken` 9）+ Argon2id 密码哈希                       |
| 金额      | `rust_decimal::Decimal` —— 全链精度（DB 存 `TEXT`）              |
| 校验      | `validator` 0.19（derive）                                       |
| 日志      | `tracing` + `tracing-subscriber`（env-filter、json）             |
| Excel/CSV | `csv` 1.3 用于批量导入                                           |
| 中间件    | `tower-http`（CORS、trace、request-id），Cookie-based 鉴权        |
| 数据库    | SQLite3 单文件（`backend/data/erp.db`，WAL 模式）                |

### 前端 — Vue 3 + Element Plus

| 类别     | 库                                                                |
|----------|-------------------------------------------------------------------|
| UI 框架  | Vue 3 + Element Plus 5 + ECharts（line + bar 可视化）              |
| 状态     | Pinia（客户端）+ TanStack Vue Query（服务端状态）                  |
| HTTP     | 原生 `fetch` 封装（`api/client.ts`）                              |
| 路由     | Vue Router 4，带权限守卫                                           |
| 构建     | Vite 6 + `bun` 包管理器                                          |
| 类型     | TypeScript 5 + `vue-tsc`                                          |

---

## 快速开始

### 前置

| 工具    | 版本    | 备注                                |
|---------|---------|-------------------------------------|
| Rust    | 1.78+   | edition 2021                        |
| bun     | 1.x     | 替代 npm（npm 在本环境不可用）       |
| SQLite  | 3.35+   | 由 sqlx 自带                        |

### 后端

```bash
cd backend
cp .env.example .env       # 或自写：DATABASE_URL=sqlite://data/erp.db?mode=rwc
cargo run                  # 启动于 http://localhost:3000
```

后端 crate 名为 `erp-v2`。数据库是单个 SQLite3 文件（`data/erp.db`），无需外部 DB server。

### 前端

```bash
cd frontend
bun install                # 不要用 npm —— 依赖版本不一致
bun run dev                # 启动于 http://localhost:5173
```

打开 `http://localhost:5173`，使用如下凭证登录：

| 用户名 | 密码       |
|--------|------------|
| `admin`| `admin123` |

---

## 构建与校验

| 内容          | 命令                                            |
|---------------|-------------------------------------------------|
| 后端类型检查   | `cd backend && cargo check --all-targets`      |
| 后端测试       | `cd backend && cargo test --all`                |
| 前端类型检查   | `cd frontend && bunx tsc --noEmit`              |
| 前端构建       | `cd frontend && bun run build`                  |

- **后端**：121 个测试全绿
- **前端**：`bunx tsc --noEmit` + `bun run build` 全绿

---

## 模块

### Phase P0 —— 核心交易闭环（✅ 完成）

| 模块                                  | 描述                                                                     | 测试 |
|---------------------------------------|--------------------------------------------------------------------------|------|
| Auth + RBAC                           | JWT 登录/刷新/登出，实时查库权限校验，hash 存储 refresh token            | 4    |
| Catalog（商品）                        | SKU 主数据（sku/name/category/unit/spec），draft/active/disabled 生命周期 | 8    |
| Parties                               | 供应商 + 客户，软删                                                       | 9    |
| 仓库 + 库位 + 库存 + 日志             | 物化 `inventory`（余额）+ `inventory_logs`（事件审计轨迹）              | 22   |
| 采购订单                              | PO 生命周期（draft → submitted → approved/rejected），Decimal 金额       | 13   |
| 销售订单 + ATP 预留                   | SO + 基于 ATP（Available-To-Promise）的预留校验                          | 14   |
| Workflow（数据驱动，ERPNext 风格）     | `workflows` / `workflow_states` / `workflow_transitions` —— 新增节点纯 INSERT | 13   |
| 收货 + 发货（端到端）                  | 端到端入库/出库联动 PO/SO                                                | 9    |
| **E2E** PO + SO                       | 端到端 采购 → 库存 → 销售 闭环                                            | 2    |

### Phase P1 —— 财务 + 报表 + ATP（✅ 完成）

| 模块                          | 描述                                                              | 测试 |
|-------------------------------|-------------------------------------------------------------------|------|
| Finance                       | GL 科目、日记账、发票、付款、试算平衡                              | —    |
| Inventory Check 盘点          | 盘点 + 余额对账                                                    | 5    |
| ATP `available_qty`           | ATP 可用量独立端点 + service                                       | 1    |
| Reports                       | inventory_summary / inbound_outbound / sales_trend / finance_summary + CSV 导出 | —    |
| **P1 E2E**                    | 财务 + 盘点 + ATP 释放端到端                                       | 4    |

### Phase P2 —— 增强（✅ 完成）

| 模块                                | 描述                                                          | 测试 |
|-------------------------------------|---------------------------------------------------------------|------|
| CSV 商品导入                        | `POST /items/import`（multipart），逐行报告                  | 2    |
| Workflow 多级/条件                  | `amount_threshold` + `transition_with_amount`（按金额选路径）| 2    |
| 迁移校验                            | finance.threshold `012_workflow_threshold.sql`                | 1    |

---

## 数据模型

SQLite3 单文件（WAL 模式），12 个迁移：

```
001_auth_rbac.sql           — users / roles / role_permissions / operation_logs / refresh_tokens
002_catalog.sql             — items（SKU 主数据，含 draft/active/disabled）
003_parties.sql             — suppliers / customers
004_inventory.sql           — warehouses / locations / inventory / inventory_logs
005_purchasing.sql          — purchase_orders / purchase_order_items
006_sales.sql               — sales_orders / sales_order_items / reservations
007_finance.sql             — accounts / journal_entries / journal_lines / invoices / payments
008_workflow.sql            — workflows / workflow_states / workflow_transitions / workflow_instances / workflow_tasks
009_seed.sql                — admin/manager/finance 角色 + 11 个权限
010_warehouses.sql          — ALTER locations.warehouse_id + deleted_at（引入父表）
011_seed_workflows.sql      — PO/SO 演示 workflow + states + transitions
012_workflow_threshold.sql  — ALTER workflow_transitions.amount_threshold TEXT
```

完整性在应用层强制（TOCTOU 安全、事务化）。软删 `deleted_at`——记录不会被物理销毁。完整 schema 见 [detailed-design.md](./docs/detailed-design.md)。

---

## 关键设计决策

- **Money 全链 Decimal**：`rust_decimal::Decimal`，DB 存 `TEXT`，`Decimal::to_string()` 序列化。**不允许 SQL SUM over TEXT money 列**——报表金额聚合在应用层做。日期分组 REAL 聚合允许（ADR-002 例外）。
- **Inventory 双轨**：物化 `inventory` 表（余额）+ `inventory_logs` 事件日志（审计轨迹）。入库写 +OUT 检查余额。
- **审批流 data-driven**：`workflows` / `workflow_states` / `workflow_transitions`——新增节点不出代码，只需 INSERT 行。
- **金额阈值条件**：`workflow_transitions.amount_threshold TEXT`，`workflow_service::transition_with_amount(...)` 会优先选 threshold 满足的 transition；不满足则 fallback 走无阈值 transition。
- **RBAC 实时查库**：JWT 携带 `user_id` + `permissions`，但 `auth_middleware` 每个请求查库注入 fresh `AuthUser`（无权限陈旧问题）。
- **Spec Drift 处理**：已执行的迁移文件绝不修改（项目规则 #234）——`010_warehouses` / `011_seed_workflows` / `012_workflow_threshold` 都是叠加的新迁移，不是对已有迁移的修改。

---

## 项目结构

```
Ikari_Shinji/
├── backend/                                # erp-v2 crate（Rust Axum）
│   ├── src/
│   │   ├── main.rs                         # 入口，服务器启动
│   │   ├── lib.rs                          # 模块声明
│   │   ├── config.rs                       # 环境配置
│   │   ├── error.rs / response.rs          # AppError + ApiResponse/PaginatedResponse
│   │   ├── auth/                           # JWT 登录/刷新/登出、bootstrap admin
│   │   ├── http/                           # 按领域分组的 handler（purchase.rs、sales.rs...）
│   │   ├── services/                       # 业务逻辑（purchase_service.rs、workflow_service.rs...）
│   │   ├── repos/                          # sqlx 仓储
│   │   ├── middleware/                     # auth + rbac 中间件
│   │   └── domain/                         # 领域枚举、校验辅助
│   ├── tests/                              # 16 个集成测试文件（共 121 测试）
│   ├── migrations/                         # 12 个 SQLx 迁移
│   ├── Cargo.toml / Cargo.lock / .env.example / rust-toolchain.toml
│   └── data/erp.db                         # SQLite3（gitignored，自动创建）
├── frontend/
│   ├── src/
│   │   ├── main.ts                         # Vue app + Pinia + Router + Vue Query
│   │   ├── App.vue / router/               # 路由 + 权限守卫
│   │   ├── api/                            # fetch client + queryClient + 各领域 api
│   │   ├── views/                          # 各领域页面（auth/items/purchases/...）
│   │   ├── stores/                         # Pinia stores
│   │   ├── components/                     # 共享 Element Plus 组件
│   │   └── styles/                         # 全局样式 + el-plus 主题覆盖
│   ├── package.json / bun.lock / vite.config.ts / tsconfig.json / DESIGN.md
├── docs/                                   # PRD、detailed-design、frontend-design、tasks
│   └── legacy/                             # 重写前（React 栈）设计文档归档
├── specs/                                  # 统一术语（ubiquitous language）
├── .github/workflows/ci.yml                # CI：cargo check + test + bun tsc + build
├── AGENTS.md                               # 权威项目索引
├── README.md / README_zh.md / CHANGELOG.md / CONTRIBUTING.md / LICENSE
└── .local-only-docs/                       #（gitignored）本地决策文档
```

---

## API 概览

所有端点在 `/api/v1/` 下（少数按 AGENTS.md 注释有短路径）。每个响应统一格式：

```json
{ "success": true, "request_id": "req_...", "data": { ... } }
```

分页响应携带 `meta: { total, page, page_size, total_pages }`。错误响应 `success: false`，仍带 `request_id`。完整路由表见 [detailed-design.md](./docs/detailed-design.md)。

### 鉴权端点

| 端点                  | 鉴权 | 用途                            |
|----------------------|:----:|--------------------------------|
| `POST /auth/login`   | 无   | 获取 access + refresh token（cookie）|
| `POST /auth/refresh` | tkn  | 轮换 refresh token             |
| `POST /auth/logout`  | tkn  | 撤销所有 refresh token          |
| `GET  /auth/me`      | tkn  | 当前用户 + 权限                 |

---

## 安全

- **密码**：Argon2id，推荐参数（`m=19456, t=2, p=1`）
- **鉴权**：无状态 JWT access token（HS256）。**Refresh token 服务端存储**——SHA-256 哈希存于 `refresh_tokens` 表，每次 `/auth/refresh` 轮换，`/auth/logout` 撤销该用户所有 refresh token（级联所有 session）。
- **RBAC**：`admin`/`manager`/`warehouse`/`finance` 等角色——中间件每个请求查库强制（无权限陈旧风险）。
- **金额**：`rust_decimal::Decimal` 端到端，业务逻辑无 f64。
- **CSV/XLSX 导出**：电子表格公式前缀（`=`、`+`、`-`、`@`）转义，导出的用户可控值始终以文本打开。
- **审计**：`inventory_logs`（物料移动轨迹）+ `operation_logs`（管理员操作审计）。

---

## 历史

| 分支                          | 时代                              | 头 SHA                                                          |
|-------------------------------|-----------------------------------|-----------------------------------------------------------------|
| `main`                        | erp-v2（当前）                    | `c6a0b62`（提升 + 日期修复）— `9570a29`（origin/main）         |
| `legacy/steel-pipe-react`     | 重写前 React 19 + Antd 栈         | `05cbf0d`（51 个旧栈演化提交，erp-v2 之前）                     |
| `legacy/react-phase1-4`       | 远端那段 React 栈 Phase 1-4 实现  | `ccedabe`（force-push 前从 origin/main 保留）                  |

重写前（`React 19 + Ant Design 5 + npm`）栈可通过 checkout 任意 legacy 分支完整恢复。

---

## 贡献

详见 [CONTRIBUTING.md](./CONTRIBUTING.md) 与 [AGENTS.md](./AGENTS.md)（权威项目索引）。原子提交、描述性信息、`main` 分支无协调不准 force-push。`docs/` 下设计文档是真相之源——架构变更时同步更新。

---

## CI

`.github/workflows/ci.yml` 在每次 push / PR 到 `main` 时运行：

- **后端**：`cargo check --all-targets` + `cargo test --all`
- **前端**：`bun install --frozen-lockfile` + `bunx tsc --noEmit` + `bun run build`

---

## 许可证

[GNU General Public License v2](./LICENSE)
