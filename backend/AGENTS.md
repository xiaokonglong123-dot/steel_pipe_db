# Backend — Rust Package (erp-server)

> 历史沿革 / History: This system was refactored from a steel-pipe industry system; all legacy modules and terminology are deprecated.

## Tech

- **Rust** stable channel (`rust-toolchain.toml`), edition 2021
- **Single crate** `erp-server` (no workspace, no monorepo nonsense)
- **SQLx** 0.8 with SQLite (runtime-tokio), migrations auto-run on startup

## Key Dependencies (from Cargo.toml)

- `axum` 0.8 — HTTP routing (macros + multipart features)
- `sqlx` 0.8 — SQL (sqlite, runtime-tokio, chrono features)
- `serde` / `serde_json` — JSON
- `jsonwebtoken` 9 — JWT auth
- `argon2` 0.5 — Password hashing (NOT bcrypt)
- `validator` 0.19 — Request validation (derive feature)
- `chrono` 0.4 — Date/time (serde feature)
- `tokio` 1 — Async runtime (full features)
- `tower-http` 0.6 — CORS, TraceLayer, request-id
- `tower` 0.5 — Utilities
- `uuid` 1 — UUID generation (v4 feature)
- `dotenvy` 0.15 — .env loading
- `thiserror` 2 — Error derive macro
- `calamine` 0.26 — Excel import
- `rust_xlsxwriter` 0.80 — Excel export
- `csv` 1.3 — CSV import/export
- `tracing` / `tracing-subscriber` — Structured logging (env-filter, json)

**Heads up:** `rust_decimal` + `rust_decimal_macros` are used in the amount input layer (DTOs, `domain/money.rs`); DB columns and calculations still use `f64`. There's no `bigdecimal`, `backpack`, or `bcrypt` here — don't go looking for them.

## Build & Test

```bash
cd backend
cargo check          # Type-check only (faster than build, CI uses this)
cargo build          # Debug build
cargo build --release # Release build
cargo test           # Run all tests
```

## Database

- **SQLite** file at `sqlite://data/erp.db?mode=rwc` (`DATABASE_URL` env var; the file is created on first run)
- **Migrations**: `backend/migrations/` — SQLx timestamp-prefixed files. The 37 legacy migration files are being rewritten to SQLite syntax, and the legacy pipe tables are dropped.
- Auto-migrate on startup via `sqlx::migrate!("./migrations")`
- No external DB server needed — it's just a file
- WAL mode enabled, soft deletes via `deleted_at` column

## Module Structure

```
src/
├── main.rs              ← Entry point: tracing, DB pool, migrate, start server
├── lib.rs               ← Module declarations, #![allow(dead_code)]
├── config.rs            ← Env-based Config (DATABASE_URL, JWT_SECRET, etc.)
├── error.rs             ← AppError enum with numeric error codes (10001-50001)
├── response.rs          ← ApiResponse<T>, PaginatedResponse<T>
├── router.rs            ← ~190 routes (~170 unique paths) assembled via .merge()
├── cache.rs             ← Response cache (locations)
├── domain/              ← Generic domain types (item, inventory, order, money)
├── dto/                 ← Request/response structs (one file per entity)
├── models/              ← DB row structs (sqlx::FromRow)
├── items/               ← Item (商品) master: handlers.rs + services.rs + repos.rs
├── inventory/           ← Stock, inbound/outbound, locations, checks, trace, legacy ATP
├── orders/              ← Purchase & sales orders
├── contracts/           ← Contract terms & payments
├── parties/             ← Suppliers & customers
├── reports/             ← Aggregations & dashboards
├── data_io/             ← Import/export & operation logs
├── health.rs            ← Health/readiness endpoints
├── utils.rs             ← Order number generation + status transition validation
├── operation_log.rs     ← Operation log repo
├── macros.rs            ← party_handler! / party_service! (suppliers/customers shared)
├── middleware/          ← auth, rbac, rate_limit, security_headers
├── auth/                ← RBAC (roles/permissions/departments/tenants) + legacy login/users (handlers_legacy, services_legacy, repos_legacy, refresh_token_repo)
├── workflow/            ← Approval engine: definitions / instances / tasks
├── hr/                  ← Employees / attendance / salaries / labor contracts
├── finance/             ← Accounts / journal / invoices / payments / trial balance
├── procurement/         ← Requisitions / receipts / quotes / scorecard
├── sales_crm/           ← Shipments / quotes / customer credit
├── inventory_atp/       ← 商品/SKU inventory: reservations / transfers / count sessions
├── manufacturing/       ← BOMs / work orders / inspections / NCRs
├── project/             ← Projects / WBS / budget
├── assets/              ← Fixed assets: registration / depreciation / disposal
├── notification/        ← Inbox / templates / preferences
├── portal/              ← Portal accounts / party JWT / PO accept / SO ack
└── bi/                  ← Sales trend / inventory value / finance summary / supplier perf
```

Every feature module (`auth/`, `workflow/`, `hr/`, …) follows the same layout: `mod.rs` + `handlers.rs` + `repos.rs` + `services.rs` (`bi/` has no `repos.rs` — read-only analytics over the shared repositories; `items/`/`orders/`/`contracts/`/`parties/`/`inventory/`/`reports/`/`data_io/` follow the same layout).

Core layers in detail:

```
├── domain/              ← Generic domain types (item, inventory, order, money)
├── dto/                 ← auth_dto, item_dto, inventory_dto, purchase_dto, sales_dto,
│                          contract_dto, customer_dto, supplier_dto, report_dto, data_io_dto, common, …
├── models/              ← DB row structs: user, rbac, item, inventory, purchase_order,
│                          sales_order, contract, customer, supplier, workflow, hr, finance,
│                          procurement, sales_crm, inventory_atp, manufacturing, project,
│                          assets, notification, portal
├── items/               ← handlers.rs, services.rs, repos.rs
├── inventory/           ← atp/check/inbound/inventory/location/outbound handlers+services, repos
├── orders/              ← purchase+sales handlers+services, purchase_order/sales_order repos
├── contracts/           ← contract handler+service, contract_repo
├── parties/             ← customer+supplier handlers+services, customer/supplier repos
├── reports/             ← report handler+service, report_repo
├── data_io/             ← data_io handler+service, data_io_repo
├── health.rs            ← health/readiness endpoints
├── utils.rs             ← generate_no + validate_status_transition (orders)
├── operation_log.rs     ← operation log repo
└── middleware/          ← auth.rs (JWT + AuthenticatedUser extractor), rbac.rs, rate_limit.rs, security_headers.rs
```

Inventory is generalized to **商品/SKU**: the item master table carries `sku` / name / category / unit / optional spec — no industry-specific fields. Reservations, transfers, and count sessions live in `inventory_atp/`.

## Key Files

- `Cargo.toml` — Package manifest (crate `erp-server`)
- `.env.example` — Environment template (DATABASE_URL, JWT_SECRET, etc.)
- `migrations/` — SQLx timestamp-prefixed migration files (rewritten to SQLite syntax; pipe tables dropped)

## Rust Conventions

- `snake_case` for functions/variables, `PascalCase` for types
- `use` statements follow `use crate::{handlers, models, ...}` pattern
- `mod.rs` files re-export public items: `pub use item_handler::*;`
- Public API functions are `pub async fn` with explicit return types
- Internal helpers are `pub(crate) fn` or `async fn`
- **All handlers return `Result<Json<...>, AppError>`** (NOT `impl IntoResponse`)
- Services are **unit structs with static methods** (no constructor DI): `ItemService::list(...)`
- Services return `Result<T, AppError>`
  - Repositories accept `&SqlitePool` and return `Result<Vec<T>, sqlx::Error>`
- The inventory service layer is split into focused modules:
  - `inbound_service.rs` — inbound (入库) record creation, approval, batch execution
  - `outbound_service.rs` — outbound (出库) record creation, approval, stock deduction
  - `check_service.rs` — inventory count session (盘点) creation, item submission, completion
  - `inventory_query_service.rs` — read-only queries (list, statistics)
  - `location_service.rs` — warehouse location CRUD, assign, transfer
- Purchase and sales are split into:
  - `purchase_service.rs` — purchase order (采购订单) lifecycle, approval, rejection
  - `sales_service.rs` — sales order (销售订单) lifecycle, ATP validation, approval
- ATP calculation lives in `sales_service.rs` and `atp_handler.rs`

## DI Pattern: Extension layers, NOT State<Arc<AppState>>

```rust
// router.rs layers:
.layer(CorsLayer::permissive())
.layer(TraceLayer::new_for_http())
.layer(Extension(pool))       // Extension<SqlitePool>
.layer(Extension(JwtSecret(jwt_secret))) // Extension<JwtSecret>

// Handler extracts:
pub async fn list_items(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<ItemFilterParams>,
) -> Result<Json<PaginatedResponse<Item>>, AppError> {
```

No `AppState` struct. The DB pool is injected directly; the JWT secret is wrapped in `JwtSecret` so it is type-safe, has redacted `Debug`, and cannot be confused with arbitrary string extensions.

## Response Shapes

```json
// Success:    { "success": true, "request_id": "req_...", "data": T }
// Paginated:  { "success": true, "request_id": "req_...", "meta": { "total": N, "page": P, "page_size": S, "total_pages": N }, "data": { "items": [], ... } }
// Error:      { "success": false, "code": 11001, "request_id": "req_...", "message": "...", "details": null }
```

`tower-http` also sets/propagates an `x-request-id` header, and CORS exposes it to the frontend.

## Error Codes (numeric, domain-prefixed)

| Range | Domain |
| ------- | -------- |
| 100xx | General (Internal, Validation, NotFound) |
| 110xx | Auth (Unauthorized, TokenExpired, Forbidden) |
| 120xx | Item 商品 (NotFound, Duplicate, StatusConflict) |
| 130xx | Inventory (InsufficientStock, LocationNotFound) |
| 140xx | Orders (CannotModify, NotFound) |
| 150xx | Inspection 质检 (NotFound, StatusConflict) |
| 160xx | Supplier (NotFound, CodeDuplicate) |
| 170xx | Customer (NotFound, CodeDuplicate) |
| 180xx | Data IO (ImportError, ExportError) |
| 50001 | Database |
