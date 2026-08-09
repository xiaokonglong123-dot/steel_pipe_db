# 015 — ERP: 总体架构设计

> **版本**: v2.0（重构）
> **日期**: 2026-08-02
> **状态**: Draft
> **作者**: Sisyphus / Ikari Shinji
> **依赖**: 通用 ERP 重构（原钢管行业系统重构而来）
> **下一级文档**: `016-data-schema.md`, `017-frontend-guide.md`

---

## 目录

1. [架构原则](#1-架构原则)
2. [系统拓扑](#2-系统拓扑)
3. [应用架构](#3-应用架构)
4. [数据库架构](#4-数据库架构)
5. [模块划分与依赖](#5-模块划分与依赖)
6. [跨模块通信](#6-跨模块通信)
7. [部署架构](#7-部署架构)
8. [安全架构](#8-安全架构)
9. [可观测性](#9-可观测性)
10. [编码约定](#10-编码约定)
11. [迁移策略](#11-迁移策略)

---

## 1. 架构原则

| 原则 | 说明 |
| ------ | ------ |
| **Module Monolith First** | 单一 Rust 进程（crate `erp-server`），内部按业务域模块化，不引入分布式复杂性直到实际需要 |
| **单数据库，表名前缀分区** | 所有模块共享同一个 SQLite3 数据库，按表名前缀分区（`inventory_*`, `finance_*`, `hr_*`, `manufacturing_*`, ...） |
| **同步事务，异步事件** | 同模块内操作保持 ACID 联动。跨模块通过事件回调/消息解耦 |
| **Failure-First Design** | 每个操作要发生错误时提供可诊断的错误码。错误码按模块分配范围 |
| **Tenant-Ready** | 单实例支持多公司/多部门，数据结构通过 `tenant_id`/`company_id` 隔离 |
| **Simple Deployment** | SQLite3 单文件（`sqlite://data/erp.db?mode=rwc`）零外部依赖，Docker Compose 可选 |
| **Frontend Features** | 前端按业务模块居中，每个模块有独立的 API/hooks/pages/i18n |
| **Audit-First** | 所有关键操作记录 audit log。禁止对数据库中任何行物理删除（始终 `deleted_at`） |

---

## 2. 系统拓扑

```
                     ┌──────────────┐
                     │   Browser    │
                     │ React 19 SPA │
                     └──────┬───────┘
                            │ HTTPS
                     ┌──────▼───────┐
                     │    Nginx      │
                     │ Reverse Proxy │
                     └──────┬───────┘
                            │
                     ┌──────▼───────┐
                     │  Rust Axum   │
                     │  Monolith    │
                     │  (erp-server)│
                     │  Port 3000  │
                     └──────┬───────┘
                            │
                     ┌──────▼───────┐
                     │   SQLite3    │
                     │ data/erp.db  │
                     │ (单文件, WAL)│
                     └──────────────┘
```

### 组件说明

| 组件 | 角色 | 部署方式 |
| ------ | ------ | ------------ |
| **Nginx** | TLS 终端 + 静态 SPA + 反代 API | Docker Container (可选) |
| **Rust Axum Monolith (erp-server)** | 所有的 REST 架构 | Docker Container (可选) |
| **SQLite3** | 主数据库，单文件 `data/erp.db`，WAL 模式 | 进程内 / 挂载卷 |

> 无外部数据库服务器、无 Redis/RabbitMQ 必需依赖（重构后去除）。

---

## 3. 应用架构

### 3.1 Rust Crate 结构

```
backend/                    ← crate: erp-server
├── Cargo.toml
├── migrations/             ← 37 个 SQLite 迁移（重写后删除钢管表）
└── src/
    ├── main.rs             ← Entry: tracing, DB pool, migrate, start server
    ├── lib.rs              ← Module declarations
    ├── router.rs           ← ~70 endpoints 全部路由
    ├── config.rs           ← Env-based config (DATABASE_URL=sqlite://data/erp.db?mode=rwc)
    ├── error.rs            ← AppError + numeric error codes
    ├── response.rs         ← ApiResponse<T>, PaginatedResponse<T>
    ├── middleware/         ← auth.rs + rbac.rs + rate_limit.rs
    ├── auth/               ← RBAC: repos + services + handlers
    ├── workflow/           ← 审批引擎: definitions/instances/tasks
    ├── hr/                 ← 员工/考勤/薪资/劳动合同
    ├── finance/            ← 科目/日记账/发票/付款/试算平衡
    ├── procurement/        ← 采购申请/收货/供应商报价/评分
    ├── sales_crm/          ← 发货/客户报价/客户信用
    ├── inventory_atp/      ← 商品/库存/预留/调拨/盘点
    ├── manufacturing/      ← BOM/工单/质检/不合格品单
    ├── project/            ← 项目/WBS/预算
    ├── assets/             ← 固定资产/直线折旧/处置
    ├── notification/       ← 通知/模板/偏好
    ├── portal/             ← 门户账户/外部 JWT
    ├── bi/                 ← 销售趋势/库存价值/财务汇总/供应商绩效
    ├── handlers/           ← 每实体一个文件（薄层）
    ├── services/           ← 业务逻辑（unit struct 静态方法）
    ├── repositories/       ← 纯 SQL，软删除感知
    ├── models/             ← DB row structs (sqlx::FromRow)
    ├── dto/                ← 请求/响应类型
    └── domain/             ← 枚举/领域类型
```

每个业务模块有四层：

```
inventory_atp/
├── mod.rs
├── repos.rs                ← 纯 SQL 查询
├── services.rs             ← 业务逻辑（unit struct + 静态方法）
└── handlers.rs             ← Axum 路由 handler（薄层）
```

### 3.2 Config 设定

```rust
// src/config.rs — 环境变量配置
pub struct Config {
    pub database_url: String,   // sqlite://data/erp.db?mode=rwc
    pub jwt_secret: String,
    pub admin_username: String,
    pub admin_password: String,
}
```

### 3.3 错误码（Domain-prefixed, 5-digit）

```rust
pub enum AppErrorDomain {
    General,       // 100xx
    Auth,          // 110xx
    Item,          // 120xx
    Inventory,     // 130xx
    Order,         // 140xx
    Inspection,    // 150xx
    Supplier,      // 160xx
    Customer,      // 170xx
    DataIO,        // 180xx
    Finance,       // 190xx
    HR,            // 200xx
    Manufacturing, // 210xx
    Projects,      // 220xx
    Assets,        // 230xx
    Workflow,      // 240xx
    Notification,  // 250xx
    Security,      // 260xx
    Db,            // 500xx
}
```

---

## 4. 数据库架构

### 4.1 SQLite 单文件 + 表名前缀分区

数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`）。每个业务域通过**表名前缀**分区，跨域访问只能通过 Repository 接口——绝不让一个模块直接触碰另一个模块的表。

```
SQLite: data/erp.db
├── users, roles, role_permissions, tenants, departments, audit_log
├── items                     ← 商品 master（sku/名称/分类/单位/规格）
├── locations, warehouses
├── inbound_records, inbound_items
├── outbound_records, outbound_items
├── inventory_logs, inventory_check_records, inventory_check_items
├── atp_slots                 ← Available-To-Promise slot
├── suppliers, customers
├── purchase_orders, purchase_order_items
├── sales_orders, sales_order_items
├── contracts, contract_items, contract_payments
├── purchase_requisitions, supplier_quotes, po_receipts
├── sales_quotes, shipments
├── finance_chart_of_accounts, finance_journal_entries, finance_journal_entry_details
├── finance_invoices, finance_payments
├── hr_employees, hr_departments, hr_attendances, hr_salaries, hr_contracts
├── manufacturing_boms, manufacturing_bom_items
├── manufacturing_work_orders, manufacturing_work_order_steps
├── manufacturing_routing_ops
├── manufacturing_quality_inspections, manufacturing_ncr_outputs
├── manufacturing_equipment_register
├── projects, wbs_elements, project_transactions
├── assets_fixed_assets, assets_depreciation_entries
├── workflow_definitions, workflow_instances, approval_nodes, workflow_delegations
├── notification_templates, notifications, notification_user_preferences
├── portal_accounts
├── data_io_import_records, data_io_export_tasks
└── operation_logs, refresh_tokens
```

### 4.2 关键数据库规则

- **时间列**：`TEXT NOT NULL DEFAULT (datetime('now'))`（SQLite 约定）
- **软删除**：禁止物理删除！始终用 `deleted_at` 列
- **FK 约束**：应用层保障完整性；SQLite 外键可配置
- **迁移**：37 个遗留迁移文件重写为 SQLite 语法，删除钢管行业专属表（管材、标签、质检证书、参考数据等；完整清单见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`），新增 `items` 商品表
- **商品化**：所有库存/订单/合同/工单引用 `items.id`（SKU 唯一编码）

---

## 5. 模块划分 & 依赖

### 5.1 Capability Map

```
                         ┌────────────────┐
                         │                │
                         │   Business     │
                         │   Intelligence │
                         └───────┬────────┘
                                 │ reads from ALL modules
┌───────────┐  ┌───────────┐  ┌──▼────────┐  ┌──────────┐  ┌────────────┐
│  Projects  │  │  Contracts │  │ Inventory  │  │ Sales/CRM │  │ Procurement│
│   (WBS)   │  │  (orders)  │  │  & ATP    │  │  (orders) │  │ (requisit.)│
└─────┬─────┘  └─────┬─────┘  └─────┬─────┘  └─────┬────┘  └──────┬─────┘
      │              │              │              │             │
      └──────┬───────┴──────┬───────┴──────┬───────┘             │
             │              │              │                      │
       ┌─────▼─────┐ ┌──────▼──────┐ ┌─────▼─────┐              │
       │  Workflow  │ │  Finance    │ │    HR     │              │
       │  (engine)  │ │  (GL/AR/AP)│ │           │              │
       └───────────┘ └─────────────┘ └───────────┘              │
                                                                │
 ┌──────────────────────────────────────────────────────────────▼────────────────┐
 │                            Core /  Cross-Cutting                               │
 │          Auth · Config · Error · Tracing · Common DB · SqlitePool          │
 └───────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 依赖

| 模块 | 依赖 |
| ------- | ------ |
| `auth` | core |
| `inventory_atp` | core, auth |
| `procurement` | core, inventory, workflow, finance |
| `sales_crm` | core, inventory, workflow, finance |
| `contracts` | core, customers, suppliers |
| `manufacturing` | core, inventory, workflow |
| `project` | core, procurement, sales_crm |
| `assets` | core, finance |
| `workflow` | core, auth |
| `notification` | core, auth, workflow |
| `portal` | core, procurement, sales_crm |
| `bi` | core, 所有模块（只读） |

---

## 6. 跨模块通信

### 6.1 事件流

**Example 采购订单 approved → inventory now awaits 预计到货:**

1. 采购服务批准采购订单
2. Service update `purchase_orders.status = 'approved'`
3. 触发事件回调：`orders.purchase.approved.purchase_order_id=42`
4. 库存模块订阅该事件 → 创建 `expected_arrival_records` (预登记) 到入库页
5. `expected_arrival` row 有 `source = 'purchase', source_id = 42`
6. 当入库用户创建入库时，确认该到货记录与预期匹配

### 6.2 事件通道

| Key Pattern | Publisher | Subscriber |
| ------------- | ----------------- | ------------ |
| `orders.purchase.approved` | procurement | inventory, finance |
| `orders.sales.approved` | sales_crm | inventory, finance |
| `inventory.stock.changed` | inventory_atp | manufacturing |
| `inventory.inbound.completed` | inventory_atp | finance, workflow |
| `hr.employee.created` | hr | auth (创建用户账户) |
| `hr.salary.paid` | hr | finance (创建 journal entry) |
| `workflow.status.changed` | workflow | notification |

### 6.3 异步 Job

SQLite 内建队列（无外部 MQ 依赖），场景：

- `excel-imports` → Excel 文件导入
- `email-sends` → 邮件通知
- `report-generations` → 生成报表 (Excel/PDF)
- `recurring` → 定时作业 (日终处理、月结)

---

## 7. 部署架构

### 7.1 Docker Compose

```yaml
services:
  nginx:
    image: nginx:1.27-alpine
    ports: ["80:80", "443:443"]
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf
      - ./nginx/certs/:/etc/ssl/certs/

  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports: ["3000:3000"]
    volumes:
      - ./backend/data:/app/data   # SQLite 数据持久化
    environment:
      - DATABASE_URL=sqlite://data/erp.db?mode=rwc
      - JWT_SECRET=${JWT_SECRET}
      - APP_ENV=production
    restart: always
```

### 7.2 Kubernetes (后续阶段)

Phase 2 以后再提供部署配置（YAML）—— 初期专注 Docker Compose 与裸进程。

---

## 8. 安全架构

### 8.1 认证流程

```
Client → JWT Bearer → POST /auth/login
              ↓
         Refresh Token (httpOnly cookie)
              ↓
       Access Token (1h)
              ↓
    RBAC Permissions check in middleware
              ↓
         Handler execution
```

### 8.2 Identity

| 组件 | 当前方案 | 升级后的方案 |
| ------ | -------- | ---------- |
| Token signing | HS256 单个共享 JWT secret | RS256 (key pair) |
| 角色 | 4 个硬编码 (admin/warehouse/qc/sales) | 多对多 RBAC (role+permission) + 元数据 |
| 组织 | 无 | multi-tenant (tenant → companies → departments) |
| 审计 | 没有权限审计 | 每个 request 记录 `user_id`, `tenant_id`, `ip_address`, `action` |
| Token 黑名单 | 无能 | refresh_tokens.revoked（DB 存储，无需 Redis） |

### 8.3 Request-Level Auditing

每个 `/api/*` 请求，中间件记录:

- User ID (0 表示匿名)
- Tenant ID (0 表示 no tenant context)
- Timestamp
- IP Address
- Duration
- HTTP Status Code

Log 汇总到 `audit_log` 表中。

---

## 9. 可观测性

### 9.1 框架

| Pillar | 工具 |
| -------- | ------ |
| 指标 | tracing + tracing-subscriber (JSON) |
| 日志 | JSON 格式的 tracing (tower-http trace layer) |
| 跟踪 | request_id (uuid v4) 贯穿请求 |
| 健康检查 | `GET /api/v1/health/live` — 是否运行<br>`GET /api/v1/health/ready` — DB 连接是否就绪 |

---

## 10. 编码约定

### 10.1 后端（延续现有模式）

| 方面 | 约定 |
| ------ | ------ |
| DI | `Extension<SqlitePool>` + `Extension<JwtSecret>`（无 State extractor） |
| Handler | `Result<Json<...>, AppError>`（非 `impl IntoResponse`） |
| Service | Unit struct 静态方法，`pool: &SqlitePool` |
| Repo | 静态方法，`WHERE deleted_at IS NULL` |
| 错误 | 5-digit error codes, domain prefixed |
| 返回类型 | `Result<Json<ApiResponse<T>>, AppError>` |
| 分层 | Handler → Service → Repo |
| 配置 | `Config::from_env()` |

### 10.2 前端

| 方面 | 约定 |
| ------ | ------ |
| Module 结构 | `features/{module}/api/, hooks/, pages/, types/, queryKeys.ts` |
| TanStack Query | `useQuery` / `useMutation` + `invalidateQueries({ queryKey: all })` |
| 路由 | `createBrowserRouter` + `ProtectedRoute` |
| 权限 UI | `Can` component wrapping |
| i18n | per-feature zh/en namespaces |

---

## 11. 迁移策略

### 阶段 A: 审计修复 + 基础设施加固 (Phase 0)

1. 修复已知缺陷（见 `000-audit-fix.md`）
2. 37 个遗留迁移文件重写为 SQLite 语法 → 删除钢管表 → 新增 `items` 商品表
3. Docker Compose 可选集成

### 阶段 B: 模块开发（Phase 1-4）

4. 按模块逐个编写 spec，执行 → 测试 → CR

### 阶段 C: 发布

1. Staging 验证
2. Production deployment

---

## 附录: 术语参考

| 领域 | 术语 | 含义 |
| ------ | ------ | ------ |
| 商品 | 商品 (Item) | 可交易的基本业务对象，全系统唯一实体 |
| 商品 | SKU | 商品的唯一业务编码 |
| 会计 | chart_of_accounts (科目表) | 分类财务 transaction 的层级结构 |
| 工作流 | approval_chain | 审批链，构成工作流的主要实例 |
| 工作流 | delegation (委托) | 用户在离岗时将审批权委托给另一个人 |
| 制造 | routing | 产品通过多个工序/工作站的序列 |
| 制造 | BOM (物料清单) | 树状结构描述产品和中间体之间的关系 |
| 制造 | NCR | Non-Conformance Report— 质检不合格处理 |
| 制造 | 质检 (Inspection) | 工单的质量检验记录 |
| | ATP | Available-To-Promise — 基于所有在库/在途的库存承诺 |
| 库存 | 库存盘点 | 周期性实物清点 process, 与系统记录对比 |
| 分账 | 3-way 匹配 | 采购订单 + 采购收货 + 发票 三者一致审批 |
| 财务 | O2C (Order-to-Cash) | 从订单到付款的完整流程 |
| 财务 | P2P (Procure-to-Pay) | 从采购申请到支付款的完整流程 |
| 财务 | Dunning | 迟付款催收流程 |
| Portal | 门户账户 (Party) | 客户/供应商在门户中的登录身份 |

---

> **下一层文档**: `016-data-schema.md` 定义所有表。
