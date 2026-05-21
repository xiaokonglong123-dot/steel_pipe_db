# `backend/src/` — Module Wiring & Router

This directory wires all backend source modules together. It is NOT the place to add new feature files — those go in `handlers/`, `services/`, `repositories/`, etc.

## Module Registration

**To add a new backend module:**
1. Create file in appropriate subdirectory (e.g., `handlers/new_thing_handler.rs`)
2. Add `pub mod new_thing_handler;` to the subdirectory's `mod.rs`
3. Add route in `router.rs`
4. Add handler to the handler `mod.rs`

## `main.rs` — Entry Point
- **File**: `src/main.rs` (NOT `src/bin/main.rs`)
- `#![allow(dead_code)]` at crate root (suppresses unused code warnings)
- `#[tokio::main]` async entry
- Initializes: logging (tracing), DB pool (SqlitePool), Config from env
- Calls `router::create_app(pool, jwt_secret)` to build the router
- Binds `0.0.0.0:3000`, starts serving
- **No `AppState` struct exists** — DI is via `Extension<>` layers

## Shared State Pattern — `Extension<>` not `State<>`
**This project uses Axum `Extension<>` layers for dependency injection, NOT `State<Arc<AppState>>`.**
```rust
// router.rs
.layer(CorsLayer::permissive())
.layer(TraceLayer::new_for_http())
.layer(Extension(pool))
.layer(Extension(jwt_secret))
```
- `Extension(SqlitePool)` — raw pool (no wrapper struct)
- `Extension(String)` — JWT secret as bare `String` (no newtype)
- Handlers extract via `Extension(pool): Extension<SqlitePool>`

## `router.rs` — Route Mounting
```rust
pub fn create_app(pool: SqlitePool, jwt_secret: String) -> Router {
    // ~50 endpoints, grouped by entity via .merge()
    Router::new()
        .route("/api/v1/auth/login", post(handlers::auth_handler::login))
        .route("/api/v1/pipes", get(handlers::pipe_handler::list))
        // ...
        .merge(pipe_routes)
        .merge(inventory_routes)
        // ...
        .layer(CorsLayer::permissive())
        .layer(Extension(pool))
        .layer(Extension(jwt_secret))
}
```
- Router function **creates** routes, does NOT accept pre-constructed services
- Each handler receives DI via `Extension<SqlitePool>` and `Extension<String>` extractors
- Auth middleware wraps individual sub-routers, not global

## `error.rs` — Error Handling (Numeric Error Codes)
- `AppError` enum has **~20 variants** grouped by domain prefix (100xx–50001)
- Each variant maps to a **numeric `error_code()`** (e.g., `Validation` → 10002) and an **HTTP `status_code()`**
- Uses `thiserror::Error` for `Display` derive
- Implements `IntoResponse` to serialize into `ApiErrorResponse { code, message, details }`
- `From<sqlx::Error>` impl converts DB errors to `AppError::Database`
- All service errors convert via `?` operator with `From` impls

Domain breakdown:

| Range   | Domain       |
|---------|--------------|
| 100xx   | General (Internal, Validation, NotFound, BadRequest) |
| 110xx   | Auth (Unauthorized, TokenExpired, Forbidden) |
| 120xx   | Pipe (NotFound, Duplicate, StatusConflict) |
| 130xx   | Inventory (InsufficientStock, LocationFull) |
| 140xx   | Orders (CannotModify, NotFound) |
| 150xx   | Quality (CertNotFound, AttachmentNotFound) |
| 160xx   | Supplier (NotFound, CodeDuplicate) |
| 170xx   | Customer (NotFound, CodeDuplicate) |
| 180xx   | Data IO (ImportError, ExportError) |
| 50001   | Database |

## `middleware/auth.rs` — JWT Middleware
- **`Claims`** struct — JWT payload (`sub` user_id, `username`, `role`, `exp`, `iat`)
- **`AuthContext`** extractor — extracted from validated JWT token (contains `user_id`, `username`, `role`)
- **`auth_middleware`** — Axum middleware layer that validates Bearer token from `Authorization` header
  - Reads JWT secret from request extensions
  - Decodes token with HS256 via `jsonwebtoken`
  - On success: inserts `AuthContext` into request extensions
  - On failure: returns 401 with numeric error code (11001/11002)
- Middleware wraps individual sub-routers (not global)
- Token generation logic lives in **handler/service layer** (not in middleware)
