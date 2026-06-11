# `services/` — Business Logic Layer (19 files)

This is where the real work happens — business rules, cross-entity orchestration, transaction management. Services get called by handlers and in turn call repositories.

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
| `auth_service.rs` | Auth | 登录, 令牌刷新, 密码验证 |
| `pipe_service.rs` | Pipes | 管道 CRUD, 钢级/热处理验证 |
| `inbound_service.rs` | Inbound | 入库记录创建, 审批, 批量执行 |
| `outbound_service.rs` | Outbound | 出库记录创建, 审批, 库存扣减 |
| `check_service.rs` | Inventory checks | 盘点创建, 项目提交, 完成 |
| `inventory_query_service.rs` | Inventory (read) | 只读库存查询 (列表, 统计) |
| `location_service.rs` | Locations | 仓库位置 CRUD, 分配, 调拨 |
| `purchase_service.rs` | Purchase Orders | 采购订单生命周期, 审批, 拒绝 |
| `sales_service.rs` | Sales Orders | 销售订单生命周期, ATP 验证, 审批 |
| `quality_service.rs` | Quality | 证书创建, 力学/NDT 测试录入 |
| `contract_service.rs` | Contracts | 合同 CRUD, 里程碑跟踪 |
| `customer_service.rs` | Customers | 客户 CRUD, 编码唯一性 |
| `supplier_service.rs` | Suppliers | 供应商 CRUD, 资质管理 |
| `label_service.rs` | Labels | 标签内容生成 |
| `report_service.rs` | Reports | 仪表板聚合, 统计报表 |
| `data_io_service.rs` | Data IO | Excel/CSV 导入解析, 导出格式化 |
| `trace_service.rs` | Trace | 全生命周期管道追溯 / 库存变动审计 |

## Service Conventions

1. **Pattern**: Unit struct with static methods — `pub struct XxxService;` then `impl XxxService { pub async fn ... }`
2. **First parameter**: Always `pool: &SqlitePool`
3. **Return type**: Always `Result<T, AppError>`
4. **Naming**: `list_*`, `get_*`, `create_*`, `update_*`, `delete_*`
5. **Transactions**: Use `sqlx::Transaction::begin(&pool).await`, then pass `&mut *tx` to repos
6. **Cross-entity ops**: Call multiple repositories directly — the pool gets passed around as a parameter
7. **No HTTP logic**: Services don't know about StatusCodes, response formatting, or headers. That's the handler's job.

## Inventory services — split by responsibility

The old monolithic `inventory_service.rs` has been split into focused modules:

- `inbound_service.rs` — stock-in record creation, approval, batch execution
- `outbound_service.rs` — stock-out record creation, approval, stock deduction
- `check_service.rs` — inventory check (盘点) creation, item submission, completion
- `inventory_query_service.rs` — read-only queries (list, statistics, filters)
- `location_service.rs` — warehouse location CRUD, assign, transfer

ATP (Available-to-Promise) 计算位于 `sales_service.rs` (销售订单审批前的 ATP 检查) 和 `atp_handler.rs`。销售订单履约检查在审批前读取 ATP。

## Adding a New Service

1. Create `new_service.rs`
2. Add `pub mod new_service;` to `mod.rs`
3. Define `pub struct NewService;` with static methods taking `pool: &SqlitePool`
4. Register routes in `router.rs`
