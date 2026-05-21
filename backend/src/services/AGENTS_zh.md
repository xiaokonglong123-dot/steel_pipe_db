# `services/` — 业务逻辑层（13 个文件）

此处是业务规则、跨实体编排和事务管理的所在地。服务由处理器调用，并调用仓库。

## 模式
```rust
pub struct PipeService {
    repo: Arc<PipeRepository>,
    inventory_repo: Arc<InventoryRepository>,
    db: Pool<Sqlite>,
}

impl PipeService {
    pub fn new(repo: Arc<PipeRepository>, inventory_repo: Arc<InventoryRepository>, db: Pool<Sqlite>) -> Self { ... }
    
    pub async fn list_pipes(&self, params: &PipeListParams) -> Result<PagedResult<PipeDto>, AppError> {
        // 1. 校验业务规则
        // 2. 调用仓库
        // 3. 转换/聚合结果
        // 4. 返回
    }
}
```

## 服务文件列表
| 文件 | 描述 | 大小 |
|------|-------------|------|
| `auth_service.rs` | 登录、注册、令牌管理 | ~200 行 |
| `pipe_service.rs` | 钢管规格、产品配置 | ~300 行 |
| `inventory_service.rs` | 库存管理（764 行 —— 最大） | **764** |
| `purchase_service.rs` | 采购订单、审批 | ~350 行 |
| `production_service.rs` | 生产订单 | ~250 行 |
| `report_service.rs` | 报表生成 | ~400 行 |
| `contract_service.rs` | 合同 | ~200 行 |
| `customer_service.rs` | 客户管理 | ~200 行 |
| `supplier_service.rs` | 供应商管理 | ~200 行 |
| `category_service.rs` | 分类管理 | ~150 行 |
| `warehouse_service.rs` | 仓库管理 | ~200 行 |
| `dashboard_service.rs` | 仪表盘聚合 | ~200 行 |

## 服务约定
1. **构造函数**：`pub fn new(...repos..., db: Pool<Sqlite>) -> Self` —— 通过构造函数进行依赖注入
2. **返回类型**：始终为 `Result<T, AppError>`，其中 T 为实体特定类型
3. **命名**：方法名与处理器操作对应：`list_*`、`get_*`、`create_*`、`update_*`、`delete_*`
4. **事务**：多仓库操作使用 `sqlx::Transaction` 包裹：
   ```rust
   let mut tx = self.db.begin().await.map_err(AppError::from)?;
   // ... 使用 &mut *tx 调用仓库 ...
   tx.commit().await.map_err(AppError::from)?;
   ```
5. **跨实体操作**：单个服务可调用多个仓库（例如 inventory_service 调用 inventory_repo 和 pipe_repo）
6. **无 HTTP 逻辑**：服务层从不涉及 HTTP StatusCode、响应格式化或头部

## `inventory_service.rs` 中的关键模式（最大、最复杂）
- 入库/出库及数量校验
- 库存移动追踪
- 带动态过滤器的查询构建
- 批量操作
- 报表计算

## 添加新服务
1. 创建 `new_service.rs`
2. 在 `mod.rs` 中添加 `pub mod new_service;`
3. 定义 `pub struct NewService { ... }` 并带构造函数
4. 在 `mod.rs` 中注册，或在 `main.rs` 中传递给 AppState
