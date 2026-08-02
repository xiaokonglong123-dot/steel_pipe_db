# 015 — Steel Pipe ERP: 总体架构设计

> **版本**: v1.0  
> **日期**: 2026-08-02  
> **状态**: Draft  
> **作者**: Sisyphus / Ikari Shinji  
> **依赖**: 基于现有 Steel Pipe DB v1.0 码库  
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
|------|------|
| **Module Monolith First** | 单一 Rust 进程，内部按业务域模块化，不引入分布式复杂性直到实际需要 |
| **Schema-隔离，单数据库** | 所有模块共享同一个 PostgreSQL 数据库，按 schema 分区 (`inventory`, `orders`, `finance`, `hr`, `manufacturing`, ...) |
| **同步事务，异步事件** | 同模块内操作保持 ACID 联动。跨模块通过 Redis Pub/Sub 事件异步 |
| **Failure-First Design** | 每个操作要发生错误时提供可诊断的错误码。错误码按模块分配范围 |
| **Tenant-Ready** | 单实例支持多公司/多工厂，数据结构通过 `tenant_id`/`company_id` 隔离 |
| **Simple Deployment** | Docker Compose 支持一键启动。可选 Kubernetes 部署 |
| **Frontend Features** | 前端按业务模块居中，每个模块有独立的 API/hooks/pages/i18n |
| **Audit-First** | 所有关键操作记录 audit log。禁止对数据库中任何行物理删除（始终 `deleted_at`） |

---

## 2. 系统拓扑

```
                     ┌──────────────┐
                     │   Browser    │
                     │ React 19 SPA │
                     └──────┬───────┘
                            │ HTTPS + WSS
                     ┌──────▼───────┐
                     │    Nginx      │
                     │ Reverse Proxy │
                     └──────┬───────┘
                            │
            ┌───────────────┼───────────────┐
            │               │               │
     ┌──────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐
     │  Rust Axum   │ │   Redis     │ │  RabbitMQ   │
     │  Monolith    │ │  (Pub/Sub)  │ │  (Async     │
     │  Port 3000  │ │  + Cache     │ │   Jobs)     │
     └──────┬──────┘ └─────────────┘ └─────────────┘
            │
     ┌──────▼──────┐
     │ PostgreSQL 16│
     │ (schemas:   │
     │  inventory, │
     │  orders,    │
     │  finance…)  │
     └──────┬──────┘
            │
     ┌──────▼──────┐
     │   MinIO     │
     │  (S3 API)  │
     └─────────────┘
```

### 组件说明

| 组件 | 角色 | 初始部署方式 |
|------|------|------------|
| **Nginx** | TLS 终端 + 静态 → SPA + 反代 API | Docker Container |
| **Rust Axum Monolith** | 所有的 REST 构架 | Docker Container |
| **PostgreSQL 16** | 主数据库 | Docker Container |
| **Redis Stack (KeyDB/Valkey)** | 缓存、Pub/Sub、分布式锁、Session 黑名单 | Docker Container |
| **RabbitMQ** | 异步 Job 队列（文件导入/导出、邮件发送） | Docker Container (关闭着指令) |
| **MinIO** (可选) | S3兼容文件存储 | Docker Container |

---

## 3. 应用架构

### 3.1 Rust Crate 结构

```
backend/
├── Cargo.toml (workspace)
├── crates/
│   ├── core/                    ← 共享基础：DB, Config, Error, Tracing, Response
│   ├── auth/                    ← Auth middleware + User/identity management
│   ├── pipes/                   ← 管材 master data
│   ├── inventory/               ← 库存管理 + 收发 + 仓库
│   ├── orders/                  ← 采购 + 销售 + 合同
│   ├── trace/                   ← 跟踪路线
│   ├── finance/                  ← 总账 + 应收应付 + 资产 + 多币种
│   ├── hr/                      ← 员工 + 部门 + 考勤 + 薪酬
│   ├── manufacturing/           ← BOM + 工单 + 工序 + 质检
│   ├── pipeline/                ← 管道设计 + 螺纹几何计算
│   ├── projects/                ← 项目 + WBS + 预算
│   ├── assets/                  ← 固定资产 + 折旧
│   ├── workflow/                ← 审批流图 + 执行引擎
│   ├── notification/            ← 消息中心 + 模板 + 通道
│   ├── reporting/               ← 报表引擎 + BI integrations
│   ├── labels/                  ← 标签打印
│   ├── dataio/                  ← Excel/CSV 批量操作
│   └── gateway/                 ← SPA 服务 + 路由合并
└── migrations/                  ← SQLx 迁移
```

每个 crate 有四个层次：

```
auth/
├── Cargo.toml
├── src/
│   ├── mod.rs                   ← Re-exports
│   ├── routes.rs                ← Axum route definitions (合并到 gateway)
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── login.rs
│   │   ├── users.rs
│   │   └── ...
│   ├── services/
│   │   └── ...
│   ├── repositories/
│   │   ├── migrations/          ← SQLx flame → 对应的 schema 表
│   │   └── ...
│   ├── models.rs
│   ├── dto/
│   │   └── ...
│   └── domain.rs
```

### 3.2 Core crate 设定

```rust
// core/src/config.rs — 全局泛型配置
pub struct AppConfig {
    pub app_env: AppEnv,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub smtp: SmtpConfig,
    pub rabbitmq: RabbitMqConfig,
    pub s3: Option<S3Config>,
}
```

```rust
// core/src/error.rs — 全局错误码范围
pub enum AppErrorDomain {
    General,    // 100xx
    Auth,       // 110xx
    Pipe,       // 120xx
    Inventory,  // 130xx
    Order,      // 140xx
    Quality,    // 150xx
    Supplier,   // 160xx
    Customer,   // 170xx
    DataIO,     // 180xx
    Finance,    // 190xx
    HR,         // 200xx
    Manufacturing, // 210xx
    Projects,   // 220xx
    Assets,     // 230xx
    Workflow,   // 240xx
    Notification, // 250xx
    Security,   // 260xx
    Db,         // 500xx
}
```

---

## 4. 数据库架构

### 4.1 PostgreSQL Schema 隔离

每个业务域拥有独立的 PostgreSQL schema。跨域访问只能通过 Repository 接口——绝不让一个模块直接触碰另一个模块的 schema。

```
PostgreSQL: steel_pipe_erp
├── common                       ← 共享实体: users, roles, tenants, companies, audit_log
│   ├── users
│   ├── roles, role_permissions
│   ├── tenants, companies
│   ├── audit_log                 ← 统一审计日志
│   └── event_store               ← CQRS event sourcing (可选)
├── inventory                     ← 原有 + 增强 Schema
│   ├── seamless_pipes, screen_pipes, welded_pipes
│   ├── locations
│   ├── inbound_records, inbound_items
│   ├── outbound_records, outbound_items
│   ├── inventory_logs, inventory_check_records, inventory_check_items
│   ├── atp_slots                  ← 高级：Available-To-Promise slot
│   └── labels
├── orders                        ← 采购 + 销售 + 合同
│   ├── purchase_orders, purchase_order_items
│   ├── sales_orders, sales_order_items
│   ├── contracts, contract_items, contract_payments
│   └── quote_requests, quote_records
├── finance                       ← 新增
│   ├── chart_of_accounts
│   ├── journal_entries, journal_entry_details
│   ├── accounts_receivable, accounts_payable
│   ├── invoices              (AP/AR)
│   ├── 收据, payments
│   ├── currency_rates
│   └── tax_codes
├── hr                            ← 新
│   ├── employees
│   ├── departments
│   ├── positions
│   ├── attendances
│   ├── 工时logs
│   ├── 薪酬配置, salary_items
│   └── contracts
├── manufacturing                 ← 新
│   ├── boms, bom_items
│   ├── work_orders, work_order_steps
│   ├── routing_ops
│   ├── quality_inspections, quality_check_items
│   ├── defect_records, defect_reasons
│   ├── equipment_register
│   ├── threading_records [API 5CT 专用]
│   └── ncr_outputs
├── projects                      ← 新
│   ├── projects, project_funding
│   ├── wbs_elements
│   ├── 日程, milestones
│   └── transactions (link to contracts, purchases, sales)
├── assets                        ← 新
│   ├── fixed_assets
│   ├── 调拨
│   ├── 折旧计算规则
│   └── depreciation_entries
├── workflow                      ← 新
│   ├── workflow_definitions
│   ├── 条件s, actions
│   ├── workflow_instances
│   ├── approval_chain_nodes
│   └── 审记录
├── notification                  ← 新
│   ├── notification_templates
│   ├── notifications
│   ├── 交付通道 (mail_send_log, smtp_req...)
│   └── 用户偏好
└── data_io                       ← 导入导出日志
    ├── import_records
    ├── import_errors
    └── export_tasks
```

### 4.2 关键数据库规则

- **`updated_at`可以做完全更新的触发器**：使用 Pg的 updated_at 函数 + TRIGGER 自动更新
- **Hard Delete**：禁止！`事务表的事务操作例外` — 永远用 `deleted_at` 柱对用户可见
- **FK 约束**：应用的层面保障完整性；PostgreSQL 中外层FK可配置（但开发时可选禁用以提高效率）
- **迁移**：使用现代的 DB 迁移工具 (Atlas/Tern)，但沿用相同命名格式 `021_create_...`。后续所有迁移统一为 PostgreSQL SQL。
- **Unit 分离**：迁移在不同 crate 的 `migrations/` 文件夹中，再由主构建工具统一编译为 `backend/migrations/`。

---

## 5. 模块划分 & 依赖

### 5.1 Capability Map

```
                         ┌────────────────┐
                         │                │
                         │   Business     │
                         │   Intelligence │
                         └───────┬────────┘
                                 │ reads from ALL crates
┌───────────┐  ┌───────────┐  ┌──▼────────┐  ┌──────────┐  ┌────────────┐
│  Projects  │  │  Contracts │  │ Inventory  │  │ Sales/CRM │  │ Procedures  │
│   (PBM)   │  │   (Orders) │  │  & ATP    │  │  (Orders) │  │  (Orders)  │
└─────┬─────┘  └─────┬─────┘  └─────┬─────┘  └─────┬────┘  └──────┬─────┘
      │              │              │              │             │
      └──────┬───────┴──────┬───────┴──────┬───────┘             │
             │              │              │                      │
       ┌─────▼─────┐ ┌──────▼──────┐ ┌─────▼─────┐              │
       │  Workflow  │ │  Finance    │ │    HR     │              │
       │            │ │  (engine)  │ │           │              │
       └───────────┘ └─────────────┘ └───────────┘              │
                                                                │
 ┌──────────────────────────────────────────────────────────────▼────────────────┐
 │                            Core /  Cross-Cutting                               │
 │          Auth · Config · Error · Tracing · Common DB · DbPool              │
 └───────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 依赖

| Crate | 依赖 |
|-------|------|
| `core` | 无（共享基础） |
| `auth` | core |
| `pipes` | core (无其他) |
| `inventory` | core, pipes |
| `orders` | core, pipes, inventory |
| `trace` | core, pipes, inventory, orders |
| `finance` | core, orders, hr |
| `hr` | core, auth |
| `manufacturing` | core, inventory, pipes, trace |
| `projects` | core, orders, finance |
| `assets` | core, finance |
| `workflow` | core, auth |
| `notification` | core, auth |
| `labels` | core, pipes, inventory |
| `data-io` | core |
| `reporting` | core, papers, inventory, orders, finance, hr |
| `gateway` | 所有 crate (合并路由) |

---

## 6. 跨模块通信

### 6.1 事件流 (Pub/Sub)

**Example Purchase Order approved → inventory now awaits 预计到货:**

1. Purchase Order Service 批准 PO
2. Service update `purchase_orders.status = 'approved'`
3. Service pub to Redis: `orders.purchase.approved.purchase_order_id=42`
4. Inventory Awaiter sub to this channel → creates `expected_arrival_records` (预登记) to send to inbound page
5. `expected_arrival` row has `source = 'purchase', source_id = 42`
6. When the inbound user actually creates the inbound, they confirm that this arrival record matches the expected one.

### 6.2 事件通道

| Key Pattern | Publisher Crate | Subscriber |
|-------------|-----------------|------------|
| `orders.purchase.approved` | orders | inventory, finance |
| `orders.sales.approved` | orders | inventory, finance |
| `inventory.stock.changed` | inventory, manufacturing | |
| `inventory.inbound.completed` | inventory, finance, workflow | |
| `manufacturing.thread.completed` | manufacturing, trace | |
| `hr.employee.created` | hr | auth (创建用户账户) |
| `hr.salary.paid` | hr | finance (创建 journal entry) |
| `workflow.status.changed` | workflow, notification | |

### 6.3 异步 Job (RabbitMQ)

Queue names:
- `excel-imports` → Excel 文件导入队列 (10万+ 行)
- `email-sends`  → 发送可定制的邮件通知
- `report-generations` → 生成报表 (Excel/PDF)
- `recurring` → 定时作业 (日终处理、库存poise重算、月结)
- `sync-external` → 与虚外部系统同步

---

## 7. 部署架构

### 7.1 Docker Compose

```yaml
services:
  nginx:
    image: nginx:1.27-alpine
    ports: ["80:32080", "443:32443"]
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf
      - ./nginx/certs/:/etc/ssl/certs/

  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports: ["3000:3000"]
    depends_on: { postgres: healthy, redis: healthy, rabbitmq: healthy }
    environment:
      - DATABASE_URL=postgres://postgres:${DB_PASSWORD}@postgres:5432/steel_pipe_erp
      - REDIS_URL=redis://redis:6379
      - RABBITMQ_URL=amqp://guest:guest@rabbitmq:5672
      - JWT_SECRET=${JWT_SECRET}
      - APP_ENV=production
    restart: always

  postgres:
    image: postgres:16-alpine
    volumes: ["pgdata:/var/lib/postgresql/data"]
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=steel_pipe_erp
    healthcheck: [check alive]

  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]

  rabbitmq:
    image: rabbitmq:3-management-alpine
    ports: ["5672:5672", "15672:15672"]

  minio:
    image: minio/minio
    ports: ["9000:9000", "9001:9001"]
```

### 7.2 Kubernetes (后续阶段)

Phase 2 以后再提供部署配置（YAML）—— 初期专注 Docker Compose。

---

## 8. 安全架构

### 8.1 认证流程

```
Client → JWT Bearer → POST /auth/login
              ↓
         Refresh Token (httpOnly cookie)
              ↓
       Access Token (1h, bearercिन)
              ↓
    RBAC Permissions check in middleware
              ↓
         Handler execution
```

### 8.2 Identity

| 组件 | 当前方案 | 升级后的方案 |
|------|--------|----------|
| Token signing | HS256 单个共享 JWT secret | RS256 (key pair) |
| 角色 | 4 个硬编码 (admin/warehouse/qc/sales) | 多对多 RBAC (role+permission) + 元数据 |
| 组织 | 无 | multi-tenant (tenant → companies → departments) |
| 审计 | 没有权限审计 | 每个 request 记录 `user_id`, `tenant_id`, `ip_address`, `action` |
| Token 黑名单 | 无能 | Redis token黑名单 (log term) + 自动失效 |
| `Refresh Token` | 256 SHA- 服务端 | 同类，by PostgreSQL |

### 8.3 Request-Level Auditing

每个 `/api/*` 请求，中间件记录:
- User ID (0 表示匿名)
- Tenant ID (0 表示 no tenant context)
- Timestamp (含 `created_at`)
- IP Address
- Duration
- HTTP Status Code

Log 汇总到 `common.audit_log` 表中，用于关键操作 (审计员 | app_owners 向系统追问)。

---

## 9. 可观测性

### 9.1 格子

| Pillar | 工具 |
|--------|------|
| 指标 | Prometheus / 导出器 (Rust-app 通过 `prometheus` 中 的 HA 指标) |
| 日志 | JSON 格式的 tracing (elastic search 或 Loki 收集) |
| 跟踪 | OpenTelemetry 摘要 (通过 `opentelemetry` 库，自定义) |
| 警报 | Prometheus 指标 + 异常模式 |

### 9.2 健康检查

`GET /api/v1/health/live` — 是否运行
`GET /api/v1/health/ready` — 数据库 + Redis + RabbitMQ 连接是否就绪

---

## 10. 编码约定

### 10.1 后端（续现行现有模式）

| 方面 | 约定 |
|------|------|
| Handler | `Extension(pool)` + `Extension(user)` + `Path/Query/Json` |
| Service | Unit struct 静态方法 |
| 错误 | 5-digit error codes, territory prefixed |
| 验证 | 通过 `req.validate().map_err(...)` |
| 返回类型 | `Result<Json<...>, AppError>` 或 `Result<Response, AppError>` |
| 分层 | Handler → Service → Repo |
| 配置 | `Config::from_env()` |

### 10.2 前端

| 方面 | 约定 |
|------|------|
| Module 结构 | `features/{module}/api/, hooks/, pages/, types.ts, queryKeys.ts` |
| TanStack Query | `useQuery` / `useMutation` on Success: `invalidateQueries({ queryKey: all })` |
| 路由 | `createBrowserRouter` + `ProtectedRoute` |
| 权限 UI | `Can` component wrapping, 隐藏没权限的元素 |

---

## 11. 迁移策略

### 阶段 A: 审计修复 + C基础升级 (Phase 0)
1. 修复已知的 8 项缺陷（见 `000-audit-fix.md`）
2. SQLite → PostgreSQL 迁移 → 编写 `PostgresMigration.sql`
3. Docker Compose 集成

### 阶段 B: 模块开发（Phase 1-4）
4. 按模块逐个编写 spec，执行 → 测试 → CR

### 阶段 C: 发布
5. Staging 验证
6. Production deployment

---

## 附录: 术语参考

| 领域 | 术语 | 含义 |
|------|------|------|
| 会计 | chart_of_accounts (科目表) | 分类财务 transaction 的层级结构 |
| 工作流 | approval_chain | 审批链，构成工作流的主要实例 |
| 工作流 | delegation (委托) | 用户在离岗时将审批权委托给另一个人 |
| 制造 | routing | 产品通过多个工序/工作站的序列 |
| 制造 | BOM (物料清单) | 树状结构描述产品和中间体之间的关系 |
| 制造 | NCR | Non-Conformance Report— 质检不合格处理 |
| 螺纹 | thread inspection | 测量和研究 pipe thread 的 machine parameters |
| | ATP | Available-To-Promise — 基于所有在库/线上的库存表面承诺 |
| 库存 | 库存盘查 | 周期性实物清点 process, 与系统录 curring 对比 |
| 分账 | 3-way 匹配 | PO + 收据 + 发票 三者一致审批 |
| 财务 | O2C (Order-to-Cash) | 从订单到付款的完整流程 |
| 财务 | P2P (Procure-to-Pay) | 从采购申请到支付款的完整流程 |
| ABA | Dunning | 迟付款催收流程 |
| Portal | Bidding | 供应商对 PO 提出投标 |

---

> **下一层文档**: `016-data-schema.md` 定义 granite-separate 模块 (scripts) 的所有表。