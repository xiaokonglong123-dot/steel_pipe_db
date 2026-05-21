# Backend — Rust Package

## Tech
- **Rust** nightly-2024-02-08, edition 2021
- **Cargo workspace** (single crate `db_backend`, no workspace member subdirs)
- **SQLx** 0.8 with SQLite, compile-time query checking via `sqlx-data.json`

## Key Dependencies (from Cargo.toml)
- `axum` 0.7 — HTTP routing
- `sqlx` 0.8 — SQL (sqlite, runtime-tokio, derive features)
- `serde` / `serde_json` — JSON
- `jsonwebtoken` — JWT auth
- `bcrypt` — Password hashing
- `validator` — Request validation (derive feature)
- `rust_decimal` — Monetary/quantity calculations
- `bigdecimal` — Precision decimal
- `chrono` — Date/time (serde feature)
- `tokio` — Async runtime (full features)
- `tower-http` — CORS, TraceLayer
- `backpack` — Validation extras

## Build & Test
```bash
cd backend
cargo build          # Debug build (~30s)
cargo build --release # Release build
cargo test           # Run all tests
cargo sqlx prepare   # Regenerate sqlx-data.json after query changes
```

## Database
- **SQLite** file at `backend/db.db` (auto-created)
- **Migrations**: `backend/migrations/` — SQLx timestamp-prefixed files
- Run migrations automatically on startup (embed_migrations! / sqlx::migrate!)
- No external DB server needed

## Module Structure

```
src/
├── bin/main.rs         ← Entry point: build router, start server
├── lib.rs              ← Module declarations
├── app_state.rs        ← Shared AppState (DB pool, AppConfig)
├── error.rs            ← AppError enum, IntoResponse impl
├── router.rs           ← Build router, mount all handler routes
├── auth.rs             ← JWT middleware, claims struct
├── handlers/           ← 13 files, 110+ handler functions
│   ├── mod.rs          ← pub mod declarations
│   ├── auth_handler.rs
│   ├── pipe_handler.rs
│   ├── inventory_handler.rs
│   └── ...             ← 1 file per entity
├── services/           ← 13 files, business logic
│   ├── mod.rs
│   ├── auth_service.rs
│   ├── pipe_service.rs
│   ├── inventory_service.rs  ← 764 lines (largest service)
│   └── ...
├── repositories/       ← 14 files, SQL layer
│   ├── mod.rs
│   ├── pipe_repo.rs
│   ├── inventory_repo.rs     ← 755 lines (largest repo)
│   ├── report_repo.rs        ← 586 lines
│   └── ...
├── models/             ← 11 files, DB row structs
│   ├── mod.rs
│   ├── pipe.rs
│   ├── inventory.rs
│   └── ...
├── dto/                ← 14 files, request/response structs
│   ├── mod.rs
│   ├── pipe_dto.rs
│   └── ...
├── domain/             ← 4 files, enums/domain types
│   └── mod.rs
└── middleware/         ← 3 files, auth + RBAC
    ├── mod.rs
    └── auth_middleware.rs
```

## Key Files
- `Cargo.toml` — Package manifest
- `build.rs` — Tauri/Vite build hook (steers `FRONTEND_DIST` for embedded frontend)
- `sqlx-data.json` — Prepared query cache (for compile-time checking)
- `db.db` — SQLite database file (gitignored)

## Rust Conventions
- `snake_case` for functions/variables, `PascalCase` for types
- `use` statements: `use crate::{handlers, models, ...}` pattern
- `mod.rs` files re-export public items: `pub use pipe_handler::*;`
- Public API functions are `pub async fn` with explicit return types
- Internal helpers are `pub(crate) fn` or `async fn`
- All handlers return `impl IntoResponse` (Axum pattern)
- Services return `Result<T, AppError>`
- Repositories accept `&Pool<Sqlite>` and return `Result<Vec<T>, sqlx::Error>`
