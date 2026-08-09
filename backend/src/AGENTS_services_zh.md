# `services/` — 业务逻辑层

真正的业务逻辑在这里 — 业务规则、跨实体编排、事务管理。Service 由 handler 调用，并进一步调用仓储。

## 模式

```rust
pub struct ItemService;  // 无字段，无构造器，无 DI

impl ItemService {
    pub async fn list_items(
        pool: &SqlitePool,
        params: &ItemFilterParams,
        pagination: &PaginationParams,
    ) -> Result<(Vec<Item>, i64), AppError> {
        // 1. 校验业务规则
        // 2. 调用仓储
        // 3. 转换/聚合结果
        // 4. 返回
    }
}
```

## Service 文件清单

| File | Entity | Description |
| ------ | -------- | ------------- |
| `auth_service.rs` | Auth | 登录、令牌刷新、密码验证 |
| `item_service.rs` | Items | 商品/SKU 主数据 CRUD、编码唯一性 |
| `inbound_service.rs` | Inbound | 入库记录创建、审批、批量执行 |
| `outbound_service.rs` | Outbound | 出库记录创建、审批、库存扣减 |
| `check_service.rs` | Inventory checks | 盘点创建、项目提交、完成 |
| `inventory_query_service.rs` | Inventory (read) | 只读库存查询（列表、统计） |
| `location_service.rs` | Locations | 库位 CRUD、分配、调拨 |
| `purchase_service.rs` | Purchase Orders | 采购订单生命周期、审批、拒绝原因 |
| `sales_service.rs` | Sales Orders | 销售订单生命周期、ATP 验证、审批 |
| `contract_service.rs` | Contracts | 合同 CRUD、里程碑跟踪 |
| `customer_service.rs` | Customers | 客户 CRUD、编码唯一性 |
| `supplier_service.rs` | Suppliers | 供应商 CRUD、资质管理 |
| `report_service.rs` | Reports | 仪表板聚合、统计报表 |
| `data_io_service.rs` | Data IO | Excel/CSV 导入解析、导出格式化 |

功能模块在模块内部自带服务：`auth/services.rs`（角色/权限/部门/租户）、`workflow/services.rs`（审批引擎）、`hr/services.rs`（员工/考勤/薪资/劳动合同）、`finance/services.rs`（会计科目/日记账/发票/付款/试算平衡）、`procurement/services.rs`（采购申请/收货/报价/供应商评分）、`sales_crm/services.rs`（发货/报价/客户信用）、`inventory_atp/services.rs`（预留/调拨/盘点）、`manufacturing/services.rs`（BOM/工单/质检/不合格品单）、`project/services.rs`、`assets/services.rs`（折旧/处置）、`notification/services.rs`、`portal/services.rs`、`bi/services.rs`（分析）。它们遵循相同的 unit struct 模式。

## Service 约定

1. **模式**：unit struct + 静态方法 — `pub struct XxxService;` 然后 `impl XxxService { pub async fn ... }`
2. **第一个参数**：永远是 `pool: &SqlitePool`
3. **返回类型**：永远是 `Result<T, AppError>`
4. **命名**：`list_*`、`get_*`、`create_*`、`update_*`、`delete_*`
5. **事务**：使用 `sqlx::Transaction::begin(&pool).await`，然后把 `&mut *tx` 传给仓储
6. **跨实体操作**：直接调用多个仓储 — 连接池作为参数传递
7. **无 HTTP 逻辑**：Service 不知道 StatusCode、响应格式化或请求头。那是 handler 的事。

## 库存服务 — 按职责拆分

库存服务层拆分为专注的模块：

- `inbound_service.rs` — 入库（创建、审批、批量执行）
- `outbound_service.rs` — 出库（创建、审批、库存扣减）
- `check_service.rs` — 盘点（创建、项目提交、完成）
- `inventory_query_service.rs` — 只读查询（列表、统计、筛选）
- `location_service.rs` — 库位 CRUD、分配、调拨

ATP（Available-to-Promise）计算位于 `sales_service.rs`（销售订单审批前的 ATP 检查）和 `atp_handler.rs`。销售订单履约检查在审批前读取 ATP。预留与调拨由 `inventory_atp/services.rs` 管理。

## 新增一个 Service

1. 创建 `new_service.rs`
2. 在 `mod.rs` 中添加 `pub mod new_service;`
3. 定义 `pub struct NewService;`，静态方法接受 `pool: &SqlitePool`
4. 在 `router.rs` 中注册路由
