<div align="center">

> **🤖 All code in this repository is AI-generated** — from architecture design to every line of code, produced entirely by large language models for technical demonstration and capability validation purposes.

</div>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/ERP-1f2937?style=flat-square&logo=rust&logoColor=white">
  <img alt="ERP" src="https://img.shields.io/badge/ERP-1f2937?style=flat-square&logo=rust&logoColor=white">
</picture>

# ERP — Enterprise Resource Planning System (通用企业资源计划系统)

> A general-purpose ERP: item/SKU inventory, procurement, sales, finance, HR, manufacturing, projects, fixed assets, workflow approvals, notifications and BI. Rust backend, React frontend. Does what it says on the tin.
>
> 历史沿革: 本系统由钢管行业系统重构而来 (legacy steel-pipe system), rebuilt as a general-purpose ERP.

![Rust](https://img.shields.io/badge/Rust-Axum-000000?style=flat-square&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-003B57?style=flat-square&logo=sqlite&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript&logoColor=white)
![Ant Design](https://img.shields.io/badge/Ant_Design-5-1677FF?style=flat-square&logo=antdesign&logoColor=white)

---

## 🚀 Quick Start

### Prerequisites

| Tool  | Version    |
|-------|------------|
| Rust  | 1.78+ (edition 2021) |
| Node  | 20+        |
| npm   | 10+        |

### Backend

```bash
cd backend
cp .env.example .env    # or roll your own: DATABASE_URL=sqlite://data/erp.db?mode=rwc
cargo run               # fires up on http://localhost:3000
```

The backend crate is `erp-server` (documented target for the code phase). The database is a single SQLite3 file (`data/erp.db`) — no external DB server required.

### Frontend

```bash
cd frontend
npm install
npm run dev             # fires up on http://localhost:5173
```

Open `http://localhost:5173` and log in with:

| Username | Password  |
|----------|-----------|
| `admin`  | `admin123` |

---

## 🏗 Tech Stack

### Backend — Rust (Axum 0.8)

| Layer        | Technology                                           |
|-------------|------------------------------------------------------|
| Framework   | Axum 0.8 with macros + multipart                     |
| ORM         | SQLx 0.8 (SQLite3, `sqlite` feature)                 |
| Auth        | JWT (jsonwebtoken 9) + Argon2 password hashing       |
| Validation  | Validator 0.19 (derive)                              |
| Logging     | Tracing + tracing-subscriber (env-filter, json)       |
| Excel/CSV   | calamine (import), rust_xlsxwriter (export), csv      |
| Middleware  | tower-http (CORS, trace, request-id)                 |
| Database    | SQLite3, `sqlite://data/erp.db?mode=rwc` (WAL mode)  |

**Architecture:** Handler → Service → Repository → Domain. No AppState — the DB pool is injected via `Extension<SqlitePool>`, while auth secrets use a redacted `JwtSecret` extension.

### Frontend — React 19

| Category       | Library                                           |
|---------------|---------------------------------------------------|
| UI Framework  | React 19 + Ant Design 5 + @ant-design/icons       |
| Routing       | react-router-dom 7                                |
| State         | Zustand 5 (client state) + TanStack Query 5 (server state) |
| HTTP Client   | Native `fetch` wrapper (`src/api/client.ts`)   |
| i18n          | react-i18next + i18next (zh + en per-module)      |
| Build Tool    | Vite 6                                            |
| Type Safety   | TypeScript 5 + Zod 3                              |

---

## 📚 Modules

### Phase 1 — Core (P0)

| Module     | Description                                    |
|------------|------------------------------------------------|
| Auth/RBAC  | JWT login/refresh/logout, RBAC (roles/permissions/departments/tenants) |
| Items & Inventory | 商品 (Item) + SKU master data (sku, name, category, unit, spec), stock at location, ATP reservations, inbound/outbound, count sessions |
| Workflow   | Approval engine: definitions, instances, tasks |

### Phase 2 — Business (P1)

| Module       | Description                                    |
|--------------|------------------------------------------------|
| Suppliers    | Supplier management, qualification, scorecard  |
| Customers    | Customer management, credit                    |
| Purchases    | 采购订单 (Purchase Order) management, inbound approval workflow |
| Sales        | 销售订单 (Sales Order), outbound, auto-ATP check |
| Procurement  | Requisitions, 采购报价 (supplier quotes), receipts |
| HR           | Employees, attendance, salaries, labor contracts |
| Finance      | Accounts (GL), journal entries, invoices, payments, trial balance |
| Manufacturing| BOMs, 工单 (work orders), 质检 (Inspection), NCRs |
| Data IO      | Excel/CSV batch import and export              |

### Phase 3 — Enterprise (P2)

| Module        | Description                                    |
|---------------|------------------------------------------------|
| Contracts     | Sales/Procurement contracts, payment milestones |
| Projects      | Projects, WBS, budget                          |
| Fixed Assets  | Registration, straight-line depreciation, disposal |
| Notifications | Inbox, templates, preferences                  |
| Portal        | Customer/supplier portal accounts, party JWT, PO accept / SO ack |
| BI            | Sales trend, inventory value, finance summary, supplier performance |
| i18n          | Internationalization (zh/en, per-feature namespaces) |

---

## 🗄 Data Model

SQLite3 single file (WAL mode). Integrity is enforced at the application layer. The table names below are the target layout of the migration rewrite (the legacy 37 migrations are rewritten to SQLite syntax with pipe-specific tables dropped):

```
users                  → System users (RBAC)                         [001]
roles / permissions / departments / tenants → RBAC structures        [001]
items                  → 商品/SKU master (sku, name, category, unit, spec) [002]
warehouses / locations → Warehouse & location hierarchy              [002]
inventory              → Stock by item at location                   [002]
inbound_records / inbound_items → Inbound header + line items        [002]
outbound_records / outbound_items → Outbound header + line items     [002]
inventory_logs         → Item movement audit trail                   [002]
inventory_check_records / inventory_check_items → Count sessions     [002]
reservations           → ATP reservations (sales orders / work orders) [002]
suppliers              → Supplier master                             [003]
customers              → Customer master                             [003]
purchase_orders / purchase_order_items → PO header + line items      [003]
sales_orders / sales_order_items → SO header + line items            [003]
quotes                 → 采购报价 / 销售报价                           [003]
shipments              → Sales shipment confirmations                [003]
customer_credit        → Customer credit usage                       [003]
contracts / contract_items / contract_payments → Contract header, line items, payment milestones [004]
accounts / journal_entries / invoices / payments → GL, journal, invoicing, payments [005]
employees / attendance / salaries / labor_contracts → HR records     [005]
requisitions / receipts / scorecards → Procurement records           [005]
boms / work_orders / inspections / ncrs → Manufacturing records      [006]
projects / wbs / budgets → Project management                        [007]
fixed_assets / depreciation / disposals → Fixed asset lifecycle      [007]
notifications / templates / preferences → Notification platform      [008]
portal_accounts       → Portal party accounts (customer/supplier)    [008]
workflow_definitions / workflow_instances / workflow_tasks → Approval engine [009]
operation_logs        → System operation audit trail                 [010]
refresh_tokens        → Server-side hashed refresh token sessions    [011]
```

All timestamps are ISO 8601 strings. Soft deletes via `deleted_at` — nothing ever truly dies.

---

## 🧪 Development

```bash
# Backend
cd backend && cargo check           # Type-check only (way faster than a full build)
cargo test                           # Run tests
cargo build                          # Debug build
cargo build --release                # Ship it

# Frontend
cd frontend && npx tsc --noEmit     # TypeScript type check
npm run build                        # Production build
npm run lint                         # ESLint
```

---

## 🔐 Security

- **Password**: Argon2id with recommended params (`m=19456, t=2, p=1`)
- **Auth**: Stateless JWT (HS256) for access tokens. Refresh tokens are stored server-side
  (SHA-256 hashed in `refresh_tokens` table) and rotate on each `/auth/refresh` call.
  Expired or revoked refresh tokens are rejected, and `/auth/logout` revokes all refresh
  tokens for the user to prevent further token renewal.
- **RBAC**: roles such as `admin`, `warehouse`, `qc`, `sales` — enforced via middleware
- **Rate limiting**: Per-IP throttling on auth endpoints (login/refresh) via middleware
- **Data IO**: Batch import is admin-only; export is limited to `admin`/`warehouse`/`sales`;
  operation logs are admin-only. CSV/XLSX exports escape spreadsheet formula prefixes
  (`=`, `+`, `-`, `@`) so exported user-controlled values open as text.
- **Data**: Soft deletes on all business entities, audit trail via `inventory_logs` and `operation_logs`

---

## 📁 Project Structure

```
erp/
├── backend/                       # erp-server crate (Rust Axum)
│   ├── src/
│   │   ├── main.rs           # Entry point, server startup
│   │   ├── lib.rs             # Module declarations
│   │   ├── router.rs          # Route definitions (~190 routes, ~170 unique paths)
│   │   ├── config.rs          # Environment config (DATABASE_URL=sqlite://data/erp.db?mode=rwc)
│   │   ├── error.rs           # AppError with ApiResponse mapping; ApiErrorResponse includes success + request_id
│   │   ├── response.rs        # ApiResponse<T> / PaginatedResponse<T> / Meta struct with request_id (uuid v4)
│   │   ├── domain/            # Domain enums & constants
│   │   ├── dto/               # Request/Response DTOs
│   │   ├── models/            # DB models
│   │   ├── repositories/      # SQL query layer
│   │   ├── services/          # Business logic layer
│   │   ├── handlers/          # Axum request handlers
│   │   ├── auth/ workflow/ hr/ finance/ procurement/ sales_crm/ inventory_atp/ manufacturing/
│   │   │   project/ assets/ notification/ portal/ bi/   # Module folders (repos + services + handlers)
│   │   └── middleware/        # Auth + RBAC middleware
│   ├── migrations/            # SQLx migrations (rewritten to SQLite syntax)
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── api/               # Native fetch wrapper (src/api/client.ts)
│   │   ├── features/          # Per-module: auth, items, inventory, purchases, sales, workflow, hr, finance...
│   │   ├── layouts/           # MainLayout with sidebar
│   │   ├── stores/            # Zustand stores
│   │   ├── routes/            # react-router route config
│   │   ├── shared/            # Shared components & hooks
│   │   ├── i18n/              # zh/en locales
│   │   ├── types/             # Global TypeScript types
│   │   └── styles/            # Global styles
│   ├── package.json
│   └── vite.config.ts
├── specs/                      # Ubiquitous language (terminology canon)
│   └── UBIQUITOUS_LANGUAGE_LATEST.md
├── docs/                      # Design & operations docs
│   ├── requirements.en.md     # PRD (English)
│   ├── detailed-design.en.md  # Architecture + DB + API design (English)
│   ├── frontend-design.en.md  # Frontend component tree & routing (English)
│   ├── 需求文档.md             # PRD (中文)
│   ├── 详细设计文档.md         # Architecture + DB + API design (中文)
│   ├── 前端设计文档.md         # Frontend design (中文)
│   ├── deployment.md          # Deployment guide (Nginx, Docker, backup)
│   ├── troubleshooting.md     # Troubleshooting (DB locks, JWT, CORS)
│   └── tasks/                 # Task breakdown
└── .github/workflows/
    └── ci.yml                 # CI: cargo check + tsc + vite build
```

---

## 🌐 API Overview

All endpoints live under `/api/v1/`:

| Group          | Prefix              | Auth Required |
|----------------|---------------------|:---:|
| Auth           | `/auth/*`           | Mixed |
| Users          | `/users/*`          | Admin only |
| Items          | `/items/*`          | Yes |
| Inventory      | `/inventory/*`      | Yes |
| Suppliers      | `/suppliers/*`      | Yes |
| Customers      | `/customers/*`      | Yes |
| Purchases      | `/purchase-orders/*`| Yes |
| Sales          | `/sales-orders/*`   | Yes |
| Contracts      | `/contracts/*`      | Yes |
| Reports        | `/reports/*`        | Yes |
| Data IO        | `/data-io/*`        | Yes |
| Workflow       | `/workflow/*`       | Yes |
| HR             | `/hr/*`             | Yes |
| Finance        | `/finance/*`        | Yes |
| Procurement    | `/procurement/*`    | Yes |
| Manufacturing  | `/manufacturing/*`  | Yes |
| Projects       | `/projects/*`       | Yes |
| Assets         | `/assets/*`         | Yes |
| Notifications  | `/notifications/*`  | Yes |
| Portal         | `/portal/*`         | Yes |
| BI             | `/bi/*`             | Yes |

Every response follows the same shape:

```json
{ "success": true, "request_id": "req_...", "data": { ... } }
```

Paginated responses tack on `meta: { total, page, page_size, total_pages }`. Error responses flip `success: false` and still include `request_id`.

---

## 🔑 RBAC Permission Matrix

| API Group | admin | warehouse | qc | sales |
| ----------------------- | :-----: | :---------: | :---: | :-----: |
| Users (write) | ✅ | ❌ | ❌ | ❌ |
| Items (write) | ✅ | ✅ | ❌ | ❌ |
| Inbound/Outbound (write) | ✅ | ✅ | ❌ | ❌ |
| Inspection (write) | ✅ | ❌ | ✅ | ❌ |
| Sales Orders (write) | ✅ | ❌ | ❌ | ✅ |
| Purchase Orders (write) | ✅ | ✅ | ❌ | ✅ |
| Suppliers/Customers (write) | ✅ | ✅ | ❌ | ✅ |
| Contracts (write) | ✅ | ✅ | ❌ | ✅ |
| Data Import | ✅ | ❌ | ❌ | ❌ |
| Data Export | ✅ | ✅ | ❌ | ✅ |
| Data IO Operation Logs | ✅ | ❌ | ❌ | ❌ |
| General read endpoints | ✅ | ✅ | ✅ | ✅ |

---

## 🧭 Design Docs

Design docs (in Chinese) live in [`docs/`](./docs/):

| Document | Content |
| ---------- | --------- |
| `requirements.en.md` | Full PRD: features, roadmap |
| `detailed-design.en.md` | Architecture, DB schema, REST API, security |
| `frontend-design.en.md` | Component tree, routing, state, i18n, theme |
| `需求文档.md` | PRD 中文版 |
| `详细设计文档.md` | 架构设计中文版 |
| `前端设计文档.md` | 前端设计中文版 |
| `deployment.md` | Deployment guide: production config, Nginx, Docker, backup |
| `troubleshooting.md` | Troubleshooting: database locks, JWT, CORS, migrations |
| `tasks/progress.md` | Master task tracking |

Also see: [`CONTRIBUTING.md`](../CONTRIBUTING.md) · [`CHANGELOG.md`](../CHANGELOG.md)

---

## 📄 License

[GNU General Public License v2](./LICENSE)
