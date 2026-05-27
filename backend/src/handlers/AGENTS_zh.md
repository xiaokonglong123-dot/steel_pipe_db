# `handlers/` — HTTP Layer (13 files, ~55 handlers)

## Pattern

Every handler follows the same damn pattern: **extract → call service → respond**

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
- Use `?` for error propagation (AppError converts itself via `IntoResponse`)
- No manual `.into_response()` calls — keep it clean
- Handlers use `ApiResponse::ok()` or `PaginatedResponse::ok()` static constructors

## Response Types (from `crate::response`)

- `ApiResponse<T>` — Standard success: `{ "success": true, "request_id": "req_...", "data": T }`
- `PaginatedResponse<T>` — Paginated: `{ "success": true, "request_id": "req_...", "meta": { total, page, page_size, total_pages }, "data": { "items": [], ... } }`
- `AppError` — Error (via `IntoResponse`): `{ "success": false, "code": 11001, "request_id": "req_...", "message": "...", "details": null }`
- Need a 201? `ApiResponse::created(data)` has you covered
- Need a 204? `no_content()` — mostly used for deletions

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
| `atp_handler.rs` | ATP | ATP check (stock availability) before sales order approval |
| `data_io_handler.rs` | Data IO | Excel/CSV import and export |

## Common Extractor Patterns

- `Extension(pool): Extension<SqlitePool>` — DB pool (every handler needs this)
- `Extension(jwt_secret): Extension<String>` — JWT secret (auth handlers only)
- `Query(params): Query<T>` — GET query params (T needs DeserializeOwned)
- `Json(body): Json<T>` — POST/PUT body (T needs DeserializeOwned)
- `Path(id): Path<i64>` — URL path parameter
- `AuthUser(user): AuthUser` — JWT-authenticated user extractor

Validation's done inline via `validator::Validate::validate()`:

```rust
req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
```

## Conventions

- One handler file per entity — keeps things organized
- Handler functions are `pub async fn` returning `Result<Json<...>, AppError>`
- Always reach for `ApiResponse::ok()` / `PaginatedResponse::ok()` — don't construct responses manually
- Error propagation via `?` with AppError auto-conversion
- Most handlers are thin (5-15 lines) — business logic lives in services, keep it there
