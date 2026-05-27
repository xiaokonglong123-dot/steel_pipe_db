# Backend — Rust Package (steel-pipe-db)

## Tech

- **Rust** nightly-2024-02-08, edition 2021
- **Single crate** `steel-pipe-db` (no workspace, no monorepo nonsense)
- **SQLx** 0.8 with SQLite (runtime-tokio-rustls), migrations auto-run on startup

## Key Dependencies (from Cargo.toml)

- `axum` 0.8 — HTTP routing (macros + multipart features)
- `sqlx` 0.8 — SQL (sqlite, runtime-tokio-rustls, chrono features)
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

**Heads up:** No `rust_decimal`, `bigdecimal`, `backpack`, or `bcrypt` here. Don't go looking for them.

## Build & Test

```bash
cd backend
cargo check          # Type-check only (faster than build, CI uses this)
cargo build          # Debug build
cargo build --release # Release build
cargo test           # Run all tests
```

## Database

- **SQLite** file at path from `DATABASE_URL` env var (defaults to `./data/steel_pipe.db`)
- **Migrations**: `backend/migrations/` — SQLx timestamp-prefixed files
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
├── router.rs            ← ~70 endpoints assembled via .merge()
├── domain/              ← 3 files (pipe.rs, inventory.rs, order.rs) — enums/domain types
│   └── mod.rs
├── dto/                 ← 14 files, request/response structs
│   ├── mod.rs
│   ├── auth_dto.rs
│   ├── pipe_dto.rs
│   ├── inventory_dto.rs
│   ├── purchase_dto.rs
│   ├── sales_dto.rs
│   ├── quality_dto.rs
│   ├── contract_dto.rs
│   ├── customer_dto.rs
│   ├── supplier_dto.rs
│   ├── label_dto.rs
│   ├── report_dto.rs
│   ├── data_io_dto.rs
│   └── common.rs
├── models/              ← 11 files, DB row structs (sqlx::FromRow)
│   ├── mod.rs
│   ├── user.rs
│   ├── seamless_pipe.rs
│   ├── screen_pipe.rs
│   ├── inventory.rs
│   ├── purchase_order.rs
│   ├── sales_order.rs
│   ├── quality.rs
│   ├── contract.rs
│   ├── customer.rs
│   └── supplier.rs
├── repositories/        ← 13 files, pure SQL, soft-delete aware
│   ├── mod.rs
│   ├── pipe_repo.rs
│   ├── inventory_repo.rs
│   ├── purchase_order_repo.rs
│   ├── sales_order_repo.rs
│   ├── quality_repo.rs
│   ├── contract_repo.rs
│   ├── customer_repo.rs
│   ├── supplier_repo.rs
│   ├── label_repo.rs
│   ├── report_repo.rs
│   ├── data_io_repo.rs
│   ├── user_repo.rs
│   └── operation_log_repo.rs
├── services/            ← 16 files, business logic (unit structs, static methods)
│   ├── mod.rs
│   ├── auth_service.rs
│   ├── pipe_service.rs
│   ├── inbound_service.rs       ← Inbound (create/approve/execute/query)
│   ├── outbound_service.rs      ← Outbound (create/approve/execute/query)
│   ├── check_service.rs         ← Inventory checks (create/submit/complete)
│   ├── inventory_query_service.rs ← Read-only inventory queries (list/stats)
│   ├── location_service.rs      ← Warehouse locations (CRUD/assign/transfer)
│   ├── purchase_sales_service.rs ← Purchase & sales orders (shared logic)
│   ├── quality_service.rs
│   ├── contract_service.rs
│   ├── customer_service.rs
│   ├── supplier_service.rs
│   ├── label_service.rs
│   ├── report_service.rs
│   ├── data_io_service.rs
│   └── trace_service.rs         ← Full-lifecycle pipe tracing
├── handlers/            ← 13 files, thin handlers (extract → call service → respond)
│   ├── mod.rs
│   ├── auth_handler.rs
│   ├── pipe_handler.rs
│   ├── inventory_handler.rs
│   ├── purchase_handler.rs
│   ├── sales_handler.rs
│   ├── quality_handler.rs
│   ├── contract_handler.rs
│   ├── customer_handler.rs
│   ├── supplier_handler.rs
│   ├── report_handler.rs
│   ├── label_handler.rs
│   ├── data_io_handler.rs
│   └── atp_handler.rs
└── middleware/          ← 2 files, auth + RBAC
    ├── mod.rs
    ├── auth.rs          ← JWT verification, Claims, AuthContext, auth_middleware
    └── rbac.rs          ← Role-based access control helpers
```

## Key Files

- `Cargo.toml` — Package manifest
- `.env.example` — Environment template (DATABASE_URL, JWT_SECRET, etc.)
- `migrations/` — SQLx timestamp-prefixed migration files (11 files, including `011_add_rejection_reason.sql`)

## Rust Conventions

- `snake_case` for functions/variables, `PascalCase` for types
- `use` statements follow `use crate::{handlers, models, ...}` pattern
- `mod.rs` files re-export public items: `pub use pipe_handler::*;`
- Public API functions are `pub async fn` with explicit return types
- Internal helpers are `pub(crate) fn` or `async fn`
- **All handlers return `Result<Json<...>, AppError>`** (NOT `impl IntoResponse`)
- Services are **unit structs with static methods** (no constructor DI): `PipeService::list(...)`
- Services return `Result<T, AppError>`
  - Repositories accept `&SqlitePool` and return `Result<Vec<T>, sqlx::Error>`
- `inventory_service.rs` has been split into focused modules:
  - `inbound_service.rs` — inbound record creation, approval, batch execution
  - `outbound_service.rs` — outbound record creation, approval, stock deduction
  - `check_service.rs` — inventory check (盘点) creation, item submission, completion
  - `inventory_query_service.rs` — read-only queries (list, statistics)
  - `location_service.rs` — warehouse location CRUD, assign, transfer
  - ATP calculation lives in `purchase_sales_service.rs` and `atp_handler.rs`

## DI Pattern: Extension layers, NOT State<Arc<AppState>>

```rust
// router.rs layers:
.layer(CorsLayer::permissive())
.layer(TraceLayer::new_for_http())
.layer(Extension(pool))       // Extension<SqlitePool>
.layer(Extension(JwtSecret(jwt_secret))) // Extension<JwtSecret>

// Handler extracts:
pub async fn list_pipes(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<PipeFilterParams>,
) -> Result<Json<PaginatedResponse<Pipe>>, AppError> {
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
|-------|--------|
| 100xx | General (Internal, Validation, NotFound) |
| 110xx | Auth (Unauthorized, TokenExpired, Forbidden) |
| 120xx | Pipe (NotFound, Duplicate, StatusConflict) |
| 130xx | Inventory (InsufficientStock, LocationFull) |
| 140xx | Orders (CannotModify, NotFound) |
| 150xx | Quality (CertNotFound, AttachmentNotFound) |
| 160xx | Supplier (NotFound, CodeDuplicate) |
| 170xx | Customer (NotFound, CodeDuplicate) |
| 180xx | Data IO (ImportError, ExportError) |
| 50001 | Database |
