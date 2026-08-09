# `handlers/` — HTTP 层（16 个文件，约 200 个处理器）

## 模式

每个 handler 遵循相同模式：**提取 → 调服务 → 响应**

```rust
pub async fn list_items_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<ItemFilterParams>,
) -> Result<Json<PaginatedResponse<Item>>, AppError> {
    let (items, total) = ItemService::list_items(&pool, &filter, &pagination).await?;
    Ok(PaginatedResponse::ok(items, total, page, page_size))
}
```

要点：

- 返回类型：`Result<Json<...>, AppError>` — 不是 `impl IntoResponse`
- 使用 `?` 传播错误（AppError 通过 `IntoResponse` 自行转换）
- 不要手动调用 `.into_response()` — 保持干净
- handler 使用 `ApiResponse::ok()` 或 `PaginatedResponse::ok()` 静态构造器

## 响应类型（来自 `crate::response`）

- `ApiResponse<T>` — 标准成功：`{ "success": true, "request_id": "req_...", "data": T }`
- `PaginatedResponse<T>` — 分页：`{ "success": true, "request_id": "req_...", "meta": { total, page, page_size, total_pages }, "data": { "items": [], ... } }`
- `AppError` — 错误（经 `IntoResponse`）：`{ "success": false, "code": 11001, "request_id": "req_...", "message": "...", "details": null }`
- 需要 201？`ApiResponse::created(data)` 已覆盖
- 需要 204？`no_content()` — 主要用于删除

## Handler 文件清单

| File | Entity | Description |
| ------ | -------- | ------------- |
| `auth_handler.rs` | Auth | 登录、登出、刷新、个人资料 |
| `item_handler.rs` | Items | 商品/SKU 主数据 CRUD、列表、筛选 |
| `inbound_handler.rs` | Inbound | 入库记录 CRUD、审批、批量 |
| `outbound_handler.rs` | Outbound | 出库记录 CRUD、审批 |
| `location_handler.rs` | Locations | 库位 CRUD、分配、调拨 |
| `check_handler.rs` | Count Sessions | 盘点 CRUD、提交、完成 |
| `inventory_handler.rs` | Inventory | 库存查询、日志、统计 |
| `purchase_handler.rs` | Purchase Orders | 采购订单 CRUD、状态流转、审批 |
| `sales_handler.rs` | Sales Orders | 销售订单 CRUD、状态流转、ATP 检查 |
| `contract_handler.rs` | Contracts | CRUD、里程碑 |
| `customer_handler.rs` | Customers | CRUD、列表 |
| `supplier_handler.rs` | Suppliers | CRUD、列表 |
| `report_handler.rs` | Reports | 仪表板、日报/月报/统计报表 |
| `data_io_handler.rs` | Data IO | Excel/CSV 导入导出（通用实体：商品、订单） |
| `atp_handler.rs` | ATP | 销售订单审批前的 ATP（可用库存）检查 |
| `health_handler.rs` | Health | 存活/就绪探针 |

功能模块在模块内部自带 handler：`workflow/handlers.rs`（审批流定义/实例/任务）、`hr/handlers.rs`、`finance/handlers.rs`、`procurement/handlers.rs`、`sales_crm/handlers.rs`、`inventory_atp/handlers.rs`、`manufacturing/handlers.rs`（含质检）、`project/handlers.rs`、`assets/handlers.rs`、`notification/handlers.rs`、`portal/handlers.rs`、`bi/handlers.rs`、以及 `auth/handlers.rs`（角色/权限/部门/租户）。它们遵循相同的 提取 → 调服务 → 响应 模式。

## 常用提取器模式

- `Extension(pool): Extension<SqlitePool>` — DB 连接池（每个 handler 都需要）
- `Extension(jwt_secret): Extension<JwtSecret>` — JWT 密钥 newtype（仅认证 handler）
- `Query(params): Query<T>` — GET 查询参数（T 需 DeserializeOwned）
- `Json(body): Json<T>` — POST/PUT 请求体（T 需 DeserializeOwned）
- `Path(id): Path<i64>` — URL 路径参数
- `AuthUser(user): AuthUser` — JWT 认证用户提取器

校验通过 `validator::Validate::validate()` 内联完成：

```rust
req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
```

## 约定

- 每实体一个 handler 文件 — 保持组织清晰
- handler 函数为 `pub async fn`，返回 `Result<Json<...>, AppError>`
- 始终使用 `ApiResponse::ok()` / `PaginatedResponse::ok()` — 不要手动构造响应
- 通过 `?` 传播错误，AppError 自动转换
- 大多数 handler 很薄（5-15 行）— 业务逻辑放在 services 层
