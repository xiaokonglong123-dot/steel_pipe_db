# `handlers/` — HTTP Layer (13 files, 110+ handlers)

## Pattern
Every handler follows: **extract → validate → call service → respond**

```rust
pub async fn list_pipes(
    Extension(pool): Extension<SqlitePool>,
    Query(params): Query<PipeListParams>,
) -> impl IntoResponse {
    // 1. Validate params (if needed)
    // 2. Call service
    match pipe_service::list_pipes(&pool, &params).await {
        Ok(result) => Json(ApiResponse::success(result)).into_response(),
        Err(e) => e.into_response(),
    }
}
```

## Response Types (from `dto/api_response.rs`)
- `ApiResponse<T>` — Standard success: `{ "code": 200, "message": "ok", "data": T }`
- `PagedResponse<T>` — Paginated: `{ "code": 200, "data": { "items": [...], "total": N, "page": P, "page_size": S } }`
- `ErrorResponse` — Error: `{ "code": N, "message": "..." }`

## Handler File List
| File | Entity | Endpoints |
|------|--------|-----------|
| `auth_handler.rs` | Auth | login, register, profile, refresh |
| `pipe_handler.rs` | Pipes (specifications) | CRUD + list |
| `inventory_handler.rs` | Inventory stock | CRUD + list |
| `purchase_handler.rs` | Purchase orders | CRUD + status transitions |
| `production_handler.rs` | Production | CRUD + status |
| `report_handler.rs` | Reports | Various report endpoints |
| `contract_handler.rs` | Contracts | CRUD |
| `customer_handler.rs` | Customers | CRUD |
| `supplier_handler.rs` | Suppliers | CRUD |
| `category_handler.rs` | Entity categories | CRUD |
| `warehouse_handler.rs` | Warehouses | CRUD |
| `dictionary_handler.rs` | Dictionaries/configs | CRUD |
| `dashboard_handler.rs` | Dashboard | Summary/stats |

## Common Extractor Patterns
- `Extension(pool): Extension<SqlitePool>` — DB pool (required on every handler)
- `Extension(jwt_secret): Extension<String>` — JWT secret (auth handlers)
- `Query(params): Query<T>` — GET query params (T: DeserializeOwned)
- `Json(body): Json<T>` — POST/PUT body (T: DeserializeOwned)
- `Path(id): Path<i64>` — URL path param
- `AuthUser(user): AuthUser` — JWT-authenticated user extractor
- `ValidatedRequest<T>` — Validated JSON body (custom extractor, uses `validator` crate)

## Conventions
- One handler file per entity
- Handler functions are `pub async fn` returning `impl IntoResponse`
- Always use `Json(ApiResponse::success(...))` for 200 responses
- Use `StatusCode::CREATED` for POST creates: `(StatusCode::CREATED, Json(ApiResponse::success(data)))`
- Error propagation via `?` operator with AppError conversion
- Most handlers are thin (5-15 lines) — business logic lives in services
