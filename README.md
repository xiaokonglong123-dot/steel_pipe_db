# Ikari_Shinji — ERP

<div align="center">

> A general-purpose ERP (Enterprise Resource Planning) system. Rust + Axum backend, Vue 3 + Element Plus frontend, SQLite storage.

![Rust](https://img.shields.io/badge/Rust-Axum_0.8-000000?style=flat-square&logo=rust&logoColor=white)
![Vue](https://img.shields.io/badge/Vue-3-4FC08D?style=flat-square&logo=vue.js&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-3-003B57?style=flat-square&logo=sqlite&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript&logoColor=white)
![Element Plus](https://img.shields.io/badge/Element_Plus-409EFF?style=flat-square&logo=element&logoColor=white)

![Tests](https://img.shields.io/badge/Backend%20tests-121%20passing-brightgreen?style=flat-square)
![Build](https://img.shields.io/badge/Frontend%20build-passing-brightgreen?style=flat-square)

</div>

---

## Overview

A modular ERP covering the core transactional loop for a single plant:
items (SKU master) → inventory (with inbound/outbound + audit trail) → procurement (purchase orders) → sales (sales orders + ATP reservations) → finance (GL/journal/invoice/payment) → reports — all gated by a data-driven approval workflow and JWT+RBAC auth.

The system is the **erp-v2 era** rewrite of the original React-stack ERP (now archived on `legacy/steel-pipe-react`). It is a single-plant, single-instance deployment with SQLite as the single source of truth — designed for small teams and zero infrastructure overhead.

---

## Tech Stack

### Backend — Rust (Axum 0.8)

| Layer        | Technology                                                |
|--------------|-----------------------------------------------------------|
| Framework    | Axum 0.8 with macros + multipart                          |
| ORM          | SQLx 0.8 (SQLite, `sqlite` + `chrono` + `uuid` features)   |
| Auth         | JWT (`jsonwebtoken` 9) + Argon2id password hashing         |
| Money        | `rust_decimal::Decimal` — full-chain precision (DB `TEXT`)|
| Validation   | `validator` 0.19 (derive)                                 |
| Logging      | `tracing` + `tracing-subscriber` (env-filter, json)       |
| Excel/CSV    | `csv` 1.3 for batch import                                 |
| Middleware   | `tower-http` (CORS, trace, request-id), cookie-based auth  |
| Database     | SQLite3 single file (`backend/data/erp.db`, WAL mode)      |

### Frontend — Vue 3 + Element Plus

| Category       | Library                                                       |
|----------------|---------------------------------------------------------------|
| UI Framework   | Vue 3 + Element Plus 5 + ECharts (line + bar visualizations) |
| State          | Pinia (client) + TanStack Vue Query (server state)            |
| HTTP Client    | Native `fetch` wrapper (`api/client.ts`)                     |
| Routing        | Vue Router 4 with permission-guarded routes                   |
| Build Tool     | Vite 6 + `bun` package manager                               |
| Type Safety    | TypeScript 5 + `vue-tsc`                                     |

---

## Quick Start

### Prerequisites

| Tool  | Version     | Notes                              |
|-------|-------------|------------------------------------|
| Rust  | 1.78+       | edition 2021                       |
| bun   | 1.x         | replaces npm (npm path unavailable) |
| SQLite| 3.35+       | bundled via sqlx                   |

### Backend

```bash
cd backend
cp .env.example .env       # or roll your own: DATABASE_URL=sqlite://data/erp.db?mode=rwc
cargo run                  # fires up on http://localhost:3000
```

The backend crate is `erp-v2`. The database is a single SQLite3 file (`data/erp.db`) — no external DB server required.

### Frontend

```bash
cd frontend
bun install                # do NOT use npm — installed deps diverge
bun run dev                # fires up on http://localhost:5173
```

Open `http://localhost:5173` and log in with:

| Username | Password    |
|----------|-------------|
| `admin`  | `admin123`  |

---

## Build & Verify

| What                | How                                            |
|---------------------|------------------------------------------------|
| Backend type-check  | `cd backend && cargo check --all-targets`      |
| Backend tests       | `cd backend && cargo test --all`               |
| Frontend type-check | `cd frontend && bunx tsc --noEmit`             |
| Frontend build      | `cd frontend && bun run build`                 |

- **Backend**: 121 tests green
- **Frontend**: `bunx tsc --noEmit` + `bun run build` green

---

## Modules

### Phase P0 — Core Transactional Loop (✅ Complete)

| Module                                         | Description                                                                         | Tests |
|------------------------------------------------|-------------------------------------------------------------------------------------|-------|
| Auth + RBAC                                    | JWT login/refresh/logout, real-time DB-backed permission check, hashed refresh tokens | 4     |
| Catalog (Items)                                | SKU master (sku, name, category, unit, spec), draft/active/disabled lifecycle        | 8     |
| Parties                                        | Suppliers + Customers with soft delete                                             | 9     |
| Warehouses + Locations + Inventory + Logs      | Materialized `inventory` (balance) + `inventory_logs` (event audit trail)           | 22    |
| Purchase Orders                                | PO lifecycle (draft → submitted → approved/rejected), Decimal money                 | 13    |
| Sales Orders + ATP Reservations                | SO + reservation-driven Available-To-Promise check                                  | 14    |
| Workflow (Data-Driven, ERPNext-style)           | `workflows` / `workflow_states` / `workflow_transitions` — new nodes by INSERT only  | 13    |
| Receipt + Shipment (E2E)                       | End-to-end inbound/outbound linkage with PO/SO                                     | 9     |
| **E2E** PO + SO                                | End-to-end procurement → inventory → sales loop                                     | 2     |

### Phase P1 — Finance + Reports + ATP (✅ Complete)

| Module                              | Description                                                                                  | Tests |
|-------------------------------------|----------------------------------------------------------------------------------------------|-------|
| Finance                             | GL accounts, journal entries, invoices, payments, trial balance                              | —     |
| Inventory Check                     | Count sessions with balance reconciliation                                                   | 5     |
| ATP `available_qty`                 | Dedicated endpoint + service for available-to-promise                                        | 1     |
| Reports                             | inventory_summary / inbound_outbound / sales_trend / finance_summary + CSV export           | —     |
| **P1 E2E**                          | finance + check + ATP reservation release                                                    | 4     |

### Phase P2 — Enhancements (✅ Complete)

| Module                                  | Description                                                              | Tests |
|-----------------------------------------|--------------------------------------------------------------------------|-------|
| CSV Item Import                         | `POST /items/import` (multipart) with per-row report                     | 2     |
| Workflow Multi-Level + Conditional      | `amount_threshold` + `transition_with_amount` (path picker by business amount) | 2     |
| Migration Validation                    | finance.threshold `012_workflow_threshold.sql`                          | 1     |

---

## Data Model

SQLite3 single file (WAL mode). 12 migrations:

```
001_auth_rbac.sql           — users / roles / role_permissions / operation_logs / refresh_tokens
002_catalog.sql             — items (SKU master, draft/active/disabled)
003_parties.sql             — suppliers / customers
004_inventory.sql           — warehouses / locations / inventory / inventory_logs
005_purchasing.sql          — purchase_orders / purchase_order_items
006_sales.sql               — sales_orders / sales_order_items / reservations
007_finance.sql             — accounts / journal_entries / journal_lines / invoices / payments
008_workflow.sql            — workflows / workflow_states / workflow_transitions / workflow_instances / workflow_tasks
009_seed.sql                — admin/manager/finance roles + 11 permissions
010_warehouses.sql          — ALTER locations.warehouse_id + deleted_at (parent-table introduction)
011_seed_workflows.sql      — PO/SO demo workflows + states + transitions
012_workflow_threshold.sql  — ALTER workflow_transitions.amount_threshold TEXT
```

Integrity is enforced at the application layer (TOCTOU-safe, transactional). Soft deletes via `deleted_at` — records are never physically destroyed. See [detailed-design.md](./docs/detailed-design.md) for the full schema.

---

## Key Design Decisions

- **Money 全链 Decimal**: `rust_decimal::Decimal`, stored as `TEXT` in SQLite, `Decimal::to_string()` for serialization. **No SQL SUM over TEXT money columns** — aggregate in app layer. Date-group REAL aggregation is allowed (ADR-002 exception).
- **Inventory Dual-Track**: materialized `inventory` (balance) + `inventory_logs` (event audit trail). Inbound writes +OUT check on balance.
- **Workflow Data-Driven**: `workflows` / `workflow_states` / `workflow_transitions` — adding a node does not require code changes, only an `INSERT` row.
- **Conditional Workflow by Amount**: `workflow_transitions.amount_threshold TEXT`; `workflow_service::transition_with_amount(...)` picks the first transition whose threshold is satisfied, falling back to NULL-threshold transitions otherwise.
- **RBAC Real-Time DB Lookup**: JWT carries `user_id` + `permissions`, but `auth_middleware` re-queries the DB on every request to inject a fresh `AuthUser` extension (no stale permission issues).
- **Spec Drift Handling**: New migrations never modify already-executed migrations (project rule #234) — `010_warehouses` / `011_seed_workflows` / `012_workflow_threshold` are all fresh migration files layered on top, not edits to existing ones.

---

## Project Structure

```
Ikari_Shinji/
├── backend/                                # erp-v2 crate (Rust Axum)
│   ├── src/
│   │   ├── main.rs                          # Entry point, server startup
│   │   ├── lib.rs                           # Module declarations
│   │   ├── config.rs                        # Environment config
│   │   ├── error.rs / response.rs           # AppError + ApiResponse/PaginatedResponse
│   │   ├── auth/                            # JWT login, refresh, logout, bootstrap admin
│   │   ├── http/                            # Handlers grouped by domain (purchase.rs, sales.rs, ...)
│   │   ├── services/                        # Business logic (purchase_service.rs, workflow_service.rs, ...)
│   │   ├── repos/                          # sqlx repositories
│   │   ├── middleware/                     # auth + rbac middleware
│   │   └── domain/                         # Domain enums, validation helpers
│   ├── tests/                              # 16 integration-test files (121 tests total)
│   ├── migrations/                         # 12 SQLx migrations
│   ├── Cargo.toml / Cargo.lock / .env.example / rust-toolchain.toml
│   └── data/erp.db                         # SQLite3 (gitignored, auto-created)
├── frontend/
│   ├── src/
│   │   ├── main.ts                         # Vue app + Pinia + Router + Vue Query
│   │   ├── App.vue / router/               # Routing with permission guards
│   │   ├── api/                            # fetch client + queryClient + per-domain api files
│   │   ├── views/                          # Per-domain pages (auth/items/purchases/...)
│   │   ├── stores/                         # Pinia stores
│   │   ├── components/                     # Shared Element Plus-based components
│   │   └── styles/                         # Global styles + el-plus theme overrides
│   ├── package.json / bun.lock / vite.config.ts / tsconfig.json / DESIGN.md
├── docs/                                   # PRD, detailed-design, frontend-design, tasks
│   └── legacy/                             # Pre-rewrite (React-stack) design docs (archived)
├── specs/                                  # Ubiquitous language (terminology canon)
├── .github/workflows/ci.yml                # CI: cargo check + test + bun tsc + build
├── AGENTS.md                               # Authoritative project index
├── README.md / README_zh.md / CHANGELOG.md / CONTRIBUTING.md / LICENSE
└── .local-only-docs/                       # (gitignored) Personal decision docs
```

---

## API Overview

All endpoints live under `/api/v1/` (or shorter where the AGENTS.md notes exceptions). Every response follows the same shape:

```json
{ "success": true, "request_id": "req_...", "data": { ... } }
```

Paginated responses tack on `meta: { total, page, page_size, total_pages }`. Error responses flip `success: false` and still include `request_id`. See [detailed-design.md](./docs/detailed-design.md) for the full route table.

### Auth endpoints

| Endpoint                | Auth  | Purpose                                |
|-------------------------|:-----:|----------------------------------------|
| `POST /auth/login`      | no    | Get access + refresh tokens (cookies)  |
| `POST /auth/refresh`    | tkn   | Rotate refresh token                   |
| `POST /auth/logout`     | tkn   | Revoke all refresh tokens              |
| `GET  /auth/me`         | tkn   | Current user + permissions             |

---

## Security

- **Password**: Argon2id with recommended params (`m=19456, t=2, p=1`)
- **Auth**: Stateless JWT access tokens (HS256). **Refresh tokens are stored server-side** — SHA-256 hashed in the `refresh_tokens` table, rotated on each `/auth/refresh`, and revoked on `/auth/logout` (cascades all sessions for that user).
- **RBAC**: Roles like `admin`/`manager`/`warehouse`/`finance` — enforced via middleware that re-queries the DB per request (no stale permission risk).
- **Money**: `rust_decimal::Decimal` end-to-end — no f64 in business logic.
- **CSV/XLSX Export**: spreadsheet formula prefixes (`=`, `+`, `-`, `@`) are escaped so exported user-controlled values always open as text.
- **Audit**: `inventory_logs` (item movement trail) + `operation_logs` (admin-only operation audit).

---

## History

| Branch                          | Era                                | Head SHA                                                       |
|---------------------------------|------------------------------------|----------------------------------------------------------------|
| `main`                          | erp-v2 (current)                   | `c6a0b62` (Promotion + date-fix) — `9570a29` (origin/main)    |
| `legacy/steel-pipe-react`       | Pre-rewrite React 19 + Antd stack  | `05cbf0d` (51 commits of legacy evolution, pre-erp-v2)        |
| `legacy/react-phase1-4`         | Remote-only React-stack Phase 1-4  | `ccedabe` (preserved from pre-force-push origin/main)          |

The pre-rewrite stack (`React 19 + Ant Design 5 + npm`) is fully recoverable via checkout either legacy branch.

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) and [AGENTS.md](./AGENTS.md) (the authoritative project index). Atomic commits, descriptive messages, no force-push to `main` without coordination. Design docs in `docs/` are the source of truth — keep them in sync when architecture changes.

---

## CI

`.github/workflows/ci.yml` runs on every push / PR to `main`:

- **Backend**: `cargo check --all-targets` + `cargo test --all`
- **Frontend**: `bun install --frozen-lockfile` + `bunx tsc --noEmit` + `bun run build`

---

## License

[GNU General Public License v2](./LICENSE)
