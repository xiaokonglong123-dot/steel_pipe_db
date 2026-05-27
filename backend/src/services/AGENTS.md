# `services/` — Business Logic Layer (12 files)

This is where the real work happens — business rules, cross-entity orchestration, transaction management. Services get called by handlers and in turn call repositories.

## Pattern

```rust
pub struct PipeService;  // No fields, no constructor, no DI

impl PipeService {
    pub async fn list_seamless_pipes(
        pool: &SqlitePool,
        params: &PipeFilterParams,
        pagination: &PaginationParams,
    ) -> Result<(Vec<SeamlessPipe>, i64), AppError> {
        // 1. Validate business rules
        // 2. Call repository
        // 3. Transform/aggregate results
        // 4. Return
    }
}
```

## Service File List

| File | Entity | Description |
|------|--------|-------------|
| `auth_service.rs` | Auth | login, token refresh, password verify |
| `pipe_service.rs` | Pipes | pipe CRUD, steel grade/heat treatment validation |
| `inventory_service.rs` | Inventory | inbound, outbound, ATP calculations, location management, inventory checks |
| `purchase_sales_service.rs` | Purchase + Sales | PO/SO lifecycle, approval workflow, rejection reason, ATP validation |
| `quality_service.rs` | Quality | cert creation, mechanical/NDT test entry |
| `contract_service.rs` | Contracts | contract CRUD, milestone tracking |
| `customer_service.rs` | Customers | customer CRUD, code uniqueness |
| `supplier_service.rs` | Suppliers | supplier CRUD, qualification |
| `label_service.rs` | Labels | label content generation |
| `report_service.rs` | Reports | dashboard aggregation, statistical reports |
| `data_io_service.rs` | Data IO | Excel/CSV import parsing, export formatting |
| `trace_service.rs` | Trace | inventory movement audit trail |

## Service Conventions

1. **Pattern**: Unit struct with static methods — `pub struct XxxService;` then `impl XxxService { pub async fn ... }`
2. **First parameter**: Always `pool: &SqlitePool`
3. **Return type**: Always `Result<T, AppError>`
4. **Naming**: `list_*`, `get_*`, `create_*`, `update_*`, `delete_*`
5. **Transactions**: Use `sqlx::Transaction::begin(&pool).await`, then pass `&mut *tx` to repos
6. **Cross-entity ops**: Call multiple repositories directly — the pool gets passed around as a parameter
7. **No HTTP logic**: Services don't know about StatusCodes, response formatting, or headers. That's the handler's job.

## `inventory_service.rs` — the big one

This is the largest and most complex service. Here's what it handles:

- Stock-in / stock-out with quantity validation
- Inventory movement tracking
- ATP (Available-to-Promise) calculations
- Dynamic query building with filters
- Batch operations
- Report calculations
- Sales order fulfillment checks

## Adding a New Service

1. Create `new_service.rs`
2. Add `pub mod new_service;` to `mod.rs`
3. Define `pub struct NewService;` with static methods taking `pool: &SqlitePool`
4. Register routes in `router.rs`
