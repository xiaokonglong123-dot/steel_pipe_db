# `repositories/` + `models/` — Data Access Layer

## repositories/ (14 files, SQL layer)

### Pattern
```rust
pub struct PipeRepository {
    db: Pool<Sqlite>,
}

impl PipeRepository {
    pub fn new(db: Pool<Sqlite>) -> Self { Self { db } }

    pub async fn list(&self, params: &PipeListParams) -> Result<Vec<PipeModel>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PipeModel>(
            "SELECT * FROM pipes WHERE deleted_at IS NULL ORDER BY id DESC"
        )
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<PipeModel>, sqlx::Error> {
        sqlx::query_as::<_, PipeModel>("SELECT * FROM pipes WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.db)
            .await
    }
}
```

### File List (5 largest — complexity hotspots)
| File | Entity | Size |
|------|--------|------|
| `inventory_repo.rs` | Inventory stock | **755 lines** — dynamic queries, stock movements, filters |
| `report_repo.rs` | Reports | **586 lines** — many query variants, aggregation, date ranges |
| `pipe_repo.rs` | Pipe specs | **584 lines** — CRUD + filtered + paginated queries |
| `contract_repo.rs` | Contracts | **539 lines** — CRUD + status queries |
| `purchase_repo.rs` | Purchase orders | ~400 lines |
| `production_repo.rs` | Production | ~300 lines |
| `customer_repo.rs` | Customers | ~200 lines |
| `supplier_repo.rs` | Suppliers | ~200 lines |
| `category_repo.rs` | Categories | ~150 lines |
| `warehouse_repo.rs` | Warehouses | ~150 lines |
| `dashboard_repo.rs` | Dashboard stats | ~150 lines |
| `dictionary_repo.rs` | Dictionaries | ~150 lines |
| `auth_repo.rs` | Auth (users) | ~100 lines |
| `mod.rs` | Module exports | |

### Repository Conventions
1. **Constructor**: `pub fn new(db: Pool<Sqlite>) -> Self`
2. **Methods**: Named `list`, `find_by_id`, `create`, `update`, `delete` per entity
3. **Soft delete**: All entities query `WHERE deleted_at IS NULL`
4. **Pagination**: `LIMIT ? OFFSET ?` with `page`/`page_size` params
5. **Returns**: `Result<Vec<Model>>` for list, `Result<Option<Model>>` for find_by_id
6. **Error type**: `sqlx::Error` (caller converts to `AppError`)
7. **Dynamic queries**: Use string building for conditional filters (e.g., date ranges, status filters)
8. **No business logic**: Pure SQL — no validation, no transformations beyond row→struct mapping

### Query Patterns
- **Basic CRUD**: `sqlx::query_as!` macro with compile-time checking
- **Dynamic filters**: Build query string with `WHERE 1=1` pattern, append conditions
- **Pagination**: Always return total count via `SELECT COUNT(*)` in same method
- **Transactions**: Repository methods can accept `&mut Transaction<'_, Sqlite>` via `&mut *tx`

## models/ (11 files — DB row structs)

### Pattern
```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Pipe {
    pub id: i64,
    pub name: String,
    pub spec: String,
    pub unit_price: Option<rust_decimal::Decimal>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}
```

### Conventions
- 1 struct per DB table, 1 file per entity
- `#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]`
- Fields match DB columns exactly (including `Option<T>` for nullable)
- `sqlx::FromRow` enables automatic mapping from query results
- Soft delete tracked via `deleted_at: Option<NaiveDateTime>`
- Timestamps use `chrono::NaiveDateTime`
- Decimal types use `rust_decimal::Decimal`

### models/ vs dto/ distinction
- **models/**: DB row structs — one per table, mirrors DB schema
- **dto/**: API request/response structs — validation annotations, may aggregate multiple models

### File List
`pipe.rs`, `inventory.rs`, `purchase.rs`, `production.rs`, `contract.rs`, `customer.rs`, `supplier.rs`, `category.rs`, `warehouse.rs`, `dictionary.rs`, `user.rs`
