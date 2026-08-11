# ERP v2 — 详细设计文档（后端）

> **版本**: v2.0-alpha
> **日期**: 2026-08-09
> **依赖**: [PRD.md](./PRD.md) — 所有模块定义、FR、术语以此为准
> **目标读者**: 实现者（含 AI agent）

---

## 1. 架构总览

```
HTTP Request
  → tower-http (CORS, trace, request-id)
  → axum::Router
  → middleware/auth.rs  (JWT 校验 → AuthUser → RBAC 查库实时校验)
  → http/{module}.rs   (routes + handlers 合一，解析 DTO，调用 service)
  → services/{module}.rs (事务边界，业务规则，Decimal 计算)
  → repos/{module}.rs   (纯 SQL，sqlx::query! / query_as!，无业务逻辑)
  → SqlitePool
```

**分层纪律**：
- **Handler**（`http/`）：仅做 DTO 解析/校验 → 调 service → 序列化响应。**不含 SQL、不含事务。**
- **Service**（`services/`）：业务规则 + **事务边界**（`.begin()/.commit()/.rollback()`）。调用 repo 完成读写。金额计算用 `rust_decimal`。
- **Repo**（`repos/`）：纯 SQL（`sqlx::query!` / `query_as!`），返回 domain 类型或 Row 结构。**不含事务控制、不含业务逻辑。**
- **Domain**（`domain/`）：enums、money 模块（Decimal 类型别名 + 序列化）、状态机（`order.rs`）、错误码。

**DI 模式**（继承 v1 验证过的 Extension 模式）：

```rust
// router.rs
let app = Router::new()
    .layer(Extension(pool))
    .layer(Extension(JwtSecret(jwt_secret)));

// handler
async fn create_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(AuthUser(user)): Extension<AuthUser>,
    Json(dto): Json<CreateItemRequest>,
) -> Result<Json<ApiResponse<ItemResponse>>, AppError> { ... }
```

无 `State<Arc<AppState>>`，无全局可变状态。

---

## 2. 项目结构

```
backend/
├── Cargo.toml
├── migrations/
│   ├── 001_auth_rbac.sql
│   ├── 002_catalog.sql
│   ├── 003_parties.sql
│   ├── 004_inventory.sql
│   ├── 010_warehouses.sql     ← 在 004 之后追加的层级扩展：warehouses 父表 + locations.warehouse_id/deleted_at
│   ├── 005_purchasing.sql
│   ├── 006_sales.sql
│   ├── 007_finance.sql
│   ├── 008_workflow.sql
│   └── 009_seed.sql
├── src/
│   ├── main.rs           # tracing init + pool + migrate + serve
│   ├── lib.rs            # 模块声明
│   ├── config.rs         # Config::from_env()
│   ├── error.rs          # AppError + error_codes! 宏 + IntoResponse
│   ├── response.rs       # ApiResponse<T> / PaginatedResponse<T> / Meta
│   ├── db.rs             # pool 初始化 + 迁移
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── money.rs      # Money(MoneyDecimal)，rust_decimal 的序列化/反序列化
│   │   └── order.rs      # 订单状态机（purchase/sales shared）
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs       # JWT 校验 + AuthUser 提取器
│   │   └── rbac.rs       # 查库实时 RBAC 校验
│   ├── auth.rs           # JWT 签发/校验、refresh token 轮换、bootstrap_admin
│   ├── http/             # routes + handlers
│   │   ├── mod.rs        # router() 组装
│   │   ├── auth.rs
│   │   ├── catalog.rs
│   │   ├── parties.rs
│   │   ├── inventory.rs
│   │   ├── purchasing.rs
│   │   ├── sales.rs
│   │   ├── finance.rs
│   │   ├── workflow.rs
│   │   └── reports.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth_service.rs
│   │   ├── catalog_service.rs
│   │   ├── parties_service.rs
│   │   ├── inventory_service.rs
│   │   ├── purchasing_service.rs
│   │   ├── sales_service.rs
│   │   ├── finance_service.rs
│   │   ├── workflow_service.rs
│   │   └── reports_service.rs
│   └── repos/
│       ├── mod.rs
│       ├── auth_repo.rs
│       ├── catalog_repo.rs
│       ├── parties_repo.rs
│       ├── inventory_repo.rs
│       ├── purchasing_repo.rs
│       ├── sales_repo.rs
│       ├── finance_repo.rs
│       ├── workflow_repo.rs
│       └── reports_repo.rs
├── tests/
│   ├── common/
│   │   └── mod.rs         # 临时文件 SQLite 实例 + 迁移
│   ├── auth_integration.rs
│   ├── catalog_integration.rs
│   ├── inventory_integration.rs
│   ├── purchasing_integration.rs
│   ├── sales_integration.rs
│   ├── finance_integration.rs
│   └── workflow_integration.rs
```

**模块声明在 lib.rs**：

```rust
pub mod auth;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod http;
pub mod middleware;
pub mod repos;
pub mod response;
pub mod services;
```

---

## 3. 金额 Decimal 策略（ADR-002 决议）

### 3.1 问题

sqlx-sqlite 0.8 不实现 `rust_decimal::Decimal` 的编解码（`Encode`/`Decode` trait 在 sqlite feature 下不实现）。直接绑定 `Decimal` 到 SQLite 列会编译失败。

### 3.2 方案

**金额存储**：SQLite 列类型 `TEXT`，存储 `rust_decimal::Decimal::to_string()` 的 canonical 十进制字符串（如 `"1234.56"`、`"-0.01"`）。

**编码（写）**：repo 层 `amount.to_string()` → `bind(&amount_str)`。

**解码（读）**：repo 层 QueryRow 解码为 `String` → `Decimal::from_str(&row.amount_str)?`。

**计算**：service 层全链路 `rust_decimal::Decimal` 运算（`+`、`*`、`round_dp`、`checked_div`），金额比较用 `Decimal` 的 `==` / `partial_cmp`。

**SQL 聚合**：业务事务（PO 金额合计、日记账借贷平衡）**不在 SQL 做 SUM**（SQLite SUM 对 TEXT 列会 CAST AS REAL 丢精度），在应用层用 `Decimal` 累计：
```rust
let total: Decimal = rows.iter().map(|r| r.amount).sum();
```
**报表场景**：销售额趋势等聚合报表可对金额做 `CAST(total_amount AS REAL)` 后 SQL SUM——REAL 双精度 15 位有效数字，对单厂 ERP 金额范围（≤10 万亿）精确到分，可接受。若需绝对精度，报表在 service 层用 Decimal 累计各条记录。

**倒排（Display→JSON）**：serde 序列化 Decimal 为 JSON number（`#[serde(with = "rust_decimal::serde::float")]` 或自定义为字符串）。

### 3.3 repo 层金额绑定模板

```rust
// 写
sqlx::query("INSERT INTO purchase_orders (total_amount) VALUES (?)")
    .bind(total_amount.to_string())
    .execute(pool).await?;

// 读
struct PORow { total_amount_str: String }
let row = sqlx::query_as!(PORow, "SELECT total_amount FROM purchase_orders WHERE id = ?", id)
    .fetch_one(pool).await?;
let total_amount = Decimal::from_str(&row.total_amount_str)?;
```

---

## 4. 数据库 Schema（全新迁移）

> 所有迁移为 SQLite 方言：`?` 占位符、`INTEGER PRIMARY KEY AUTOINCREMENT`、`datetime('now')`、无 PG 特性。软删除统一 `deleted_at TEXT`。FK 约束仅在核心关系上声明（`REFERENCES` + `ON DELETE RESTRICT`），部分表无 FK（标注"integrity enforced at application layer"）。

### 4.1 001_auth_rbac.sql — 认证与权限

```sql
-- RBAC: roles + permissions 为唯一权限事实源，查库实时校验
CREATE TABLE users (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    username      TEXT NOT NULL UNIQUE,
    display_name  TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    email         TEXT,
    phone         TEXT,
    is_active     INTEGER NOT NULL DEFAULT 1,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at    TEXT
);

CREATE TABLE roles (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    description TEXT,
    is_system   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE permissions (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    key  TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
);

CREATE TABLE role_permissions (
    role_id       INTEGER NOT NULL REFERENCES roles(id),
    permission_id INTEGER NOT NULL REFERENCES permissions(id),
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE user_roles (
    user_id INTEGER NOT NULL REFERENCES users(id),
    role_id INTEGER NOT NULL REFERENCES roles(id),
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE refresh_tokens (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL REFERENCES users(id),
    token_hash TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    revoked_at TEXT
);
CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);

CREATE TABLE operation_logs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER REFERENCES users(id),
    action      TEXT NOT NULL,
    target_type TEXT,
    target_id   INTEGER,
    detail      TEXT,
    ip_address  TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**种子权限字典**：
```
item.read, item.write, stock.read, stock.write,
order.read, order.write, order.approve,
finance.read, finance.write, report.read,
user.manage
```

**种子角色**：admin（全权限）、manager（读+审批）、warehouse（库存/商品读写）、purchaser（采购/商品/供应商读写）、sales（销售/商品/客户读写）、finance（财务读写+报表读）。

### 4.2 002_catalog.sql — 商品主数据

```sql
CREATE TABLE items (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    sku        TEXT NOT NULL UNIQUE,
    name       TEXT NOT NULL,
    category   TEXT,                          -- 原材料/半成品/成品/备件
    unit       TEXT,                          -- kg/m/pc/件 等
    spec       TEXT,                          -- 自由文本规格
    status     TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','active','disabled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);
CREATE INDEX idx_items_sku ON items(sku);
CREATE INDEX idx_items_category ON items(category);
CREATE INDEX idx_items_status ON items(status);
```

### 4.3 003_parties.sql — 供应商与客户

```sql
CREATE TABLE suppliers (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    code       TEXT NOT NULL UNIQUE,
    name       TEXT NOT NULL,
    contact    TEXT,
    phone      TEXT,
    email      TEXT,
    address    TEXT,
    status     TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);
CREATE INDEX idx_suppliers_code ON suppliers(code);

CREATE TABLE customers (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    code       TEXT NOT NULL UNIQUE,
    name       TEXT NOT NULL,
    contact    TEXT,
    phone      TEXT,
    email      TEXT,
    address    TEXT,
    status     TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);
CREATE INDEX idx_customers_code ON customers(code);
```

### 4.4 004_inventory.sql — 库存

```sql
-- 仓库/库位（扁平层级，code 唯一）→ 由 010_warehouses.sql 扩展为层级：warehouses 父表 + locations.warehouse_id FK
CREATE TABLE locations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    description TEXT,
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 物化库存余额表（item × location 当前余额）
CREATE TABLE inventory (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id     INTEGER NOT NULL REFERENCES items(id),
    location_id INTEGER NOT NULL REFERENCES locations(id),
    quantity    REAL NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(item_id, location_id)
);

-- 库存事件日志（完整审计轨迹）
CREATE TABLE inventory_logs (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id      INTEGER NOT NULL REFERENCES items(id),
    location_id  INTEGER REFERENCES locations(id),
    change_type  TEXT NOT NULL CHECK (change_type IN ('inbound','outbound','check_adjust')),
    quantity     REAL NOT NULL,
    ref_type     TEXT,
    ref_id       INTEGER,
    notes        TEXT,
    created_by   INTEGER REFERENCES users(id),
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_invlogs_item ON inventory_logs(item_id);
CREATE INDEX idx_invlogs_created ON inventory_logs(created_at);

-- 入库单
CREATE TABLE inbound_records (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    record_no     TEXT NOT NULL UNIQUE,
    inbound_type  TEXT NOT NULL CHECK (inbound_type IN ('purchase','production','return','other')),
    order_id      INTEGER REFERENCES purchase_orders(id),
    supplier_id   INTEGER REFERENCES suppliers(id),
    status        TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','posted','cancelled')),
    notes         TEXT,
    created_by    INTEGER REFERENCES users(id),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at    TEXT
);

CREATE TABLE inbound_items (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id  INTEGER NOT NULL REFERENCES inbound_records(id),
    item_id    INTEGER NOT NULL REFERENCES items(id),
    location_id INTEGER REFERENCES locations(id),
    quantity   REAL NOT NULL,
    notes      TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 出库单
CREATE TABLE outbound_records (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    record_no     TEXT NOT NULL UNIQUE,
    outbound_type TEXT NOT NULL CHECK (outbound_type IN ('sales','requisition','other')),
    order_id      INTEGER REFERENCES sales_orders(id),
    customer_id   INTEGER REFERENCES customers(id),
    status        TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','posted','cancelled')),
    notes         TEXT,
    created_by    INTEGER REFERENCES users(id),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at    TEXT
);

CREATE TABLE outbound_items (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id  INTEGER NOT NULL REFERENCES outbound_records(id),
    item_id    INTEGER NOT NULL REFERENCES items(id),
    location_id INTEGER REFERENCES locations(id),
    quantity   REAL NOT NULL,
    notes      TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 盘点
CREATE TABLE check_records (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    record_no    TEXT NOT NULL UNIQUE,
    location_id  INTEGER REFERENCES locations(id),
    status       TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','counted','posted','cancelled')),
    notes        TEXT,
    created_by   INTEGER REFERENCES users(id),
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at   TEXT
);

CREATE TABLE check_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id   INTEGER NOT NULL REFERENCES check_records(id),
    item_id     INTEGER NOT NULL REFERENCES items(id),
    system_qty  REAL,     -- 账面数（盘点时快照）
    actual_qty  REAL,     -- 实盘数
    diff        REAL,     -- 差异 = actual_qty - system_qty
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ATP 预留
CREATE TABLE reservations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id     INTEGER NOT NULL REFERENCES items(id),
    quantity    REAL NOT NULL,
    order_type  TEXT NOT NULL CHECK (order_type IN ('sales')),
    order_id    INTEGER NOT NULL,   -- sales_orders.id
    status      TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','released','cancelled')),
    created_by  INTEGER REFERENCES users(id),
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    released_at TEXT
);
CREATE INDEX idx_reservations_item ON reservations(item_id, status);
```

> **库存过账规则**：`inbound_records.status='posted'` 触发事务：更新 `inventory` 余额 + 插入 `inventory_logs`（change_type='inbound'）。出库同理。

### 4.5 005_purchasing.sql — 采购

```sql
CREATE TABLE purchase_orders (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    order_no      TEXT NOT NULL UNIQUE,
    supplier_id   INTEGER NOT NULL REFERENCES suppliers(id),
    order_date    TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','submitted','approved','rejected','ordered','partially_received','received','cancelled')),
    doc_status    INTEGER NOT NULL DEFAULT 0,   -- 0草稿/1已提交/2已取消（审批流联动）
    total_amount  TEXT NOT NULL DEFAULT '0',     -- Decimal TEXT 列
    currency      TEXT NOT NULL DEFAULT 'CNY',
    notes         TEXT,
    created_by    INTEGER REFERENCES users(id),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at    TEXT
);
CREATE INDEX idx_po_supplier ON purchase_orders(supplier_id);
CREATE INDEX idx_po_status ON purchase_orders(status);

CREATE TABLE purchase_order_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id    INTEGER NOT NULL REFERENCES purchase_orders(id),
    item_id     INTEGER NOT NULL REFERENCES items(id),
    quantity    REAL NOT NULL,
    received_qty REAL NOT NULL DEFAULT 0,
    unit_price  TEXT,        -- Decimal TEXT
    total_price TEXT,        -- Decimal TEXT (quantity * unit_price)
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**采购订单状态机**（`domain/order.rs`）：

```
draft → submitted → approved → ordered → partially_received → received
                       ↓                    ↓
                   rejected             cancelled (仅 ordered 之前)
```

`doc_status`：0=draft, 1=submitted（已提交审批流转中），2=cancelled。`doc_status` 与 `status` 双层映射（与审批流联动）。

**审批联动规则**（防止双层状态不一致）：
1. PO `/submit` → `PO.status='submitted'` + `PO.doc_status=1` + 创建 `workflow_instance(current_state='submitted')`
2. PO `/approve` → `workflow_service` 推进 `instance.current_state='approved'` → 回调 `purchasing_service` 同步 `PO.status='approved'`，`doc_status` 保持 1（已完成审批流转）
3. PO `/reject` → `PO.status='rejected'` + `instance.current_state='rejected'` + `instance.status='completed'`
4. PO `/cancel`（仅 draft/submitted）→ `PO.status='cancelled'` + `PO.doc_status=2`，如有 workflow_instance 则 `instance.status='cancelled'`
5. PO `/receive`（收货联动）→ 不变更 workflow，仅推进订单业务状态
> 销售订单同理。`doc_status` 全局含义统一：0=草稿/未提交审批，1=已提交或审批完成，2=已取消。

### 4.6 006_sales.sql — 销售

```sql
CREATE TABLE sales_orders (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    order_no      TEXT NOT NULL UNIQUE,
    customer_id   INTEGER NOT NULL REFERENCES customers(id),
    order_date    TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','submitted','approved','rejected','awaiting_shipment','partially_shipped','shipped','cancelled')),
    doc_status    INTEGER NOT NULL DEFAULT 0,
    total_amount  TEXT NOT NULL DEFAULT '0',
    currency      TEXT NOT NULL DEFAULT 'CNY',
    notes         TEXT,
    created_by    INTEGER REFERENCES users(id),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at    TEXT
);
CREATE INDEX idx_so_customer ON sales_orders(customer_id);
CREATE INDEX idx_so_status ON sales_orders(status);

CREATE TABLE sales_order_items (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id     INTEGER NOT NULL REFERENCES sales_orders(id),
    item_id      INTEGER NOT NULL REFERENCES items(id),
    quantity     REAL NOT NULL,
    shipped_qty  REAL NOT NULL DEFAULT 0,
    unit_price   TEXT,        -- Decimal TEXT
    total_price  TEXT,
    notes        TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**销售订单状态机**：

```
draft → submitted → approved → awaiting_shipment → partially_shipped → shipped
           ↓
       rejected
     (仅 submitted)
```

**ATP 检查**：创建/提交销售订单时，`services/sales_service.rs` 计算 `available_qty = inventory.quantity - COALESCE(SUM(reservations.quantity), 0)`，若任一行 `quantity > available_qty` → `AppError::InsufficientStock`。

审批通过后（`approved`）为订单创建 reservations（quantity = 订单行数量，status='active'）。发货过账时释放预留（`reservations.status='released'`）。

### 4.7 007_finance.sql — 财务

```sql
-- 会计科目树（parent_id 自引用）
CREATE TABLE accounts (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    parent_id   INTEGER REFERENCES accounts(id),
    account_type TEXT NOT NULL CHECK (account_type IN ('asset','liability','equity','income','expense')),
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 日记账（借贷平衡 Decimal 累计 + round_dp(4) 校验）
CREATE TABLE journal_entries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_no    TEXT NOT NULL UNIQUE,
    entry_date  TEXT NOT NULL,
    description TEXT,
    status      TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','posted','voided')),
    ref_type    TEXT,
    ref_id      INTEGER,
    created_by  INTEGER REFERENCES users(id),
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE journal_entry_lines (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id    INTEGER NOT NULL REFERENCES journal_entries(id),
    account_id  INTEGER NOT NULL REFERENCES accounts(id),
    debit       TEXT NOT NULL DEFAULT '0',
    credit      TEXT NOT NULL DEFAULT '0',
    description TEXT,
    -- ⚠️ 约束由 service 层保证：同一条 line 的 debit 与 credit 不可同时非零
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 发票
CREATE TABLE invoices (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_no  TEXT NOT NULL UNIQUE,
    invoice_date TEXT NOT NULL,
    party_type  TEXT NOT NULL CHECK (party_type IN ('supplier','customer')),
    party_id    INTEGER NOT NULL,
    amount      TEXT NOT NULL,       -- Decimal TEXT
    ref_type    TEXT,                -- purchase_order / sales_order
    ref_id      INTEGER,
    status      TEXT NOT NULL DEFAULT 'unpaid' CHECK (status IN ('unpaid','partially_paid','paid','voided')),
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 付款
CREATE TABLE payments (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    payment_no  TEXT NOT NULL UNIQUE,
    payment_date TEXT NOT NULL,
    supplier_id INTEGER REFERENCES suppliers(id),
    amount      TEXT NOT NULL,
    invoice_id  INTEGER REFERENCES invoices(id),
    method      TEXT,
    notes       TEXT,
    created_by  INTEGER REFERENCES users(id),
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 4.8 008_workflow.sql — 审批流（数据驱动）

```sql
-- 审批流定义
CREATE TABLE workflows (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    applies_to  TEXT NOT NULL CHECK (applies_to IN ('purchase_order','sales_order','inbound_record','outbound_record')),
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 状态（含 doc_status 映射）
CREATE TABLE workflow_states (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id INTEGER NOT NULL REFERENCES workflows(id),
    state_key   TEXT NOT NULL,          -- draft/submitted/approved/rejected……
    doc_status  INTEGER NOT NULL DEFAULT 0,
    is_initial  INTEGER NOT NULL DEFAULT 0,
    is_final    INTEGER NOT NULL DEFAULT 0,
    UNIQUE(workflow_id, state_key)
);

-- 状态迁移规则
CREATE TABLE workflow_transitions (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id    INTEGER NOT NULL REFERENCES workflows(id),
    from_state_id  INTEGER NOT NULL REFERENCES workflow_states(id),
    to_state_id    INTEGER NOT NULL REFERENCES workflow_states(id),
    action         TEXT NOT NULL,       -- submit/approve/reject
    required_role  TEXT,                -- NULL = 任意角色可执行
    is_auto        INTEGER NOT NULL DEFAULT 0  -- 1 = 系统自动迁移
);

-- 审批流实例（绑定业务单据）
CREATE TABLE workflow_instances (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id    INTEGER NOT NULL REFERENCES workflows(id),
    business_type  TEXT NOT NULL,       -- purchase_order / sales_order
    business_id    INTEGER NOT NULL,    -- 对应 PO/SO id
    current_state  TEXT NOT NULL,
    status         TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','completed','cancelled')),
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 审批任务（待办）
CREATE TABLE workflow_tasks (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    instance_id  INTEGER NOT NULL REFERENCES workflow_instances(id),
    state_key    TEXT NOT NULL,
    assignee_id  INTEGER REFERENCES users(id),
    status       TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','completed','skipped')),
    action       TEXT,            -- 审批人选择的 action
    comment      TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);
```

**审批流集成逻辑**（`services/workflow_service.rs`）：
1. 单据提交 → `workflow_instances` 创建实例 + `workflow_tasks` 创建待办（assignee 按 `required_role` 查同角色用户）
2. 审批动作 → 更新 task.status → 按 `workflow_transitions` 推进 instance.current_state
3. 状态迁移联动 → 回调 `purchasing_service::update_status` / `sales_service::update_status`（更新 PO/SO 的 status/doc_status）

### 4.9 009_seed.sql — 种子数据

```sql
-- 系统角色
INSERT INTO roles (id, name, description, is_system) VALUES (1, 'admin', '系统管理员', 1);
INSERT INTO roles (id, name, description, is_system) VALUES (2, 'manager', '业务经理', 1);
INSERT INTO roles (id, name, description, is_system) VALUES (3, 'warehouse', '仓库', 1);
INSERT INTO roles (id, name, description, is_system) VALUES (4, 'purchaser', '采购', 1);
INSERT INTO roles (id, name, description, is_system) VALUES (5, 'sales', '销售', 1);
INSERT INTO roles (id, name, description, is_system) VALUES (6, 'finance', '财务', 1);

-- 权限字典
INSERT INTO permissions (key, name) VALUES
    ('item.read',   '商品-查看'), ('item.write',  '商品-编辑'),
    ('stock.read',  '库存-查看'), ('stock.write', '库存-操作'),
    ('order.read',  '订单-查看'), ('order.write', '订单-编辑'), ('order.approve', '订单-审批'),
    ('finance.read','财务-查看'), ('finance.write','财务-记账'),
    ('report.read', '报表-查看'),
    ('user.manage', '用户-管理');

-- admin 全权限
INSERT INTO role_permissions (role_id, permission_id) SELECT 1, id FROM permissions;
-- manager: 读 + 审批
INSERT INTO role_permissions (role_id, permission_id)
    SELECT 2, id FROM permissions WHERE key IN ('item.read','stock.read','order.read','order.approve','finance.read','report.read');
-- warehouse: 库存/商品
INSERT INTO role_permissions (role_id, permission_id)
    SELECT 3, id FROM permissions WHERE key IN ('item.read','item.write','stock.read','stock.write','order.read','report.read');
-- purchaser: 采购/商品/供应商
INSERT INTO role_permissions (role_id, permission_id)
    SELECT 4, id FROM permissions WHERE key IN ('item.read','item.write','stock.read','order.read','order.write','report.read');
-- sales: 销售/商品/客户
INSERT INTO role_permissions (role_id, permission_id)
    SELECT 5, id FROM permissions WHERE key IN ('item.read','item.write','stock.read','order.read','order.write','report.read');
-- finance: 财务 + 报表读
INSERT INTO role_permissions (role_id, permission_id)
    SELECT 6, id FROM permissions WHERE key IN ('finance.read','finance.write','report.read');
```

> bootstrap_admin 在 `main.rs` 启动时创建 admin 用户（Argon2 哈希，密码从 env `ADMIN_PASSWORD` 取，默认 `admin123`），分配给 role_id=1。

---

## 5. API 契约

> 所有 API 前缀 `/api/v1/`。响应形状统一（见 PRD §7.4）。`?` = query param（可选）。所有写操作需 `Authorization: Bearer <token>`；读操作按 RBAC 校验权限。

### 5.1 Auth

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| POST | `/auth/login` | 登录 → access token (cookie) + refresh token | public |
| POST | `/auth/refresh` | 刷新 token | public（带 cookie） |
| POST | `/auth/logout` | 登出 → 吊销所有 refresh token | authenticated |
| GET | `/auth/me` | 当前用户信息 | authenticated |
| GET | `/users` | 用户列表（分页） | `user.manage` |
| POST | `/users` | 创建用户 | `user.manage` |
| PUT | `/users/{id}` | 更新用户 | `user.manage` |
| DELETE | `/users/{id}` | 禁用（软删除）用户 | `user.manage` |
| GET | `/roles` | 角色列表 | `user.manage` |
| GET | `/permissions` | 权限列表 | `user.manage` |
| GET | `/operation-logs` | 操作日志（分页+筛选） | `admin` |

### 5.2 商品 (Catalog)

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/items` | 列表（分页+筛选） | `item.read` |
| GET | `/items/{id}` | 详情 | `item.read` |
| POST | `/items` | 创建 | `item.write` |
| PUT | `/items/{id}` | 更新 | `item.write` |
| DELETE | `/items/{id}` | 软删除 | `item.write` |

### 5.3 往来单位 (Parties)

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/suppliers` | 供应商列表（分页+筛选） | `item.read` |
| POST | `/suppliers` | 创建 | `item.write` |
| PUT | `/suppliers/{id}` | 更新 | `item.write` |
| DELETE | `/suppliers/{id}` | 软删除 | `item.write` |
| GET | `/customers` | 客户列表 | `item.read` |
| POST | `/customers` | 创建 | `item.write` |
| PUT | `/customers/{id}` | 更新 | `item.write` |
| DELETE | `/customers/{id}` | 软删除 | `item.write` |

### 5.4 库存 (Inventory)

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/locations` | 库位列表 | `stock.read` |
| POST | `/locations` | 创建库位 | `stock.write` |
| GET | `/inventory` | 库存余额查询（按商品/库位/分类 筛选，分页） | `stock.read` |
| GET | `/inventory/logs` | 库存流水（按商品/时间范围 筛选，分页） | `stock.read` |
| GET | `/inbounds` | 入库单列表 | `stock.read` |
| POST | `/inbounds` | 创建入库单（头+行） | `stock.write` |
| GET | `/inbounds/{id}` | 入库单详情 | `stock.read` |
| POST | `/inbounds/{id}/post` | 入库过账（→ 单事务更新库存 + 写日志） | `stock.write` |
| GET | `/outbounds` | 出库单列表 | `stock.read` |
| POST | `/outbounds` | 创建出库单 | `stock.write` |
| GET | `/outbounds/{id}` | 出库单详情 | `stock.read` |
| POST | `/outbounds/{id}/post` | 出库过账 | `stock.write` |
| GET | `/check-records` | 盘点单列表 | `stock.read` |
| POST | `/check-records` | 创建盘点单 | `stock.write` |
| PUT | `/check-records/{id}` | 录入盘点结果 | `stock.write` |
| POST | `/check-records/{id}/post` | 盘点过账（生成差异调整 → inventory_logs） | `stock.write` |
| GET | `/reservations` | 预留列表（按商品/状态） | `stock.read` |

### 5.5 采购 (Purchasing)

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/purchase-orders` | 订单列表（分页+筛选） | `order.read` |
| POST | `/purchase-orders` | 创建（ATP 不需要，采购不预占用库存） | `order.write` |
| GET | `/purchase-orders/{id}` | 详情（含明细行） | `order.read` |
| PUT | `/purchase-orders/{id}` | 更新（仅 draft） | `order.write` |
| DELETE | `/purchase-orders/{id}` | 软删除（仅 draft/cancelled） | `order.write` |
| POST | `/purchase-orders/{id}/submit` | 提交（→ 创建审批流实例） | `order.write` |
| POST | `/purchase-orders/{id}/approve` | 审批通过（→ 更新状态 + 审批流实例推进） | `order.approve` |
| POST | `/purchase-orders/{id}/reject` | 驳回 | `order.approve` |
| POST | `/purchase-orders/{id}/cancel` | 取消 | `order.write` |

### 5.6 销售 (Sales)

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/sales-orders` | 订单列表 | `order.read` |
| POST | `/sales-orders` | 创建（创建时 ATP 检查：可用量 = 库存-预留 ≥ 订单量） | `order.write` |
| GET | `/sales-orders/{id}` | 详情 | `order.read` |
| PUT | `/sales-orders/{id}` | 更新（仅 draft） | `order.write` |
| DELETE | `/sales-orders/{id}` | 软删除 | `order.write` |
| POST | `/sales-orders/{id}/submit` | 提交（→ 审批流） | `order.write` |
| POST | `/sales-orders/{id}/approve` | 审批通过（→ 创建 reservations） | `order.approve` |
| POST | `/sales-orders/{id}/reject` | 驳回 | `order.approve` |
| POST | `/sales-orders/{id}/cancel` | 取消（→ 释放 reservations） | `order.write` |

### 5.7 财务 (Finance)

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/accounts` | 会计科目列表 | `finance.read` |
| POST | `/accounts` | 创建 | `finance.write` |
| GET | `/journal-entries` | 日记账列表（分页+日期筛选） | `finance.read` |
| POST | `/journal-entries` | 创建（余额校验：借方合计 = 贷方合计，Decimal 累计 round_dp(4)） | `finance.write` |
| POST | `/journal-entries/{id}/post` | 过账 | `finance.write` |
| GET | `/invoices` | 发票列表 | `finance.read` |
| POST | `/invoices` | 创建 | `finance.write` |
| PUT | `/invoices/{id}` | 更新 | `finance.write` |
| GET | `/payments` | 付款列表 | `finance.read` |
| POST | `/payments` | 创建付款 | `finance.write` |
| GET | `/trial-balance` | 试算平衡表（按科目汇总借/贷发生额） | `finance.read` |

### 5.8 审批流 (Workflow)

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/workflows` | 审批流定义列表 | `order.read` |
| POST | `/workflows` | 创建 | admin |
| PUT | `/workflows/{id}` | 更新（节点/迁移规则） | admin |
| GET | `/workflow-tasks` | 我的待办列表 | authenticated |
| GET | `/workflow-instances` | 实例列表 | `order.read` |
| POST | `/workflow-tasks/{id}/approve` | 审批通过 | 任务 assignee |
| POST | `/workflow-tasks/{id}/reject` | 驳回 | 任务 assignee |

### 5.9 报表 (Reports)

| 方法 | 路径 | 说明 | 权限 |
|------|------|------|------|
| GET | `/reports/inventory-summary` | 库存汇总（按分类/商品分组） | `report.read` |
| GET | `/reports/inbound-outbound` | 出入库明细（日期范围+商品筛选） | `report.read` |
| GET | `/reports/sales-trend` | 销售趋势（按月/季，支持 ?months=N） | `report.read` |
| GET | `/reports/finance-summary` | 财务摘要（本期收入/支出/应收/应付） | `finance.read` |
| GET | `/reports/export` | 导出（CSV，query params 同对应明细接口） | `report.read` |

---

## 6. 实现模式（Handler / Service / Repository 模板）

### 6.1 Handler

```rust
// http/catalog.rs
pub async fn create_item(
    Extension(pool): Extension<SqlitePool>,
    Extension(AuthUser(user)): Extension<AuthUser>,
    Json(dto): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ItemResponse>>), AppError> {
    let item = CatalogService::create_item(&pool, &dto).await?;
    let resp = ApiResponse::ok(ItemResponse::from(item));
    Ok((StatusCode::CREATED, Json(resp)))
}
```

### 6.2 Service（事务边界 + Decimal 计算）

```rust
// services/purchasing_service.rs
pub struct PurchasingService;

impl PurchasingService {
    pub async fn create_order(
        pool: &SqlitePool,
        user_id: i64,
        dto: &CreatePORequest,
    ) -> Result<PurchaseOrderResponse, AppError> {
        let mut tx = pool.begin().await?;

        // 1. 计算每个行的 total_price（Decimal）
        let items: Vec<POLineCalc> = dto.items.iter().map(|i| {
            let qty = Decimal::from_f64_retain(i.quantity)
                .ok_or(AppError::Validation("invalid quantity".into()))?;
            let price = Decimal::from_str(&i.unit_price)
                .map_err(|_| AppError::Validation("invalid price".into()))?;
            let total = qty * price;
            Ok(POLineCalc { item_id: i.item_id, qty, price, total })
        }).collect::<Result<Vec<_>, AppError>>()?;

        // 2. 汇总总金额
        let total_amount: Decimal = items.iter().map(|i| i.total).sum();

        // 3. 写订单头
        let order = PurchasingRepo::create_order(&mut tx, &dto, total_amount, user_id).await?;

        // 4. 批量写订单行
        PurchasingRepo::create_order_items(&mut tx, order.id, &items).await?;

        // 5. 写操作日志
        AuthRepo::log_operation(&mut tx, user_id, "create_purchase_order", "purchase_order", order.id, "").await?;

        tx.commit().await?;
        Ok(order.into())
    }
}
```

### 6.3 Repo

```rust
// repos/purchasing_repo.rs
pub struct PurchasingRepo;

impl PurchasingRepo {
    pub async fn create_order(
        tx: &mut Transaction<'_, Sqlite>,
        dto: &CreatePORequest,
        total_amount: Decimal,
        user_id: i64,
    ) -> Result<PORow, AppError> {
        let order_no = generate_order_no("PO"); // 日期+序号
        sqlx::query!(
            "INSERT INTO purchase_orders (order_no, supplier_id, order_date, total_amount, notes, created_by)
             VALUES (?, ?, ?, ?, ?, ?)",
            order_no, dto.supplier_id, dto.order_date,
            total_amount.to_string(), dto.notes, user_id
        )
        .execute(&mut **tx).await?;
        let id = sqlx::query_scalar!("SELECT last_insert_rowid()")
            .fetch_one(&mut **tx).await?;
        Ok(PORow { id, order_no, /* ... */ })
    }
}
```

### 6.4 库存过账模板（service 层单事务）

```rust
// services/inventory_service.rs
pub async fn post_inbound(pool: &SqlitePool, record_id: i64, user_id: i64) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    let record = InventoryRepo::find_inbound(&mut tx, record_id).await?;
    if record.status != "draft" {
        return Err(AppError::StatusConflict("only draft can be posted".into()));
    }

    let items = InventoryRepo::find_inbound_items(&mut tx, record_id).await?;
    for item in &items {
        // 原子更新库存余额
        InventoryRepo::upsert_inventory(&mut tx, item.item_id, item.location_id, item.quantity).await?;
        // 写日志
        InventoryRepo::insert_log(&mut tx, item.item_id, item.location_id, "inbound", item.quantity,
            "inbound", record_id, user_id).await?;
    }

    InventoryRepo::update_inbound_status(&mut tx, record_id, "posted").await?;
    tx.commit().await?;
    Ok(())
}
```

### 6.5 错误码与 IntoResponse

```rust
// error.rs
use axum::response::{IntoResponse, Response};
use axum::Json;

error_codes! {
    // 通用 100xx
    Internal(10001, "内部错误"),
    Validation(10002, "请求参数校验失败"),
    NotFound(10003, "资源未找到"),
    StatusConflict(10004, "状态冲突"),
    // Auth 110xx
    Unauthorized(11001, "未认证"),
    TokenExpired(11002, "token 已过期"),
    Forbidden(11003, "权限不足"),
    // Catalog 120xx
    ItemNotFound(12001, "商品未找到"),
    ItemDuplicateSKU(12002, "SKU 重复"),
    // Inventory 130xx
    InsufficientStock(13001, "库存不足"),
    LocationNotFound(13002, "库位未找到"),
    // Orders 140xx
    OrderCannotModify(14001, "订单当前状态不可修改"),
    OrderNotFound(14002, "订单未找到"),
    // Parties 150xx
    SupplierNotFound(15001, "供应商未找到"),
    CustomerNotFound(15002, "客户未找到"),
    // Finance 160xx
    AccountNotFound(16001, "会计科目未找到"),
    UnbalancedJournal(16002, "日记账借贷不平衡"),
    // Workflow 170xx
    WorkflowNotFound(17001, "审批流未找到"),
    InvalidTransition(17002, "无效的状态迁移"),
    // DB
    Database(50001, "数据库错误"),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_code();
        let body = ApiErrorResponse {
            success: false,
            code,
            request_id: Uuid::new_v4().to_string(),
            message: self.user_message(),  // 用户友好文案，不含 SQL 细节
            details: None,
        };
        (status, Json(body)).into_response()
    }
}

// From<sqlx::Error> → 一律转 Database(50001)，不暴露 SQL 字符串
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(?e, "database error");
        AppError::Database
    }
}
```

---

## 7. 中间件策略

### 7.1 Auth Middleware

```rust
// middleware/auth.rs
pub struct AuthUser(pub AuthenticatedUser);

pub async fn auth_middleware(
    Extension(pool): Extension<SqlitePool>,
    Extension(JwtSecret(secret)): Extension<JwtSecret>,
    cookies: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. 从 cookie 或 Authorization header 提取 token
    // 2. JWT verify → user_id + username + role（或从 DB 实时查？role 从 token 取作快速展示，RBAC 在下面单独查）
    // 3. 注入 AuthUser
}

// middleware/rbac.rs
pub async fn rbac_middleware(
    Extension(pool): Extension<SqlitePool>,
    Extension(AuthUser(user)): Extension<AuthUser>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. 查 user_roles → roles → role_permissions → permissions
    // 2. 所需权限从 req 的 extension/matched_path 映射
    // 3. 无权限 → 403
}
```

### 7.2 限流

```rust
// middleware/rate_limit.rs
// auth 端点（login/refresh）基于 socket peer IP（ConnectInfo）限流 10/min
// 中间件可用 tower::limit 或自定义速率桶
```

---

## 8. 配置（config.rs）

```rust
pub struct Config {
    pub database_url: String,       // sqlite://data/erp.db?mode=rwc
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,      // 默认 24
    pub refresh_expiry_days: u64,   // 默认 7
    pub server_host: String,        // 默认 0.0.0.0
    pub server_port: u16,           // 默认 3000
    pub cors_origins: Vec<String>,  // 默认 localhost:5173
    pub admin_username: String,     // 默认 admin
    pub admin_password: String,     // 默认 admin123
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> { /* dotenvy + env */ }
    pub fn server_addr(&self) -> Result<SocketAddr, AppError> { /* format + parse */ }
}
```

---

## 9. 测试策略

### 9.1 单元测试

- **Service 层**：每个 service 函数单元测试，使用临时 SQLite 数据库（tempfile + migrate）。`tests/common/mod.rs` 提供 `test_pool()` —— 每个测试独立临时文件，全程隔离。

### 9.2 集成测试

- **测试范围**：auth login flow → item CRUD → supplier/customer CRUD → inbound post → outbound post → purchase order submit/approve/receive → sales order submit/approve/ship → journal entry balance → workflow instance → report queries
- **覆盖 v1 已验证的语义**：291 个 v1 测试的语义作为设计参照，但不搬运代码
- **每个业务链一条**：auth → catalog → inventory → purchasing → sales → finance → workflow → reports，顺序验证

### 9.3 构建/CI

```bash
cargo check                           # 类型检查
cargo test                            # 全量测试
cd frontend && bun install && tsc --noEmit && vite build  # 前端
```

### 9.4 测试辅助 `tests/common/mod.rs`

```rust
pub async fn test_pool() -> SqlitePool {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let pool = SqlitePool::connect(&format!("sqlite://{}?mode=rwc", db_path.display())).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    // bootstrap admin
    auth::bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    pool
}
```

---

## 10. 迁移与演进

- 迁移文件**不可修改**已执行过的 version（防止 checksum 错误）
- 新功能加新版号迁移（010+, 011+, …）
- 所有迁移为 SQLite 方言（`?`、`INTEGER PRIMARY KEY AUTOINCREMENT`、`datetime('now')`、`REAL` 用量等）
- 数字类型：`REAL` 用于数量、比率；`TEXT` 用于金额和枚举字符串

---

> **下一步**：`frontend-design.md`（Vue 3 + Element Plus 组件树 / 路由 / Pinia stores / TanStack Vue Query 模式）。
