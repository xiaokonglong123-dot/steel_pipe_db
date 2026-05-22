# `services/` — 业务逻辑层（12 个文件）

此处是业务规则、跨实体编排和事务管理的所在地。服务由处理器调用，并调用仓库。

## 模式
```rust
pub struct PipeService;  // 无字段、无构造函数、无依赖注入

impl PipeService {
    pub async fn list_seamless_pipes(
        pool: &SqlitePool,
        params: &PipeFilterParams,
        pagination: &PaginationParams,
    ) -> Result<(Vec<SeamlessPipe>, i64), AppError> {
        // 1. 校验业务规则
        // 2. 调用仓库
        // 3. 转换/聚合结果
        // 4. 返回
    }
}
```

## 服务文件列表
| 文件 | 实体 | 描述 |
|------|------|-------------|
| `auth_service.rs` | 认证 | 登录、令牌刷新、密码验证 |
| `pipe_service.rs` | 钢管 | 钢管 CRUD、钢级/热处理校验 |
| `inventory_service.rs` | 库存 | 入库、出库、ATP 计算、库位管理、库存校验 |
| `purchase_sales_service.rs` | 采购 + 销售 | 采购单/销售单生命周期、审批流程、驳回原因、ATP 校验 |
| `quality_service.rs` | 质量 | 质检证书创建、力学/无损检测录入 |
| `contract_service.rs` | 合同 | 合同 CRUD、里程碑跟踪 |
| `customer_service.rs` | 客户 | 客户 CRUD、编码唯一性 |
| `supplier_service.rs` | 供应商 | 供应商 CRUD、资质管理 |
| `label_service.rs` | 标签 | 标签内容生成 |
| `report_service.rs` | 报表 | 仪表盘聚合、统计报表 |
| `data_io_service.rs` | 数据 IO | Excel/CSV 导入解析、导出格式化 |
| `trace_service.rs` | 追溯 | 库存变动审计追溯 |

## 服务约定
1. **模式**：单元结构体 + 静态方法 —— `pub struct XxxService;` 然后 `impl XxxService { pub async fn ... }`
2. **首个参数**：始终为 `pool: &SqlitePool`
3. **返回类型**：始终为 `Result<T, AppError>`
4. **命名**：`list_*`、`get_*`、`create_*`、`update_*`、`delete_*`
5. **事务**：使用 `sqlx::Transaction::begin(&pool).await`，将 `&mut *tx` 传递给仓库
6. **跨实体操作**：直接调用多个仓库（通过参数共享 pool）
7. **无 HTTP 逻辑**：服务层从不涉及 HTTP StatusCode、响应格式化或头部

## `inventory_service.rs` 中的关键模式（最大、最复杂）
- 入库/出库及数量校验
- 库存移动追踪
- ATP（可承诺量）计算
- 带动态过滤器的查询构建
- 批量操作
- 报表计算
- 销售订单履约库存校验

## 添加新服务
1. 创建 `new_service.rs`
2. 在 `mod.rs` 中添加 `pub mod new_service;`
3. 定义 `pub struct NewService;` 并编写以 `pool: &SqlitePool` 为参数的静态方法
4. 在 `router.rs` 中注册路由
