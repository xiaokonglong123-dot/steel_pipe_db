# `backend/src/` — Module Wiring & Router

This directory is where all the backend source modules get wired together. Don't drop new feature files here — those belong in `handlers/`, `services/`, `repositories/`, etc.

## Module Registration

**So you want to add a new module? Here's the drill:**

1. Create the file in the right subdirectory (e.g., `handlers/new_thing_handler.rs`)
2. Add `pub mod new_thing_handler;` to that subdirectory's `mod.rs`
3. Wire up the route in `router.rs`
4. Expose the handler in the handler `mod.rs`

## `main.rs` — Entry Point

- **File**: `src/main.rs` (NOT `src/bin/main.rs` — don't overthink it)
- `#![allow(dead_code)]` lives at the crate root (keeps the compiler quiet about legit unused stuff)
- `#[tokio::main]` async entry point
- Initializes: tracing (logging), DB pool (SqlitePool), Config from env vars
- Calls `router::create_app(pool, jwt_secret)` to build the router
- Binds `0.0.0.0:3000`, starts serving
- **No `AppState` struct** — we use `Extension<>` layers for DI

## Shared State Pattern — `Extension<>` not `State<>`

**This project rocks Axum `Extension<>` layers for dependency injection, not `State<Arc<AppState>>`.**

```rust
// router.rs
.layer(CorsLayer::permissive())
.layer(TraceLayer::new_for_http())
.layer(Extension(pool))
.layer(Extension(JwtSecret(jwt_secret)))
```

- `Extension(SqlitePool)` — raw pool, no wrapper struct
- `Extension(JwtSecret)` — JWT secret newtype with redacted `Debug`; missing extension fails closed instead of using an empty fallback
- Handlers grab what they need via `Extension(pool): Extension<SqlitePool>`

## `router.rs` — Route Mounting

```rust
pub fn create_app(pool: SqlitePool, jwt_secret: String) -> Router {
    // ~70 endpoints, grouped by entity via .merge()
    Router::new()
        .route("/api/v1/auth/login", post(handlers::auth_handler::login))
        .route("/api/v1/pipes", get(handlers::pipe_handler::list))
        // ...
        .merge(pipe_routes)
        .merge(inventory_routes)
        // ...
        .layer(CorsLayer::permissive())
        .layer(Extension(pool))
        .layer(Extension(JwtSecret(jwt_secret)))
}
```

- The router function **creates** routes fresh — doesn't accept pre-built services
- Handlers get DI through `Extension<SqlitePool>` and auth handlers use `Extension<JwtSecret>`
- Auth middleware wraps individual sub-routers, not the whole app
- Request ID middleware sets/propagates `x-request-id`; CORS exposes that header to the browser

## `error.rs` — Error Handling (Numeric Error Codes)

- `AppError` enum has **~20 variants** grouped by domain prefix (100xx–50001)
- Each variant maps to a **numeric `error_code()`** (e.g., `Validation` → 10002) and an **HTTP `status_code()`**
- Uses `thiserror::Error` for `Display` derive
- Implements `IntoResponse` to serialize into `ApiErrorResponse { success: false, code, request_id, message, details }`
- `From<sqlx::Error>` impl converts DB errors to `AppError::Database`
- Service errors convert via `?` operator with `From` impls — clean and simple

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

## `response.rs` — Response Types

- `ApiResponse<T>` has `success: bool`, `request_id: String`, `data: T`
- `PaginatedResponse<T>` has `success`, `request_id`, `meta: Meta`, `data: PaginatedData<T>`
- `ApiResponse::created(data)` returns 201
- `no_content()` returns 204

## `middleware/auth.rs` — JWT Middleware

- **`Claims`** struct — JWT payload (`sub` user_id, `username`, `role`, `exp`, `iat`)
- **`AuthContext`** extractor — pulled from a validated JWT token (contains `user_id`, `username`, `role`)
- **`auth_middleware`** — Axum middleware layer that validates Bearer tokens from the `Authorization` header
  - Reads JWT secret from request extensions
  - Missing `JwtSecret` returns 500 `Authentication is not configured` instead of silently using an empty secret
  - Decodes the token with HS256 via `jsonwebtoken`
  - On success: injects `AuthContext` into request extensions
  - On failure: returns 401 with `ApiErrorResponse` (includes `success: false`, `request_id`, numeric error code 11001/11002)
- Middleware wraps individual sub-routers (not the whole app)
- Token generation lives in the **handler/service layer**, not in middleware
