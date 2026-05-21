# `repositories/` + `models/` — 数据访问层

## repositories/（13 个文件，SQL 层）

### 模式
```rust
pub struct SeamlessPipeRepo;  // 无字段，无构造函数

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

关键事实：
- Repository 是**单元结构体**（无字段），方法均为**静态方法**
- 第一个参数始终是 `pool: &SqlitePool`（不存储在 `self` 中）
- **无构造函数**，**无 DI 模式**——不存在 `pub fn new(db)` 
- 返回 `Result<T, sqlx::Error>`（调用方转换为 AppError）

### 文件列表（13 个文件）
| 文件 | 实体 | 说明 |
|------|------|------|
| `inventory_repo.rs` | 库存 | 动态查询、库存移动 |
| `report_repo.rs` | 报表 | 聚合查询、日期范围 |
| `pipe_repo.rs` | 钢管规格 | CRUD + 过滤 + 分页 |
| `purchase_order_repo.rs` | 采购订单 | PO CRUD + 状态 |
| `sales_order_repo.rs` | 销售订单 | SO CRUD + ATP 查询 |
| `contract_repo.rs` | 合同 | CRUD + 状态查询 |
| `quality_repo.rs` | 质量 | 证书 + 检测结果 |
| `customer_repo.rs` | 客户 | CRUD |
| `supplier_repo.rs` | 供应商 | CRUD |
| `label_repo.rs` | 标签 | 标签数据查询 |
| `data_io_repo.rs` | 数据导入导出 | 批量读写 |
| `user_repo.rs` | 用户 | 认证查询 |
| `operation_log_repo.rs` | 操作日志 | 插入 + 查询 |

### Repository 约定
1. **模式**：单元结构体 + 静态方法 —— `pub struct XxxRepo;` 然后 `impl XxxRepo { pub async fn ... }`
2. **第一个参数**：始终是 `pool: &SqlitePool`
3. **方法命名**：按操作命名 —— `find_by_*`、`create`、`update`、`delete_soft`
4. **软删除**：所有查询过滤 `WHERE deleted_at IS NULL`
5. **分页**：`LIMIT ? OFFSET ?` 配合 `page`/`page_size` 参数
6. **返回类型**：列表返回 `Result<Vec<Model>>`，单条查询返回 `Result<Option<Model>>`，创建返回 `Result<Model>`
7. **错误类型**：`sqlx::Error`（调用方转换为 `AppError`）
8. **动态查询**：对条件过滤器（如日期范围、状态过滤）使用字符串构建方式
9. **无业务逻辑**：纯 SQL——不包含验证，不做行→结构体映射之外的转换

### 查询模式
- **基本 CRUD**：`sqlx::query_as::<_, Model>(...)` 配合 `.bind()` 传参
- **动态过滤器**：使用 `WHERE 1=1` 模式构建查询字符串，追加条件
- **分页**：同一方法中始终通过 `SELECT COUNT(*)` 返回总数
- **事务**：Repository 方法可在事务中接受 `&mut Transaction<'_, Sqlite>`

## models/（10 个文件——数据库行结构体）

### 模式
```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SeamlessPipe {
    pub id: i64,
    pub pipe_number: String,
    pub grade: String,
    pub od: f64,              // 非 rust_decimal::Decimal
    pub wt: f64,              // 非 rust_decimal::Decimal
    pub length: Option<f64>,  // 非 rust_decimal::Decimal
    pub weight_per_unit: Option<f64>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,   // ISO 8601 文本格式，非 chrono::NaiveDateTime
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
```

### 约定
- 每个数据库表 1 个结构体，每个实体 1 个文件
- `#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]`
- 字段与数据库列完全匹配（可空字段使用 `Option<T>`）
- `sqlx::FromRow` 支持从查询结果自动映射
- 软删除通过 `deleted_at: Option<String>` 追踪
- 时间戳使用 **`String`**（ISO 8601 文本格式），非 `chrono::NaiveDateTime`
- 十进制字段使用 **`f64`**（非 `rust_decimal::Decimal`）

### models/ 与 dto/ 的区别
- **models/**：数据库行结构体——每表一个，镜像数据库模式
- **dto/**：API 请求/响应结构体——包含验证注解，可能聚合多个模型

### 文件列表
`user.rs`、`seamless_pipe.rs`、`screen_pipe.rs`、`inventory.rs`、`purchase_order.rs`、`sales_order.rs`、`quality.rs`、`contract.rs`、`customer.rs`、`supplier.rs`
