# `handlers/` — HTTP Layer (16 files, ~100 handlers)

## Pattern

Every handler follows the same pattern: **extract → call service → respond**

```rust
pub async fn list_items_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<ItemFilterParams>,
) -> Result<Json<PaginatedResponse<Item>>, AppError> {
    let (items, total) = ItemService::list_items(&pool, &filter, &pagination).await?;
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
| ------ | -------- | ------------- |
| `auth_handler.rs` | Auth | login, logout, refresh, profile |
| `item_handler.rs` | Items | 商品/SKU master data CRUD, list, filter |
| `inbound_handler.rs` | Inbound | inbound (入库) record CRUD, approval, batch |
| `outbound_handler.rs` | Outbound | outbound (出库) record CRUD, approval |
| `location_handler.rs` | Locations | warehouse location CRUD, assign, transfer |
| `check_handler.rs` | Count Sessions | inventory count session (盘点) CRUD, submit, complete |
| `inventory_handler.rs` | Inventory | stock query, logs, statistics |
| `purchase_handler.rs` | Purchase Orders | 采购订单 CRUD, status transitions, approval |
| `sales_handler.rs` | Sales Orders | 销售订单 CRUD, status transitions, ATP check |
| `contract_handler.rs` | Contracts | CRUD, milestones |
| `customer_handler.rs` | Customers | CRUD, list |
| `supplier_handler.rs` | Suppliers | CRUD, list |
| `report_handler.rs` | Reports | dashboard, daily/monthly/statistical reports |
| `data_io_handler.rs` | Data IO | Excel/CSV import and export (generic entities: items, orders) |
| `atp_handler.rs` | ATP | ATP check (stock availability) before sales order approval |
| `health_handler.rs` | Health | liveness/readiness probes |

Feature modules carry their own handlers inside the module: `workflow/handlers.rs` (definitions/instances/tasks), `hr/handlers.rs`, `finance/handlers.rs`, `procurement/handlers.rs`, `sales_crm/handlers.rs`, `inventory_atp/handlers.rs`, `manufacturing/handlers.rs` (incl. inspections), `project/handlers.rs`, `assets/handlers.rs`, `notification/handlers.rs`, `portal/handlers.rs`, `bi/handlers.rs`, and `auth/handlers.rs` (roles/permissions/departments/tenants). They follow the same extract → service → respond pattern.

## Common Extractor Patterns

- `Extension(pool): Extension<SqlitePool>` — DB pool (every handler needs this)
- `Extension(jwt_secret): Extension<JwtSecret>` — JWT secret newtype (auth handlers only)
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
