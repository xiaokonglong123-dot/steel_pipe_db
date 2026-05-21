# `handlers/` — HTTP Layer (12 files, ~50 handlers)

## Pattern
Every handler follows: **extract → call service → respond**

```rust
pub async fn list_seamless_pipes_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<PipeFilterParams>,
) -> Result<Json<PaginatedResponse<SeamlessPipe>>, AppError> {
    let (items, total) = PipeService::list_seamless_pipes(&pool, &filter, &pagination).await?;
    Ok(PaginatedResponse::ok(items, total, page, page_size))
}
```

Key points:
- Return type: `Result<Json<...>, AppError>` — NOT `impl IntoResponse`
- Use `?` operator for error propagation (AppError auto-converts via `IntoResponse`)
- No manual `.into_response()` calls
- Handlers use `ApiResponse::ok()` or `PaginatedResponse::ok()` static constructors

## Response Types (from `crate::response`)
- `ApiResponse<T>` — Standard success: `{ "success": true, "data": T }`
- `PaginatedResponse<T>` — Paginated: `{ "success": true, "data": { "items": [], "total": N, "page": P, "page_size": S, "total_pages": N } }`
- `AppError` — Error (via `IntoResponse`): `{ "code": 11001, "message": "...", "details": null }`

## Handler File List

| File | Entity | Description |
|------|--------|-------------|
| `auth_handler.rs` | Auth | login, logout, refresh, profile |
| `pipe_handler.rs` | Pipes | seamless + screen pipe CRUD, list, filter |
| `inventory_handler.rs` | Inventory | inbound, outbound, stock query, locations, check |
| `purchase_handler.rs` | Purchase Orders | CRUD, status transitions, approval |
| `sales_handler.rs` | Sales Orders | CRUD, status transitions, ATP check |
| `quality_handler.rs` | Quality | certs CRUD, mechanical tests, NDT |
| `contract_handler.rs` | Contracts | CRUD, milestones |
| `customer_handler.rs` | Customers | CRUD, list |
| `supplier_handler.rs` | Suppliers | CRUD, list |
| `report_handler.rs` | Reports | dashboard, daily/monthly/statistical reports |
| `label_handler.rs` | Labels | barcode/spec label generation |
| `data_io_handler.rs` | Data IO | Excel/CSV import and export |

## Common Extractor Patterns
- `Extension(pool): Extension<SqlitePool>` — DB pool (required on every handler)
- `Extension(jwt_secret): Extension<String>` — JWT secret (auth handlers)
- `Query(params): Query<T>` — GET query params (T: DeserializeOwned)
- `Json(body): Json<T>` — POST/PUT body (T: DeserializeOwned)
- `Path(id): Path<i64>` — URL path param
- `AuthUser(user): AuthUser` — JWT-authenticated user extractor

Validation is done inline via `validator::Validate::validate()`:
```rust
req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
```

## Conventions
- One handler file per entity
- Handler functions are `pub async fn` returning `Result<Json<...>, AppError>`
- Always use `ApiResponse::ok()` / `PaginatedResponse::ok()` static constructors
- Error propagation via `?` operator with AppError auto-conversion
- Most handlers are thin (5-15 lines) — business logic lives in services
