# `services/` — Business Logic Layer

This is where the real work happens — business rules, cross-entity orchestration, transaction management. Services get called by handlers and in turn call repositories.

## Pattern

```rust
pub struct ItemService;  // No fields, no constructor, no DI

impl ItemService {
    pub async fn list_items(
        pool: &SqlitePool,
        params: &ItemFilterParams,
        pagination: &PaginationParams,
    ) -> Result<(Vec<Item>, i64), AppError> {
        // 1. Validate business rules
        // 2. Call repository
        // 3. Transform/aggregate results
        // 4. Return
    }
}
```

## Service File List

| File | Entity | Description |
| ------ | -------- | ------------- |
| `auth_service.rs` | Auth | login, token refresh, password verify |
| `item_service.rs` | Items | 商品/SKU master data CRUD, code uniqueness |
| `inbound_service.rs` | Inbound | inbound (入库) record creation, approval, batch execution |
| `outbound_service.rs` | Outbound | outbound (出库) record creation, approval, stock deduction |
| `check_service.rs` | Inventory checks | count session (盘点) creation, item submission, completion |
| `inventory_query_service.rs` | Inventory (read) | read-only inventory queries (list, statistics) |
| `location_service.rs` | Locations | warehouse location CRUD, assign, transfer |
| `purchase_service.rs` | Purchase Orders | 采购订单 lifecycle, approval workflow, rejection reason |
| `sales_service.rs` | Sales Orders | 销售订单 lifecycle, approval workflow, ATP validation |
| `contract_service.rs` | Contracts | contract CRUD, milestone tracking |
| `customer_service.rs` | Customers | customer CRUD, code uniqueness |
| `supplier_service.rs` | Suppliers | supplier CRUD, qualification |
| `report_service.rs` | Reports | dashboard aggregation, statistical reports |
| `data_io_service.rs` | Data IO | Excel/CSV import parsing, export formatting |

Feature modules ship their own services inside the module: `auth/services.rs` (roles/permissions/departments/tenants), `workflow/services.rs` (approval engine), `hr/services.rs` (employees/attendance/salaries/labor contracts), `finance/services.rs` (accounts/journal/invoices/payments/trial balance), `procurement/services.rs` (requisitions/receipts/quotes/scorecard), `sales_crm/services.rs` (shipments/quotes/customer credit), `inventory_atp/services.rs` (reservations/transfers/count sessions), `manufacturing/services.rs` (BOMs/work orders/inspections/NCRs), `project/services.rs`, `assets/services.rs` (depreciation/disposal), `notification/services.rs`, `portal/services.rs`, `bi/services.rs` (analytics). They follow the same unit-struct pattern.

## Service Conventions

1. **Pattern**: Unit struct with static methods — `pub struct XxxService;` then `impl XxxService { pub async fn ... }`
2. **First parameter**: Always `pool: &SqlitePool`
3. **Return type**: Always `Result<T, AppError>`
4. **Naming**: `list_*`, `get_*`, `create_*`, `update_*`, `delete_*`
5. **Transactions**: Use `sqlx::Transaction::begin(&pool).await`, then pass `&mut *tx` to repos
6. **Cross-entity ops**: Call multiple repositories directly — the pool gets passed around as a parameter
7. **No HTTP logic**: Services don't know about StatusCodes, response formatting, or headers. That's the handler's job.

## Inventory services — split by responsibility

The inventory service layer is split into focused modules:

- `inbound_service.rs` — stock-in (入库) record creation, approval, batch execution
- `outbound_service.rs` — stock-out (出库) record creation, approval, stock deduction
- `check_service.rs` — inventory count session (盘点) creation, item submission, completion
- `inventory_query_service.rs` — read-only queries (list, statistics, filters)
- `location_service.rs` — warehouse location CRUD, assign, transfer

ATP (Available-to-Promise) calculation lives in `sales_service.rs` (ATP check before sales order approval) and `atp_handler.rs`. Sales-order fulfillment checks read ATP before approval. Reservations and transfers are managed by `inventory_atp/services.rs`.

## Adding a New Service

1. Create `new_service.rs`
2. Add `pub mod new_service;` to `mod.rs`
3. Define `pub struct NewService;` with static methods taking `pool: &SqlitePool`
4. Register routes in `router.rs`
