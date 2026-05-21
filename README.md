<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/API-5CT-1f2937?style=flat-square&logo=rust&logoColor=white">
  <img alt="API 5CT" src="https://img.shields.io/badge/API-5CT-1f2937?style=flat-square&logo=rust&logoColor=white">
</picture>

# Steel Pipe DB — API 5CT 无缝钢管 & 筛管库存管理系统

> Oil & gas industry inventory management system for API 5CT seamless steel pipe and screen pipe. Built with Rust + React.

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
cp .env.example .env    # or create with: DATABASE_URL=sqlite://./data/steel_pipe.db?mode=rwc
cargo run               # starts on http://localhost:3000
```

### Frontend

```bash
cd frontend
npm install
npm run dev             # starts on http://localhost:5173
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

**Architecture Pattern:** Handler → Service → Repository → Domain

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
| Inventory  | Per-pipe granular tracking, ATP calculation, inventory logs |

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

19 tables in SQLite (WAL mode, no FK constraints — integrity enforced at application layer):

```
pipes                → Master pipe data (API 5CT specs)
inventory            → Current stock by pipe spec
inventory_logs       → Per-pipe movement audit trail
suppliers            → Supplier master
customers            → Customer master
purchase_orders      → PO header
purchase_order_items → PO line items
sales_orders         → SO header
sales_order_items    → SO line items
inbound_records      → Inbound header (purchase, production, return, transfer)
inbound_record_items → Inbound line items
outbound_records     → Outbound header (sales, scrapped, transfer)
outbound_record_items→ Outbound line items
quality_certificates → QC certificates
quality_mechanical   → Mechanical test results
quality_ndt          → NDT (UT/MI/MPI) results
contracts            → Contract header
contract_milestones  → Payment/delivery milestones
users               → System users (4 roles)
```

All timestamps are ISO 8601 text; soft deletes via `deleted_at`.

---

## 🧪 Development

```bash
# Backend
cd backend && cargo check           # Type-check only (faster than build)
cargo test                           # Run tests
cargo build                          # Debug build
cargo build --release                # Release build

# Frontend
cd frontend && npx tsc --noEmit     # TypeScript type check
npm run build                        # Production build
npm run lint                         # ESLint
```

---

## 🔐 Security

- **Password**: Argon2id with recommended params (`m=19456, t=2, p=1`)
- **Auth**: JWT with configurable expiration, refresh token rotation
- **RBAC**: 4 roles — `admin`, `warehouse`, `qc`, `sales` — enforced via middleware
- **Data**: Soft deletes on all business entities, audit trail via `inventory_logs`

---

## 📁 Project Structure

```
steel_pipe_db/
├── backend/
│   ├── src/
│   │   ├── main.rs           # Entry point, server startup
│   │   ├── lib.rs             # App state, shared types
│   │   ├── router.rs          # Route definitions (~50 endpoints)
│   │   ├── config.rs          # Environment config
│   │   ├── error.rs           # AppError with ApiResponse mapping
│   │   ├── response.rs        # ApiResponse<T> / PaginatedResponse<T>
│   │   ├── domain/            # Domain enums & constants (pipe specs, etc.)
│   │   ├── dto/               # Request/Response DTOs
│   │   ├── models/            # DB models (19 tables)
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
├── docs/                      # Design docs (in Chinese)
│   ├── 需求文档.md            # PRD
│   ├── 详细设计文档.md         # Architecture + DB + API design
│   ├── 前端设计文档.md          # Frontend component tree & routing
│   └── tasks/                 # Task breakdown (~320 items)
└── .github/workflows/
    └── ci.yml                 # CI: cargo check + tsc + vite build
```

---

## 🌐 API Overview

All endpoints under `/api/v1/`:

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
| Data IO     | `/data/*`           | Yes |

All responses wrapped in:
```json
{ "success": true, "data": { ... } }
```

---

## 🧭 Design Docs

Design documents (in Chinese) are in [`docs/`](./docs/):

| Document | Content |
|----------|---------|
| `需求文档.md` | Full PRD: features, API 5CT standards, roadmap |
| `详细设计文档.md` | Architecture, 19-table DB schema, REST API, security |
| `前端设计文档.md` | Component tree, routing, state, i18n, theme |
| `tasks/progress.md` | Master task tracking (~320 items across 3 phases) |

---

## 📄 License

[MIT](./LICENSE)
