# `handlers/` — HTTP 层（13 个文件，110+ 个处理器）

## 模式
每个处理器遵循：**提取 → 校验 → 调用服务 → 响应**

```rust
pub async fn list_pipes(
    Extension(pool): Extension<SqlitePool>,
    Query(params): Query<PipeListParams>,
) -> impl IntoResponse {
    // 1. 校验参数（如需要）
    // 2. 调用服务
    match pipe_service::list_pipes(&pool, &params).await {
        Ok(result) => Json(ApiResponse::success(result)).into_response(),
        Err(e) => e.into_response(),
    }
}
```

## 响应类型（来自 `dto/api_response.rs`）
- `ApiResponse<T>` — 标准成功响应：`{ "code": 200, "message": "ok", "data": T }`
- `PagedResponse<T>` — 分页响应：`{ "code": 200, "data": { "items": [...], "total": N, "page": P, "page_size": S } }`
- `ErrorResponse` — 错误响应：`{ "code": N, "message": "..." }`

## 处理器文件列表
| 文件 | 实体 | 端点 |
|------|--------|-----------|
| `auth_handler.rs` | 认证 | login、register、profile、refresh |
| `pipe_handler.rs` | 钢管（规格） | CRUD + 列表 |
| `inventory_handler.rs` | 库存 | CRUD + 列表 |
| `purchase_handler.rs` | 采购订单 | CRUD + 状态转换 |
| `production_handler.rs` | 生产 | CRUD + 状态 |
| `report_handler.rs` | 报表 | 各类报表端点 |
| `contract_handler.rs` | 合同 | CRUD |
| `customer_handler.rs` | 客户 | CRUD |
| `supplier_handler.rs` | 供应商 | CRUD |
| `category_handler.rs` | 实体分类 | CRUD |
| `warehouse_handler.rs` | 仓库 | CRUD |
| `dictionary_handler.rs` | 字典/配置 | CRUD |
| `dashboard_handler.rs` | 仪表盘 | 摘要/统计 |

## 通用提取器模式
- `Extension(pool): Extension<SqlitePool>` — 数据库池（每个处理器必需）
- `Extension(jwt_secret): Extension<String>` — JWT 密钥（认证处理器）
- `Query(params): Query<T>` — GET 查询参数（T: DeserializeOwned）
- `Json(body): Json<T>` — POST/PUT 请求体（T: DeserializeOwned）
- `Path(id): Path<i64>` — URL 路径参数
- `AuthUser(user): AuthUser` — JWT 认证用户提取器
- `ValidatedRequest<T>` — 校验后的 JSON 请求体（自定义提取器，使用 `validator` crate）

## 约定
- 每个实体一个处理器文件
- 处理函数为 `pub async fn` 返回 `impl IntoResponse`
- 200 响应始终使用 `Json(ApiResponse::success(...))`
- POST 创建使用 `StatusCode::CREATED`：`(StatusCode::CREATED, Json(ApiResponse::success(data)))`
- 通过 `?` 运算符和 AppError 转换进行错误传播
- 大多数处理器较薄（5-15 行）—— 业务逻辑在服务层
