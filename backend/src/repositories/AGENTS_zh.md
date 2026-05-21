# `repositories/` + `models/` — 数据访问层

## repositories/（14 个文件，SQL 层）

### 模式
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

### 文件列表（5 个最大的——复杂度热点）
| 文件 | 实体 | 大小 |
|------|--------|------|
| `inventory_repo.rs` | 库存 | **755 行**——动态查询、库存移动、过滤器 |
| `report_repo.rs` | 报表 | **586 行**——大量查询变体、聚合、日期范围 |
| `pipe_repo.rs` | 钢管规格 | **584 行**——CRUD + 过滤 + 分页查询 |
| `contract_repo.rs` | 合同 | **539 行**——CRUD + 状态查询 |
| `purchase_repo.rs` | 采购订单 | ~400 行 |
| `production_repo.rs` | 生产 | ~300 行 |
| `customer_repo.rs` | 客户 | ~200 行 |
| `supplier_repo.rs` | 供应商 | ~200 行 |
| `category_repo.rs` | 分类 | ~150 行 |
| `warehouse_repo.rs` | 仓库 | ~150 行 |
| `dashboard_repo.rs` | 仪表盘统计 | ~150 行 |
| `dictionary_repo.rs` | 字典 | ~150 行 |
| `auth_repo.rs` | 认证（用户） | ~100 行 |
| `mod.rs` | 模块导出 | |

### Repository 约定
1. **构造函数**：`pub fn new(db: Pool<Sqlite>) -> Self`
2. **方法**：按实体命名为 `list`、`find_by_id`、`create`、`update`、`delete`
3. **软删除**：所有实体查询 `WHERE deleted_at IS NULL`
4. **分页**：`LIMIT ? OFFSET ?` 配合 `page`/`page_size` 参数
5. **返回**：列表返回 `Result<Vec<Model>>`，单个查询返回 `Result<Option<Model>>`
6. **错误类型**：`sqlx::Error`（调用方转换为 `AppError`）
7. **动态查询**：对条件过滤器（如日期范围、状态过滤）使用字符串构建方式
8. **无业务逻辑**：纯 SQL——不包含验证，不做行→结构体映射之外的转换

### 查询模式
- **基本 CRUD**：`sqlx::query_as!` 宏配合编译时检查
- **动态过滤器**：使用 `WHERE 1=1` 模式构建查询字符串，追加条件
- **分页**：同一方法中始终通过 `SELECT COUNT(*)` 返回总数
- **事务**：Repository 方法可通过 `&mut *tx` 接受 `&mut Transaction<'_, Sqlite>`

## models/（11 个文件——数据库行结构体）

### 模式
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

### 约定
- 每个数据库表 1 个结构体，每个实体 1 个文件
- `#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]`
- 字段与数据库列完全匹配（可空字段使用 `Option<T>`）
- `sqlx::FromRow` 支持从查询结果自动映射
- 软删除通过 `deleted_at: Option<NaiveDateTime>` 追踪
- 时间戳使用 `chrono::NaiveDateTime`
- 十进制类型使用 `rust_decimal::Decimal`

### models/ 与 dto/ 的区别
- **models/**：数据库行结构体——每表一个，镜像数据库模式
- **dto/**：API 请求/响应结构体——包含验证注解，可能聚合多个模型

### 文件列表
`pipe.rs`、`inventory.rs`、`purchase.rs`、`production.rs`、`contract.rs`、`customer.rs`、`supplier.rs`、`category.rs`、`warehouse.rs`、`dictionary.rs`、`user.rs`
