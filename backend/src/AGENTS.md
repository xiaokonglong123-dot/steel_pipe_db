# `backend/src/` — Module Wiring & Router

This directory wires all backend source modules together. It is NOT the place to add new feature files — those go in `handlers/`, `services/`, `repositories/`, etc.

## Module Registration

**To add a new backend module:**
1. Create file in appropriate subdirectory (e.g., `handlers/new_thing_handler.rs`)
2. Add `pub mod new_thing_handler;` to the subdirectory's `mod.rs`
3. Add route in `router.rs`
4. Add handler to the handler `mod.rs`

## `main.rs` — Entry Point
- `#![allow(dead_code)]` at crate root (suppresses unused code warnings)
- `#[tokio::main]` async entry
- Initializes: logging (tracing), DB pool (SqlitePool), Config from env
- Calls `router::create_app(pool, jwt_secret)` to build the router
- Binds `0.0.0.0:3000`, starts serving
- **No `AppState` struct exists** — DI is via `Extension<>` layers

## Shared State Pattern — `Extension<>` not `State<>`
**This project uses Axum `Extension<>` layers for dependency injection, NOT `State<Arc<AppState>>`.**
```rust
// router.rs (line 423-426)
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

## `error.rs` — Error Handling
```rust
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    BadRequest(String),
    Internal(String),
}
impl IntoResponse for AppError { ... }
```
- All service layer errors convert into HTTP 4xx/5xx via `IntoResponse`
- Use `map_err(AppError::from)` or `?` operator with `From` impls

## `auth.rs` — JWT Claims
- `Claims` struct (JWT payload with `sub`, `role`, `scope`, `exp`)
- Token generation and verification functions
- `AuthUser` struct extracted by middleware
