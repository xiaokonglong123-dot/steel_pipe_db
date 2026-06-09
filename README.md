<div align="center">

> **🤖 All code in this repository is AI-generated** — from architecture design to every line of code, produced entirely by large language models for technical demonstration and capability validation purposes.

</div>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/API-5CT-1f2937?style=flat-square&logo=rust&logoColor=white">
  <img alt="API 5CT" src="https://img.shields.io/badge/API-5CT-1f2937?style=flat-square&logo=rust&logoColor=white">
</picture>

# Steel Pipe DB — API 5CT Seamless Steel Pipe & Screen Pipe Inventory Management System

> Oil & gas inventory management for API 5CT seamless steel pipe and screen pipe. Rust backend, React frontend. Does what it says on the tin.

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
cp .env.example .env    # or roll your own: DATABASE_URL=sqlite://./data/steel_pipe.db?mode=rwc
cargo run               # fires up on http://localhost:3000
```

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
| ORM         | SQLx 0.8 (SQLite, runtime-tokio-rustls)              |
| Auth        | JWT (jsonwebtoken 9) + Argon2 password hashing       |
| Validation  | Validator 0.19 (derive)                              |
| Logging     | Tracing + tracing-subscriber (env-filter, json)       |
| Excel/CSV   | calamine (import), rust_xlsxwriter (export), csv      |
| Middleware  | tower-http (CORS, trace, request-id)                 |

**Architecture:** Handler → Service → Repository → Domain. No AppState — the DB pool is injected via `Extension<SqlitePool>`, while auth secrets use a redacted `JwtSecret` extension.

### Frontend — React 19

| Category       | Library                                           |
|---------------|---------------------------------------------------|
| UI Framework  | React 19 + Ant Design 5 + @ant-design/icons       |
| Routing       | react-router-dom 7                                |
| State         | Zustand 5 (client state) + TanStack Query 5 (server state) |
| HTTP Client   | Axios                                             |
| i18n          | react-i18next + i18next (zh + en per-module)      |
| Build Tool    | Vite 6                                            |
| Type Safety   | TypeScript 5 + Zod 3                              |

---

## 📚 Modules

### Phase 1 — Core (P0)
| Module     | Description                                    |
|------------|------------------------------------------------|
| Auth       | JWT login/refresh/logout, RBAC (admin/warehouse/qc/sales) |
| Pipes      | API 5CT pipe master data (steel grade, heat treatment, threading) |
| Inventory  | Per-pipe granular tracking, ATP calculation, inventory logs, inbound templates (auto-fill from PO), outbound stock awareness (browse in-stock pipes) |

### Phase 2 — Business (P1)
| Module     | Description                                    |
|------------|------------------------------------------------|
| Suppliers  | Supplier management, qualification tracking    |
| Customers  | Customer management, credit/contract history   |
| Purchases  | PO management, inbound approval workflow       |
| Sales      | Sales Orders, outbound, auto-ATP check         |
| Quality    | Inspection certificates, NDT, mechanical tests |
| Data IO    | Excel/CSV batch import and export              |

### Phase 3 — Enterprise (P2)
| Module     | Description                                    |
|------------|------------------------------------------------|
| Contracts  | Sales/Procurement contracts, payment milestones |
| Reports    | Dashboard, daily/monthly/statistical reports   |
| Labels     | Barcode and specification label generation     |
| i18n       | Internationalization (zh/en, metric/imperial)  |

---

## 🗄 Data Model

25 tables in SQLite (WAL mode, no FK constraints — integrity enforced at the app layer because SQLite FK support is meh). Table names below match `backend/migrations/001`–`015`:

```
users                  → System users (4 roles)                         [001]
seamless_pipes         → Seamless steel pipe master data (API 5CT specs) [002]
screen_pipes           → Screen pipe master data                        [003]
locations              → Warehouse locations                            [004]
inbound_records        → Inbound header (purchase, production, return)   [005]
inbound_items          → Inbound line items                             [005]
outbound_records       → Outbound header (sales, scrapped, transfer)    [005]
outbound_items         → Outbound line items                            [005]
inventory_logs         → Per-pipe movement audit trail                  [005]
inventory_check_records→ Inventory check (盘点) header                   [005]
inventory_check_items  → Inventory check line items                     [005]
suppliers              → Supplier master                                [006]
customers              → Customer master                                [006]
purchase_orders        → PO header                                      [006]
purchase_order_items   → PO line items                                  [006]
sales_orders           → SO header                                      [006]
sales_order_items      → SO line items                                  [006]
quality_certs          → QC certificates                                [007]
api_5ct_grade_ref      → API 5CT steel grade reference data             [007]
pipe_attachments       → Pipe document/attachment metadata              [007]
operation_logs         → System operation audit trail                   [008]
contracts              → Contract header                                [009]
contract_items         → Contract line items                            [009]
contract_payments      → Contract payment milestones                    [009]
refresh_tokens         → Server-side hashed refresh token sessions       [014]
```

> Migrations `010`–`013` seed reference data and add columns (e.g. `011` adds the
> sales-order rejection reason). Migration `014` creates refresh-token storage, and
> `015` rebuilds contract tables to tighten CHECK constraints.

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
- **RBAC**: 4 roles — `admin`, `warehouse`, `qc`, `sales` — enforced via middleware
- **Rate limiting**: Per-IP throttling on auth endpoints (login/refresh) via middleware
- **Data IO**: Batch import is admin-only; export is limited to `admin`/`warehouse`/`sales`;
  operation logs are admin-only. CSV/XLSX exports escape spreadsheet formula prefixes
  (`=`, `+`, `-`, `@`) so exported user-controlled values open as text.
- **Data**: Soft deletes on all business entities, audit trail via `inventory_logs` and `operation_logs`

---

## 📁 Project Structure

```
steel_pipe_db/
├── backend/
│   ├── src/
│   │   ├── main.rs           # Entry point, server startup
│   │   ├── lib.rs             # Module declarations
│   │   ├── router.rs          # Route definitions (~70 endpoints)
│   │   ├── config.rs          # Environment config
│   │   ├── error.rs           # AppError with ApiResponse mapping; ApiErrorResponse includes success + request_id
│   │   ├── response.rs        # ApiResponse<T> / PaginatedResponse<T> / Meta struct with request_id (uuid v4)
│   │   ├── domain/            # Domain enums & constants (pipe specs, etc.)
│   │   ├── dto/               # Request/Response DTOs
│   │   ├── models/            # DB models (25 tables)
│   │   ├── repositories/      # SQL query layer
│   │   ├── services/          # Business logic layer
│   │   ├── handlers/          # Axum request handlers
│   │   └── middleware/        # Auth + RBAC middleware
│   ├── migrations/            # SQLx migrations
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── api/               # Axios API clients
│   │   ├── features/          # Per-module: auth, pipes, inventory, purchases...
│   │   ├── layouts/           # MainLayout with sidebar
│   │   ├── stores/            # Zustand stores
│   │   ├── routes/            # react-router route config
│   │   ├── shared/            # Shared components & hooks
│   │   ├── i18n/              # zh/en locales
│   │   ├── types/             # Global TypeScript types
│   │   └── styles/            # Global styles
│   ├── package.json
│   └── vite.config.ts
├── docs/                      # Design & operations docs
│   ├── requirements.en.md     # PRD (English)
│   ├── detailed-design.en.md  # Architecture + DB + API design (English)
│   ├── frontend-design.en.md  # Frontend component tree & routing (English)
│   ├── 需求文档.md             # PRD (中文)
│   ├── 详细设计文档.md         # Architecture + DB + API design (中文)
│   ├── 前端设计文档.md         # Frontend design (中文)
│   ├── deployment.md          # Deployment guide (Nginx, Docker, backup)
│   ├── troubleshooting.md     # Troubleshooting (DB locks, JWT, CORS)
│   └── tasks/                 # Task breakdown (~320 items)
└── .github/workflows/
    └── ci.yml                 # CI: cargo check + tsc + vite build
```

---

## 🌐 API Overview

All endpoints live under `/api/v1/`:

| Group       | Prefix              | Auth Required |
|-------------|---------------------|:---:|
| Auth        | `/auth/*`           | Mixed |
| Users       | `/users/*`          | Admin only |
| Pipes       | `/pipes/*`          | Yes |
| Inventory   | `/inventory/*`      | Yes |
| Suppliers   | `/suppliers/*`      | Yes |
| Customers   | `/customers/*`      | Yes |
| Purchases   | `/purchase-orders/*`| Yes |
| Sales       | `/sales-orders/*`   | Yes |
| Quality     | `/quality/*`        | Yes |
| Contracts   | `/contracts/*`      | Yes |
| Reports     | `/reports/*`        | Yes |
| Labels      | `/labels/*`         | Yes |
| Data IO     | `/data-io/*`        | Yes |

Every response follows the same shape:
```json
{ "success": true, "request_id": "req_...", "data": { ... } }
```
Paginated responses tack on `meta: { total, page, page_size, total_pages }`. Error responses flip `success: false` and still include `request_id`.

---

## 🔑 RBAC Permission Matrix

| API Group | admin | warehouse | qc | sales |
|-----------|:-----:|:---------:|:---:|:-----:|
| Users (write) | ✅ | ❌ | ❌ | ❌ |
| Pipes (write) | ✅ | ✅ | ❌ | ❌ |
| Inbound/Outbound (write) | ✅ | ✅ | ❌ | ❌ |
| Quality (write) | ✅ | ❌ | ✅ | ❌ |
| Sales Orders (write) | ✅ | ❌ | ❌ | ✅ |
| Purchase Orders (write) | ✅ | ✅ | ❌ | ✅ |
| Suppliers/Customers (write) | ✅ | ✅ | ❌ | ✅ |
| Contracts (write) | ✅ | ✅ | ❌ | ✅ |
| Data Import | ✅ | ❌ | ❌ | ❌ |
| Data Export | ✅ | ✅ | ❌ | ✅ |
| Data IO Operation Logs | ✅ | ❌ | ❌ | ❌ |
| Labels (write) | ✅ | ✅ | ❌ | ❌ |
| General read endpoints | ✅ | ✅ | ✅ | ✅ |

---

## 🧭 Design Docs

Design docs (in Chinese) live in [`docs/`](./docs/):

| Document | Content |
|----------|---------|
| `requirements.en.md` | Full PRD: features, API 5CT standards, roadmap |
| `detailed-design.en.md` | Architecture, 25-table DB schema, REST API, security |
| `frontend-design.en.md` | Component tree, routing, state, i18n, theme |
| `需求文档.md` | PRD 中文版 |
| `详细设计文档.md` | 架构设计中文版 |
| `前端设计文档.md` | 前端设计中文版 |
| `deployment.md` | Deployment guide: production config, Nginx, Docker, backup |
| `troubleshooting.md` | Troubleshooting: database locks, JWT, CORS, migrations |
| `tasks/progress.md` | Master task tracking (~320 items across 3 phases) |

Also see: [`CONTRIBUTING.md`](../CONTRIBUTING.md) · [`CHANGELOG.md`](../CHANGELOG.md)

---

## 📄 License

[GNU General Public License v2](./LICENSE)
