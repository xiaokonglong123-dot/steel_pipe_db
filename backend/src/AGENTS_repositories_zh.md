# `repositories/` + `models/` — 数据访问层

## repositories/（SQL 层）

### 模式

```rust
pub struct ItemRepo;  // 无字段，无构造器

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

要点：

- Repo 是 **unit struct**（无字段）+ **静态方法**
- 第一个参数永远是 `pool: &SqlitePool`（`self` 中不存任何东西）
- **无构造器、无 DI 模式** — 任何地方都找不到 `pub fn new(db)`
- 返回 `Result<T, sqlx::Error>` — 由调用方转换为 AppError

### 文件清单

| File | Entity | Description |
| ------ | -------- | ------------- |
| `inventory_repo.rs` | Inventory (ATP) | ATP 查询、库存统计、商品库位 |
| `location_repo.rs` | Locations | 库位 CRUD |
| `inbound_repo.rs` | Inbound | 入库记录 CRUD |
| `outbound_repo.rs` | Outbound | 出库记录 CRUD |
| `inventory_log_repo.rs` | Inventory Logs | 商品变动审计日志 |
| `check_repo.rs` | Inventory Checks | 盘点记录和盘点项 |
| `report_repo.rs` | Reports | 聚合查询、日期范围 |
| `item_repo.rs` | Items | 商品/SKU 主数据 CRUD + 过滤 + 分页 |
| `purchase_order_repo.rs` | Purchase Orders | 采购订单 CRUD + 状态 |
| `sales_order_repo.rs` | Sales Orders | 销售订单 CRUD + ATP 查询 |
| `contract_repo.rs` | Contracts | CRUD + 状态查询 |
| `customer_repo.rs` | Customers | CRUD |
| `supplier_repo.rs` | Suppliers | CRUD |
| `data_io_repo.rs` | Data IO | 批量读写 |
| `user_repo.rs` | Users | 认证查询 |
| `operation_log_repo.rs` | Audit logs | 插入 + 查询 |
| `refresh_token_repo.rs` | Refresh Tokens | 令牌管理 |

功能模块在模块内部自带仓储（`workflow/repos.rs`、`hr/repos.rs`、`finance/repos.rs`、`procurement/repos.rs`、`sales_crm/repos.rs`、`inventory_atp/repos.rs`、`manufacturing/repos.rs`、`project/repos.rs`、`assets/repos.rs`、`notification/repos.rs`、`portal/repos.rs`、`auth/repos.rs`）。`bi/` 无 `repos.rs` — 只读分析复用共享仓储。

### Repository 约定

1. **模式**：unit struct + 静态方法 — `pub struct XxxRepo;` 然后 `impl XxxRepo { pub async fn ... }`
2. **第一个参数**：永远是 `pool: &SqlitePool`
3. **方法命名**：按操作命名 — `find_by_*`、`create`、`update`、`delete_soft`
4. **软删除**：每个查询都过滤 `WHERE deleted_at IS NULL`
5. **分页**：`LIMIT ? OFFSET ?` + `page`/`page_size` 参数
6. **返回**：列表用 `Result<Vec<Model>>`，find_by 用 `Result<Option<Model>>`，create 用 `Result<Model>`
7. **错误类型**：`sqlx::Error`（调用方转换为 `AppError`）
8. **动态查询**：字符串拼接条件过滤（日期范围、状态等）
9. **无业务逻辑**：纯 SQL — 除行到结构体的映射外无校验、无转换

### 查询模式

- **基础 CRUD**：`sqlx::query_as::<_, Model>(...)` + `.bind()` 参数
- **动态过滤**：用 `WHERE 1=1` 模式拼接查询字符串，按需追加条件
- **分页**：同一方法内用 `SELECT COUNT(*)` 返回总数
- **事务**：作为事务一部分时，Repo 方法可接受 `&mut Transaction<'_, Sqlite>`

## models/（数据库行结构体）

### 模式

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
    pub created_at: String,      // ISO 8601 文本，NOT chrono::NaiveDateTime
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
```

### 约定

- 每张表一个结构体，每实体一个文件
- `#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]`
- 字段与数据库列完全一致（可空列用 `Option<T>`）
- `sqlx::FromRow` 自动处理查询结果的映射
- 软删除通过 `deleted_at: Option<String>` 跟踪
- 时间戳是 **`String`**（ISO 8601 文本），不是 `chrono::NaiveDateTime`
- 小数字段是 **`f64`**（不是 `rust_decimal::Decimal`）

### models/ 与 dto/ 的区别

- **models/**：数据库行结构体 — 每表一个，镜像数据库 schema
- **dto/**：API 请求/响应结构体 — 带校验注解，可能聚合多个 model

### 文件清单

`user.rs`、`rbac.rs`、`item.rs`、`inventory.rs`、`purchase_order.rs`、`sales_order.rs`、`contract.rs`、`customer.rs`、`supplier.rs` — 以及功能模型：`workflow.rs`、`hr.rs`、`finance.rs`、`procurement.rs`、`sales_crm.rs`、`inventory_atp.rs`、`manufacturing.rs`、`project.rs`、`assets.rs`、`notification.rs`、`portal.rs`
