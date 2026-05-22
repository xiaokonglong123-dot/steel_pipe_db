# `services/` — Business Logic Layer (12 files)

This is where business rules, cross-entity orchestration, and transaction management live. Services are called by handlers and call repositories.

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
5. **Transactions**: Use `sqlx::Transaction::begin(&pool).await`, pass `&mut *tx` to repos
6. **Cross-entity ops**: Call multiple repositories directly (pool shared via parameter)
7. **No HTTP logic**: Services never know about StatusCodes, response formatting, or headers

## Key patterns in `inventory_service.rs` (largest, most complex)
- Stock-in / stock-out with quantity validation
- Inventory movement tracking
- ATP (Available-to-Promise) calculations
- Query building with dynamic filters
- Batch operations
- Report calculations
- Inventory checks for sales order fulfillment

## Adding a New Service
1. Create `new_service.rs`
2. Add `pub mod new_service;` to `mod.rs`
3. Define `pub struct NewService;` with static methods taking `pool: &SqlitePool`
4. Route registration in `router.rs`
