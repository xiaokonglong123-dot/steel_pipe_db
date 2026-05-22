# `handlers/` — HTTP 层（13 个文件，约 55 个处理器）

## 模式
每个处理器遵循：**提取 → 调用服务 → 响应**

```rust
pub async fn list_seamless_pipes_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<PipeFilterParams>,
) -> Result<Json<PaginatedResponse<SeamlessPipe>>, AppError> {
    let (items, total) = PipeService::list_seamless_pipes(&pool, &filter, &pagination).await?;
    Ok(PaginatedResponse::ok(items, total, page, page_size))
}
```

要点：
- 返回类型：`Result<Json<...>, AppError>` — 而非 `impl IntoResponse`
- 使用 `?` 运算符进行错误传播（AppError 通过 `IntoResponse` 自动转换）
- 无需手动调用 `.into_response()`
- 处理器使用 `ApiResponse::ok()` 或 `PaginatedResponse::ok()` 静态构造函数

## 响应类型（来自 `crate::response`）
- `ApiResponse<T>` — 标准成功响应：`{ "success": true, "request_id": "req_...", "data": T }`
- `PaginatedResponse<T>` — 分页响应：`{ "success": true, "request_id": "req_...", "meta": { total, page, page_size, total_pages }, "data": { "items": [], ... } }`
- `AppError` — 错误响应（通过 `IntoResponse`）：`{ "success": false, "code": 11001, "request_id": "req_...", "message": "...", "details": null }`
- 处理器可使用 `ApiResponse::created(data)` 返回 201 Created 响应
- 处理器可使用 `no_content()` 返回 204 No Content 响应（如删除操作）

## 处理器文件列表

| 文件 | 实体 | 描述 |
|------|------|------|
| `auth_handler.rs` | 认证 | 登录、登出、刷新令牌、个人信息 |
| `pipe_handler.rs` | 钢管 | 无缝管 + 筛管 CRUD、列表、筛选 |
| `inventory_handler.rs` | 库存 | 入库、出库、库存查询、库位、盘点 |
| `purchase_handler.rs` | 采购订单 | CRUD、状态转换、审批 |
| `sales_handler.rs` | 销售订单 | CRUD、状态转换、ATP 检查 |
| `quality_handler.rs` | 质量 | 质检证书 CRUD、力学检测、无损检测 |
| `contract_handler.rs` | 合同 | CRUD、里程碑 |
| `customer_handler.rs` | 客户 | CRUD、列表 |
| `supplier_handler.rs` | 供应商 | CRUD、列表 |
| `report_handler.rs` | 报表 | 仪表盘、日报/月报/统计报表 |
| `label_handler.rs` | 标签 | 条码/规格标签生成 |
| `atp_handler.rs` | 可用库存 | 销售订单审批前的 ATP 库存可用量检查 |
| `data_io_handler.rs` | 数据导入导出 | Excel/CSV 导入和导出 |

## 通用提取器模式
- `Extension(pool): Extension<SqlitePool>` — 数据库池（每个处理器必需）
- `Extension(jwt_secret): Extension<String>` — JWT 密钥（认证处理器）
- `Query(params): Query<T>` — GET 查询参数（T: DeserializeOwned）
- `Json(body): Json<T>` — POST/PUT 请求体（T: DeserializeOwned）
- `Path(id): Path<i64>` — URL 路径参数
- `AuthUser(user): AuthUser` — JWT 认证用户提取器

校验通过 `validator::Validate::validate()` 内联完成：
```rust
req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
```

## 约定
- 每个实体一个处理器文件
- 处理函数为 `pub async fn` 返回 `Result<Json<...>, AppError>`
- 始终使用 `ApiResponse::ok()` / `PaginatedResponse::ok()` 静态构造函数
- 通过 `?` 运算符和 AppError 自动转换进行错误传播
- 大多数处理器较薄（5-15 行）—— 业务逻辑在服务层
