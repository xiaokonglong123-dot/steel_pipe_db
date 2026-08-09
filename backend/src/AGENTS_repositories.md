# `repositories/` + `models/` — Data Access Layer

## repositories/ (SQL layer)

### Pattern

```rust
pub struct ItemRepo;  // No fields, no constructor

impl ItemRepo {
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateItemRequest,
    ) -> Result<Item, sqlx::Error> {
        sqlx::query_as::<_, Item>(
            "INSERT INTO items (sku, name, category, unit, spec) VALUES (?, ?, ?, ?, ?) RETURNING *"
        )
        .bind(&dto.sku)
        // ...
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_sku(
        pool: &SqlitePool,
        sku: &str,
    ) -> Result<Option<Item>, sqlx::Error> {
        sqlx::query_as::<_, Item>(
            "SELECT * FROM items WHERE sku = ? AND deleted_at IS NULL"
        )
        .bind(sku)
        .fetch_optional(pool)
        .await
    }
}
```

Key facts:

- Repos are **unit structs** (no fields) with **static methods**
- First param is always `pool: &SqlitePool` (nothing stored in `self`)
- **No constructor**, **no DI pattern** — you won't find `pub fn new(db)` anywhere
- Returns `Result<T, sqlx::Error>` — the caller converts to AppError

### File List

| File | Entity | Description |
| ------ | -------- | ------------- |
| `inventory_repo.rs` | Inventory (ATP) | ATP queries, stock counting, item locations |
| `location_repo.rs` | Locations | warehouse location CRUD |
| `inbound_repo.rs` | Inbound | inbound (入库) record CRUD |
| `outbound_repo.rs` | Outbound | outbound (出库) record CRUD |
| `inventory_log_repo.rs` | Inventory Logs | item movement audit trail |
| `check_repo.rs` | Inventory Checks | count session (盘点) records and items |
| `report_repo.rs` | Reports | aggregation queries, date ranges |
| `item_repo.rs` | Items | 商品/SKU master data CRUD + filtered + paginated |
| `purchase_order_repo.rs` | Purchase Orders | 采购订单 CRUD + status |
| `sales_order_repo.rs` | Sales Orders | 销售订单 CRUD + ATP queries |
| `contract_repo.rs` | Contracts | CRUD + status queries |
| `customer_repo.rs` | Customers | CRUD |
| `supplier_repo.rs` | Suppliers | CRUD |
| `data_io_repo.rs` | Data IO | bulk read/write |
| `user_repo.rs` | Users | auth queries |
| `operation_log_repo.rs` | Audit logs | insert + query |
| `refresh_token_repo.rs` | Refresh Tokens | token management |

Feature modules ship their own repos inside the module (`workflow/repos.rs`, `hr/repos.rs`, `finance/repos.rs`, `procurement/repos.rs`, `sales_crm/repos.rs`, `inventory_atp/repos.rs`, `manufacturing/repos.rs`, `project/repos.rs`, `assets/repos.rs`, `notification/repos.rs`, `portal/repos.rs`, `auth/repos.rs`). `bi/` has no repos — read-only analytics reuse the shared repositories.

### Repository Conventions

1. **Pattern**: Unit struct with static methods — `pub struct XxxRepo;` then `impl XxxRepo { pub async fn ... }`
2. **First parameter**: Always `pool: &SqlitePool`
3. **Methods**: Named by operation — `find_by_*`, `create`, `update`, `delete_soft`
4. **Soft delete**: Every query filters `WHERE deleted_at IS NULL`
5. **Pagination**: `LIMIT ? OFFSET ?` with `page`/`page_size` params
6. **Returns**: `Result<Vec<Model>>` for list, `Result<Option<Model>>` for find_by, `Result<Model>` for create
7. **Error type**: `sqlx::Error` (caller converts to `AppError`)
8. **Dynamic queries**: String building for conditional filters (date ranges, statuses, etc.)
9. **No business logic**: Pure SQL — no validation, no transformations beyond row-to-struct mapping

### Query Patterns

- **Basic CRUD**: `sqlx::query_as::<_, Model>(...)` with `.bind()` for params
- **Dynamic filters**: Build the query string with a `WHERE 1=1` pattern, append conditions as needed
- **Pagination**: Always return total count via `SELECT COUNT(*)` in the same method
- **Transactions**: Repo methods can accept `&mut Transaction<'_, Sqlite>` when they're part of a transaction

## models/ (DB row structs)

### Pattern

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: i64,
    pub sku: String,             // 商品唯一业务编码
    pub name: String,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub spec: Option<String>,    // 可选规格，无行业强制字段
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,      // ISO 8601 text, NOT chrono::NaiveDateTime
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
```

### Conventions

- One struct per DB table, one file per entity
- `#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]`
- Fields match DB columns exactly (nullable columns use `Option<T>`)
- `sqlx::FromRow` handles automatic mapping from query results
- Soft delete tracked via `deleted_at: Option<String>`
- Timestamps are **`String`** (ISO 8601 text), NOT `chrono::NaiveDateTime`
- Decimal fields are **`f64`** (NOT `rust_decimal::Decimal`)

### models/ vs dto/ distinction

- **models/**: DB row structs — one per table, mirrors the DB schema
- **dto/**: API request/response structs — have validation annotations, might aggregate multiple models

### File List

`user.rs`, `rbac.rs`, `item.rs`, `inventory.rs`, `purchase_order.rs`, `sales_order.rs`, `contract.rs`, `customer.rs`, `supplier.rs` — plus feature models: `workflow.rs`, `hr.rs`, `finance.rs`, `procurement.rs`, `sales_crm.rs`, `inventory_atp.rs`, `manufacturing.rs`, `project.rs`, `assets.rs`, `notification.rs`, `portal.rs`
