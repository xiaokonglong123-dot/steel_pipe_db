# `repositories/` + `models/` — Data Access Layer

## repositories/ (20 files, SQL layer)

### Pattern

```rust
pub struct SeamlessPipeRepo;  // No fields, no constructor

impl SeamlessPipeRepo {
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateSeamlessPipeRequest,
    ) -> Result<SeamlessPipe, sqlx::Error> {
        sqlx::query_as::<_, SeamlessPipe>(
            "INSERT INTO seamless_pipes (...) VALUES (...) RETURNING *"
        )
        .bind(&dto.pipe_number)
        // ...
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_pipe_number(
        pool: &SqlitePool,
        pipe_number: &str,
    ) -> Result<Option<SeamlessPipe>, sqlx::Error> {
        sqlx::query_as::<_, SeamlessPipe>(
            "SELECT * FROM seamless_pipes WHERE pipe_number = ? AND deleted_at IS NULL"
        )
        .bind(pipe_number)
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

### File List (20 files)

| File | Entity | Description |
|------|--------|-------------|
| `inventory_repo.rs` | Inventory (ATP) | ATP查询, 库存统计, 管道位置 |
| `location_repo.rs` | Locations | 仓库位置 CRUD |
| `inbound_repo.rs` | Inbound | 入库记录 CRUD |
| `outbound_repo.rs` | Outbound | 出库记录 CRUD |
| `inventory_log_repo.rs` | Inventory Logs | 管道移动审计日志 |
| `check_repo.rs` | Inventory Checks | 盘点记录和盘点项 |
| `report_repo.rs` | Reports | 聚合查询, 日期范围 |
| `pipe_repo.rs` | Pipe specs | CRUD + 过滤 + 分页 |
| `purchase_order_repo.rs` | Purchase Orders | PO CRUD + 状态 |
| `sales_order_repo.rs` | Sales Orders | SO CRUD + ATP 查询 |
| `contract_repo.rs` | Contracts | CRUD + 状态查询 |
| `quality_repo.rs` | Quality | 证书 + 测试结果 |
| `customer_repo.rs` | Customers | CRUD |
| `supplier_repo.rs` | Suppliers | CRUD |
| `label_repo.rs` | Labels | 标签数据查询 |
| `data_io_repo.rs` | Data IO | 批量读写 |
| `user_repo.rs` | Users | 认证查询 |
| `operation_log_repo.rs` | Audit logs | 插入 + 查询 |
| `refresh_token_repo.rs` | Refresh Tokens | 令牌管理 |

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

## models/ (10 files — DB row structs)

> **Note**: `inventory.rs` got a new field recently, but we're still at 10 model files.

### Pattern

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SeamlessPipe {
    pub id: i64,
    pub pipe_number: String,
    pub grade: String,
    pub od: f64,              // NOT rust_decimal::Decimal
    pub wt: f64,              // NOT rust_decimal::Decimal
    pub length: Option<f64>,  // NOT rust_decimal::Decimal
    pub weight_per_unit: Option<f64>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,   // ISO 8601 text, NOT chrono::NaiveDateTime
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

`user.rs`, `seamless_pipe.rs`, `screen_pipe.rs`, `inventory.rs`, `purchase_order.rs`, `sales_order.rs`, `quality.rs`, `contract.rs`, `customer.rs`, `supplier.rs`
