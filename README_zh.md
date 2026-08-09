<div align="center">

> **🤖 本仓库中的所有代码均由 AI 生成** — 从架构设计到每一行代码，完全由大语言模型生成，用于技术演示和能力验证。

</div>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/ERP-1f2937?style=flat-square&logo=rust&logoColor=white">
  <img alt="ERP" src="https://img.shields.io/badge/ERP-1f2937?style=flat-square&logo=rust&logoColor=white">
</picture>

# ERP — 通用企业资源计划系统 (Enterprise Resource Planning System)

> 通用 ERP：商品/SKU 库存、采购、销售、财务、人力资源、制造、项目、固定资产、审批流、通知与 BI 分析。Rust 后端，React 前端。名副其实。
>
> 历史沿革：本系统由钢管行业系统重构而来，已重构为通用 ERP。

![Rust](https://img.shields.io/badge/Rust-Axum-000000?style=flat-square&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-003B57?style=flat-square&logo=sqlite&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript&logoColor=white)
![Ant Design](https://img.shields.io/badge/Ant_Design-5-1677FF?style=flat-square&logo=antdesign&logoColor=white)

---

## 🚀 快速开始

### 前置条件

| 工具  | 版本      |
|-------|-----------|
| Rust  | 1.78+（edition 2021） |
| Node  | 20+       |
| npm   | 10+       |

### 后端

```bash
cd backend
cp .env.example .env    # 或自行配置：DATABASE_URL=sqlite://data/erp.db?mode=rwc
cargo run               # 启动于 http://localhost:3000
```

后端 crate 名为 `erp-server`（代码阶段的实施目标）。数据库为单个 SQLite3 文件（`data/erp.db`）— 无需外部数据库服务器。

### 前端

```bash
cd frontend
npm install
npm run dev             # 启动于 http://localhost:5173
```

打开 `http://localhost:5173`，使用以下账号登录：

| 用户名   | 密码       |
|----------|-----------|
| `admin`  | `admin123` |

---

## 🏗 技术栈

### 后端 — Rust (Axum 0.8)

| 层级         | 技术                                                |
|-------------|-----------------------------------------------------|
| 框架         | Axum 0.8（macros + multipart 特性）                  |
| ORM         | SQLx 0.8（SQLite3，`sqlite` 特性）                    |
| 认证         | JWT（jsonwebtoken 9）+ Argon2 密码哈希               |
| 校验         | Validator 0.19（derive 特性）                        |
| 日志         | Tracing + tracing-subscriber（env-filter, json）      |
| Excel/CSV   | calamine（导入）、rust_xlsxwriter（导出）、csv         |
| 中间件       | tower-http（CORS、trace、request-id）                 |
| 数据库       | SQLite3，`sqlite://data/erp.db?mode=rwc`（WAL 模式）   |

**架构：** Handler → Service → Repository → Domain。不使用 AppState — 数据库连接池通过 `Extension<SqlitePool>` 注入，认证密钥使用脱敏的 `JwtSecret` 扩展。

### 前端 — React 19

| 类别          | 库                                                |
|--------------|---------------------------------------------------|
| UI 框架       | React 19 + Ant Design 5 + @ant-design/icons        |
| 路由          | react-router-dom 7                                 |
| 状态管理       | Zustand 5（客户端状态）+ TanStack Query 5（服务端状态） |
| HTTP 客户端   | 原生 `fetch` 封装（`src/api/client.ts`）           |
| 国际化        | react-i18next + i18next（中英文按模块划分）           |
| 构建工具       | Vite 6                                             |
| 类型安全       | TypeScript 5 + Zod 3                               |

---

## 📚 功能模块

### 阶段一 — 核心（P0）

| 模块           | 描述                                           |
|---------------|------------------------------------------------|
| 认证/RBAC     | JWT 登录/刷新/登出，RBAC（角色/权限/部门/租户）     |
| 商品与库存     | 商品 (Item) + SKU 主数据（sku/名称/分类/单位/规格），库位库存、ATP 预留、入库/出库、盘点 |
| 审批流         | 审批引擎：审批流定义、审批流实例、审批任务         |

### 阶段二 — 业务（P1）

| 模块           | 描述                                           |
|---------------|------------------------------------------------|
| 供应商         | 供应商管理、资质、供应商评分                      |
| 客户           | 客户管理、客户信用                               |
| 采购           | 采购订单管理、入库审批流程                         |
| 销售           | 销售订单、出库、自动 ATP 检查                     |
| 采购管理       | 采购申请、采购报价、采购收货                       |
| 人力资源       | 员工、考勤、薪资、劳动合同                         |
| 财务           | 会计科目、日记账、发票、付款、试算平衡              |
| 制造           | BOM、工单、质检 (Inspection)、不合格品单 (NCR)      |
| 数据导入导出   | Excel/CSV 批量导入与导出                          |

### 阶段三 — 企业级（P2）

| 模块           | 描述                                           |
|---------------|------------------------------------------------|
| 合同           | 销售/采购合同、付款里程碑                          |
| 项目           | 项目、WBS、预算                                  |
| 固定资产       | 资产登记、直线法折旧、资产处置                      |
| 通知           | 通知收件箱、模板、偏好设置                         |
| 门户           | 客户/供应商门户账户、门户 JWT、采购订单确认/销售订单回执 |
| BI 分析        | 销售趋势、库存价值、财务汇总、供应商绩效            |
| 国际化         | 中英文切换（按功能模块划分命名空间）                |

---

## 🗄 数据模型

单个 SQLite3 文件（WAL 模式），完整性在应用层保障。下表为迁移重写后的目标表结构（旧版 37 个迁移将重写为 SQLite 语法，删除管材专属表）：

```
users                  → 系统用户（RBAC）                            [001]
roles / permissions / departments / tenants → RBAC 结构               [001]
items                  → 商品/SKU 主数据（sku、名称、分类、单位、规格） [002]
warehouses / locations → 仓库与库位层级                               [002]
inventory              → 商品在库位的库存                              [002]
inbound_records / inbound_items → 入库单头 + 行项目                   [002]
outbound_records / outbound_items → 出库单头 + 行项目                  [002]
inventory_logs         → 商品流转审计日志                              [002]
inventory_check_records / inventory_check_items → 盘点                [002]
reservations           → ATP 预留（销售订单 / 工单）                   [002]
suppliers              → 供应商主数据                                  [003]
customers              → 客户主数据                                    [003]
purchase_orders / purchase_order_items → 采购订单头 + 行项目           [003]
sales_orders / sales_order_items → 销售订单头 + 行项目                 [003]
quotes                 → 采购报价 / 销售报价                            [003]
shipments              → 销售发货确认                                  [003]
customer_credit        → 客户信用使用情况                               [003]
contracts / contract_items / contract_payments → 合同头、行项目、付款里程碑 [004]
accounts / journal_entries / invoices / payments → 会计科目、日记账、发票、付款 [005]
employees / attendance / salaries / labor_contracts → 人力资源记录      [005]
requisitions / receipts / scorecards → 采购记录（申请/收货/评分）        [005]
boms / work_orders / inspections / ncrs → 制造记录                      [006]
projects / wbs / budgets → 项目管理                                     [007]
fixed_assets / depreciation / disposals → 固定资产全生命周期            [007]
notifications / templates / preferences → 通知平台                      [008]
portal_accounts       → 门户账户（客户/供应商）                         [008]
workflow_definitions / workflow_instances / workflow_tasks → 审批引擎   [009]
operation_logs        → 系统操作审计日志                                [010]
refresh_tokens        → 服务端哈希刷新令牌会话                          [011]
```

所有时间戳均为 ISO 8601 字符串。通过 `deleted_at` 实现软删除 — 数据永不真正消亡。

---

## 🧪 开发

```bash
# 后端
cd backend && cargo check           # 仅类型检查（比完整构建快得多）
cargo test                           # 运行测试
cargo build                          # Debug 构建
cargo build --release                # 发布构建

# 前端
cd frontend && npx tsc --noEmit     # TypeScript 类型检查
npm run build                        # 生产构建
npm run lint                         # ESLint 检查
```

---

## 🔐 安全

- **密码**：Argon2id，推荐参数（`m=19456, t=2, p=1`）
- **认证**：无状态 JWT（HS256）访问令牌。刷新令牌存储在服务端（`refresh_tokens` 表中 SHA-256 哈希），每次 `/auth/refresh` 轮换。过期或被撤销的刷新令牌会被拒绝，`/auth/logout` 撤销该用户所有刷新令牌，防止继续续期。
- **RBAC**：`admin`、`warehouse`、`qc`、`sales` 等角色 — 通过中间件强制执行
- **限流**：认证端点（登录/刷新）按 IP 限流（中间件）
- **数据导入导出**：批量导入仅管理员可用；导出限 `admin`/`warehouse`/`sales`；操作日志仅管理员可见。CSV/XLSX 导出会转义电子表格公式前缀（`=`、`+`、`-`、`@`），确保导出的用户可控内容以文本打开。
- **数据**：所有业务实体软删除，通过 `inventory_logs` 和 `operation_logs` 审计追踪

---

## 📁 项目结构

```
erp/
├── backend/                       # erp-server crate（Rust Axum）
│   ├── src/
│   │   ├── main.rs           # 入口，服务器启动
│   │   ├── lib.rs             # 模块声明
│   │   ├── router.rs          # 路由定义（约 190 个路由，约 170 个唯一路径）
│   │   ├── config.rs          # 环境配置（DATABASE_URL=sqlite://data/erp.db?mode=rwc）
│   │   ├── error.rs           # AppError 与 ApiResponse 映射；ApiErrorResponse 包含 success + request_id
│   │   ├── response.rs        # ApiResponse<T> / PaginatedResponse<T> / Meta 结构体，含 request_id（uuid v4）
│   │   ├── domain/            # 领域枚举和常量
│   │   ├── dto/               # 请求/响应 DTO
│   │   ├── models/            # 数据库模型
│   │   ├── items/ inventory/ orders/ contracts/ parties/ reports/ data_io/
│   │   │   # 资源域模块（handlers.rs + services.rs + repos.rs）
│   │   ├── auth/ workflow/ hr/ finance/ procurement/ sales_crm/ inventory_atp/ manufacturing/
│   │   │   project/ assets/ notification/ portal/ bi/   # 业务域模块
│   │   ├── health.rs utils.rs operation_log.rs macros.rs   # 顶层单文件
│   │   └── middleware/        # 认证 + RBAC 中间件（含 AuthenticatedUser 提取器）
│   ├── migrations/            # SQLx 迁移文件（重写为 SQLite 语法）
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── api/               # 原生 fetch 封装（src/api/client.ts）
│   │   ├── features/          # 按模块划分：auth、items、inventory、purchases、sales、workflow、hr、finance...
│   │   ├── layouts/           # MainLayout 含侧边栏
│   │   ├── stores/            # Zustand 状态仓库
│   │   ├── routes/            # react-router 路由配置
│   │   ├── shared/            # 共享组件和 hooks
│   │   ├── i18n/              # 中英文语言包
│   │   ├── types/             # 全局 TypeScript 类型
│   │   └── styles/            # 全局样式
│   ├── package.json
│   └── vite.config.ts
├── specs/                      # 统一术语表（术语规范）
│   └── UBIQUITOUS_LANGUAGE_LATEST.md
├── docs/                      # 设计与运维文档
│   ├── requirements.en.md     # 产品需求文档（英文）
│   ├── detailed-design.en.md  # 架构 + 数据库 + API 设计（英文）
│   ├── frontend-design.en.md  # 前端组件树 & 路由（英文）
│   ├── 需求文档.md             # 产品需求文档（中文）
│   ├── 详细设计文档.md         # 架构 + 数据库 + API 设计（中文）
│   ├── 前端设计文档.md         # 前端设计（中文）
│   ├── deployment.md          # 部署指南（Nginx、Docker、备份）
│   ├── troubleshooting.md     # 故障排查（数据库锁定、JWT、CORS）
│   └── tasks/                 # 任务分解
└── .github/workflows/
    └── ci.yml                 # CI：cargo check + tsc + vite build
```

---

## 🌐 API 概览

所有端点位于 `/api/v1/` 下：

| 分组           | 前缀                  | 需要认证 |
|---------------|----------------------|:---:|
| 认证           | `/auth/*`            | 部分需要 |
| 用户           | `/users/*`           | 仅管理员 |
| 商品           | `/items/*`           | 是 |
| 库存           | `/inventory/*`       | 是 |
| 供应商         | `/suppliers/*`       | 是 |
| 客户           | `/customers/*`       | 是 |
| 采购           | `/purchase-orders/*` | 是 |
| 销售           | `/sales-orders/*`    | 是 |
| 合同           | `/contracts/*`       | 是 |
| 报表           | `/reports/*`         | 是 |
| 数据导入导出   | `/data-io/*`         | 是 |
| 审批流         | `/workflow/*`        | 是 |
| 人力资源       | `/hr/*`              | 是 |
| 财务           | `/finance/*`         | 是 |
| 采购管理       | `/procurement/*`     | 是 |
| 制造           | `/manufacturing/*`   | 是 |
| 项目           | `/projects/*`        | 是 |
| 固定资产       | `/assets/*`          | 是 |
| 通知           | `/notifications/*`   | 是 |
| 门户           | `/portal/*`          | 是 |
| BI 分析        | `/bi/*`              | 是 |

每个响应遵循统一格式：

```json
{ "success": true, "request_id": "req_...", "data": { ... } }
```

分页响应额外包含 `meta: { total, page, page_size, total_pages }`。错误响应将 `success` 置为 `false`，同样包含 `request_id`。

---

## 🔑 RBAC 权限矩阵

| API 分组           | admin | warehouse | qc  | sales |
|-------------------|:-----:|:---------:|:---:|:-----:|
| 用户（写入）        | ✅    | ❌        | ❌  | ❌    |
| 商品（写入）        | ✅    | ✅        | ❌  | ❌    |
| 入库/出库（写入）    | ✅    | ✅        | ❌  | ❌    |
| 质检（写入）        | ✅    | ❌        | ✅  | ❌    |
| 销售订单（写入）     | ✅    | ❌        | ❌  | ✅    |
| 采购订单（写入）     | ✅    | ✅        | ❌  | ✅    |
| 供应商/客户（写入）  | ✅    | ✅        | ❌  | ✅    |
| 合同（写入）        | ✅    | ✅        | ❌  | ✅    |
| 数据导入           | ✅    | ❌        | ❌  | ❌    |
| 数据导出           | ✅    | ✅        | ❌  | ✅    |
| 数据导入导出操作日志 | ✅    | ❌        | ❌  | ❌    |
| 所有读取端点       | ✅    | ✅        | ✅  | ✅    |

---

## 🧭 设计文档

设计文档位于 [`docs/`](./docs/) 目录：

| 文档                    | 内容                                    |
|------------------------|-----------------------------------------|
| `需求文档.md`            | 完整产品需求：功能、路线图                |
| `详细设计文档.md`        | 架构、数据库设计、REST API、安全方案       |
| `前端设计文档.md`        | 组件树、路由、状态管理、国际化、主题        |
| `detailed-design.en.md` | 详细设计文档英文版                        |
| `requirements.en.md`    | 需求文档英文版                           |
| `frontend-design.en.md` | 前端设计文档英文版                        |
| `deployment.md`         | 部署指南：生产配置、Nginx、Docker、备份    |
| `troubleshooting.md`    | 故障排查：数据库锁定、JWT、CORS、迁移      |
| `tasks/progress.md`     | 主任务追踪                               |

另见：[`CONTRIBUTING.md`](../CONTRIBUTING.md) · [`CHANGELOG.md`](../CHANGELOG.md)

---

## 📄 许可证

[GNU General Public License v2](./LICENSE)
