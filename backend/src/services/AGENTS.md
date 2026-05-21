# `services/` — Business Logic Layer (13 files)

This is where business rules, cross-entity orchestration, and transaction management live. Services are called by handlers and call repositories.

## Pattern
```rust
pub struct PipeService {
    repo: Arc<PipeRepository>,
    inventory_repo: Arc<InventoryRepository>,
    db: Pool<Sqlite>,
}

impl PipeService {
    pub fn new(repo: Arc<PipeRepository>, inventory_repo: Arc<InventoryRepository>, db: Pool<Sqlite>) -> Self { ... }
    
    pub async fn list_pipes(&self, params: &PipeListParams) -> Result<PagedResult<PipeDto>, AppError> {
        // 1. Validate business rules
        // 2. Call repository
        // 3. Transform/aggregate results
        // 4. Return
    }
}
```

## Service File List
| File | Description | Size |
|------|-------------|------|
| `auth_service.rs` | Login, register, token management | ~200 lines |
| `pipe_service.rs` | Pipe specs, product config | ~300 lines |
| `inventory_service.rs` | Stock management (764 lines — largest) | **764** |
| `purchase_service.rs` | Purchase orders, approvals | ~350 lines |
| `production_service.rs` | Production orders | ~250 lines |
| `report_service.rs` | Report generation | ~400 lines |
| `contract_service.rs` | Contracts | ~200 lines |
| `customer_service.rs` | Customer management | ~200 lines |
| `supplier_service.rs` | Supplier management | ~200 lines |
| `category_service.rs` | Category management | ~150 lines |
| `warehouse_service.rs` | Warehouse management | ~200 lines |
| `dashboard_service.rs` | Dashboard aggregation | ~200 lines |

## Service Conventions
1. **Constructor**: `pub fn new(...repos..., db: Pool<Sqlite>) -> Self` — dependency injection via constructor
2. **Return type**: Always `Result<T, AppError>` where T is entity-specific
3. **Naming**: Methods mirror handler actions: `list_*`, `get_*`, `create_*`, `update_*`, `delete_*`
4. **Transactions**: Wrap multi-repo operations in `sqlx::Transaction`:
   ```rust
   let mut tx = self.db.begin().await.map_err(AppError::from)?;
   // ... repository calls with &mut *tx ...
   tx.commit().await.map_err(AppError::from)?;
   ```
5. **Cross-entity ops**: Single service can call multiple repositories (e.g., inventory_service calls inventory_repo AND pipe_repo)
6. **No HTTP logic**: Services never know about HTTP StatusCodes, response formatting, or headers

## Key patterns in `inventory_service.rs` (largest, most complex)
- Stock-in / stock-out with quantity validation
- Inventory movement tracking
- Query building with dynamic filters
- Batch operations
- Report calculations

## Adding a New Service
1. Create `new_service.rs`
2. Add `pub mod new_service;` to `mod.rs`
3. Define `pub struct NewService { ... }` with constructor
4. Register in `mod.rs` or pass to AppState in `main.rs`
