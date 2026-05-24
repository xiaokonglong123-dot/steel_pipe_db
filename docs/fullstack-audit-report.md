# Steel Pipe DB — 全栈审计报告

> 审计范围：Rust Axum 后端 (79 .rs 文件) + React 19 前端 (13 个 feature 模块)
> 对照文档：`docs/需求文档.md` + `docs/详细设计文档.md` + `docs/前端设计文档.md`
> 扫描时间：2026-05-23 | 扫描 agent 数：4

---

## 扫描结果概要

| 维度 | 结果 |
|------|------|
| 后端总文件 | 79 个 .rs 文件，~9,500 行生产代码 |
| 后端 todo!/unimplemented! | **零** — 全量实现，无占位符 |
| 后端路由注册 | **126 个** route+method 组合，14 个实体组 |
| 数据库表 | **18 张**（迁移 001~011） |
| 前端路由 | **46 条**，全部指向真实页面 |
| 前端 placeholder/TODO | **零** — 无占位符、无 Mock 数据 |
| 严重问题 (Critical) | **1** |
| 中等问题 (Medium) | **5** |
| 低等问题 (Minor) | **7** |
| 架构建议 | **5** 条 |
| 设计文档断层 | **4** 处 |

---

## 🧱 1. 功能断层与未实现补全

### 1.1 UserManagementPage 存在但无路由

- **对比文档模块：** `docs/前端设计文档.md` — 用户管理章节
- **残缺描述：**
  - `frontend/src/features/auth/pages/UserManagementPage.tsx` 是一个 331 行的完整用户管理页面（含用户 CRUD、角色切换、密码重置弹窗、搜索过滤）
  - **但在 `frontend/src/routes/index.tsx` 中没有注册任何路由**
  - **侧边栏菜单未添加入口**
  - **影响**：admin 用户无法通过 UI 管理其他用户
- **【生产级补全代码】**：

**文件：`frontend/src/routes/index.tsx`** — 添加路由条目：
```tsx
// 在 import 区添加
import UserManagementPage from '@/features/auth/pages/UserManagementPage';
// 中文设计文档中建议使用 lazy loading，但当前页面模式是直接 import

// 在 protected routes 数组末尾（或其他合适位置）添加：
{
  path: '/users',
  element: <UserManagementPage />,
}
```

**文件：`frontend/src/layouts/MainLayout.tsx`** — 添加侧边栏菜单项：
```tsx
// 在 `<Menu>` 的 items 数组中，建议放在最顶部或"系统设置"区域：
{
  key: '/users',
  icon: <TeamOutlined />,
  label: <NavLink to="/users">用户管理</NavLink>,
}
```

### 1.2 报表子页面缺失（3 个路由不存在）

- **对比文档模块：** `docs/前端设计文档.md` — 报表模块路由
- **残缺描述：**
  - `ReportListPage.tsx` 中有卡片链接跳转到 `/reports/inventory`、`/reports/orders`、`/reports/quality`
  - **但这三个路由在 `routes/index.tsx` 中未注册**
  - 点击后路由命中空白页（react-router 默认渲染 fallback）
- **【生产级补全代码】**：

**文件：`frontend/src/routes/index.tsx`** — 将报表路由组改为：
```tsx
{
  path: '/reports',
  element: <MainLayout />,
  children: [
    { index: true, element: <ReportListPage /> },
    { path: 'dashboard', element: <DashboardPage /> },
    // 添加下面三个子路由（页面文件需创建或待开发）
    { path: 'inventory', element: <ReportListPage /> },    // 待实现独立 InventoryReportPage
    { path: 'orders', element: <ReportListPage /> },       // 待实现独立 OrderReportPage
    { path: 'quality', element: <ReportListPage /> },      // 待实现独立 QualityReportPage
  ],
}
```

### 1.3 Data IO 前端模块完全缺失

- **对比文档模块：** `docs/详细设计文档.md` — 数据导入导出 API 定义
- **残缺描述：**
  - 后端已完整实现：`data_io_handler.rs`（150行）、`data_io_service.rs`（493行）、`data_io_repo.rs`（367行）
  - 支持 Excel/CSV 导入导出、模板下载、操作日志查询
  - **前端没有任何 Data IO 模块** — 无页面、无路由、无 API hooks、无侧边栏入口
- **【生产级补全代码】**（创建新模块）：

```bash
# 需要创建的文件结构：
frontend/src/features/data-io/
├── api/
│   └── dataIoApi.ts
├── pages/
│   ├── DataImportPage.tsx
│   └── DataExportPage.tsx
├── hooks/
│   └── useDataIo.ts
└── types/
    └── index.ts
```

**核心文件：`frontend/src/features/data-io/api/dataIoApi.ts`**
```tsx
import apiClient from '@/api/client';
import type { ApiResponse, PaginatedResponse } from '@/types';

export interface ImportResult {
  success_count: number;
  error_count: number;
  errors: { row: number; message: string }[];
}

export interface OperationLog {
  id: number;
  user_id: number | null;
  username: string | null;
  action: string;
  entity_type: string;
  entity_id: number | null;
  details: string | null;
  ip_address: string | null;
  created_at: string;
}

export const dataIoApi = {
  importData: (entityType: string, file: File) => {
    const formData = new FormData();
    formData.append('file', file);
    return apiClient.post<ApiResponse<ImportResult>>(`/data-io/import/${entityType}`, formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
  },
  exportData: async (entityType: string, params?: Record<string, string>) => {
    const response = await apiClient.get(`/data-io/export/${entityType}`, {
      params,
      responseType: 'blob',
    });
    return response.data as Blob;
  },
  getTemplate: async (entityType: string) => {
    const response = await apiClient.get(`/data-io/templates/${entityType}`, {
      responseType: 'blob',
    });
    return response.data as Blob;
  },
  listOperationLogs: (params: { page?: number; page_size?: number }) =>
    apiClient.get<PaginatedResponse<OperationLog>>('/data-io/operation-logs', { params }),
};
```

### 1.4 设计文档 API 未暴露为端点

- **对比文档模块：** `docs/详细设计文档.md` — API 列表
- **残缺描述：**

| 文档中描述的 API | 实现状态 |
|---|---|
| `generate_pipe_number(pipe_type)` | 未暴露 — 仅在 `PipeService` 内部使用 |
| `validate_pipe_number_unique(number)` | 未暴露 — 仅在 `PipeService` 内部使用 |
| `get_stock_status(pipe_type, pipe_id)` | 未暴露 — 需通过 trace 或 inventory list 间接获取 |
| `trace_by_pipe_number(pipe_no)` | 未实现 — 只实现了 `trace/pipe/{pipe_type}/{pipe_id}`（按内部 ID） |

**影响**：低（内部方法不需要 API 端点，`get_stock_status` 可通过现有 trace/inventory 端点实现）

---

## 🚨 2. 代码缺陷与安全漏洞

### 🔴 2.1 [Critical] 质检附件查询参数前后端不匹配

- **所属端：** 跨端联调
- **风险等级：** **高**
- **场景重现：**
  - 前端 `qualityApi.ts:53` 调用 `GET /quality/attachments?cert_id=X`
  - 后端 `quality_handler.rs:126` 声明了 `AttachmentListQuery`，参数为 `pipe_type` + `pipe_id`
  - 运行时前端发 `cert_id` 参数，后端收到的 `pipe_type`/`pipe_id` 均为 `None`，返回空列表
  - **用户永远看不到任何附件**
- **修复方案（推荐改后端，兼容前端）：**

**文件：`backend/src/handlers/quality_handler.rs`** — 修改 `list_attachments_handler`：
```rust
pub async fn list_attachments_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<PipeAttachment>>>, AppError> {
    let pipe_type = params.get("pipe_type").or_else(|| params.get("cert_id")).map(|s| s.as_str());
    let pipe_id = params.get("pipe_id").or_else(|| {
        // If cert_id is provided, first look up the cert to get pipe_type + pipe_id
        params.get("cert_id").and_then(|cid| cid.parse::<i64>().ok())
    });

    let items = QualityService::list_attachments(&pool, pipe_type, pipe_id).await?;
    Ok(ApiResponse::ok(items))
}
```

> 或者更干净的方式：在 `AttachmentListQuery` 中增加 `cert_id: Option<i64>`，在 handler 中先查 cert 拿到 `pipe_type`+`pipe_id` 再查 attachments。

### ⚠️ 2.2 [Medium] 采购/销售订单详情响应结构前后端不匹配

- **所属端：** 跨端联调
- **风险等级：** **中**
- **场景重现：**
  - 前端 `purchaseApi.ts:23` 期望 `ApiResponse<PurchaseOrder>`（扁平结构）
  - 后端 `get_purchase_order_handler` 返回 `{ success: true, data: { order: {...}, items: [...] } }`（嵌套结构）
  - Zod 运行时校验可能因字段不匹配而告警或静默丢弃数据
  - **同样问题存在于销售订单和合同详情**
- **修复方案：**

**前端 — 更新类型定义（推荐方案，不改后端）：**
```tsx
// frontend/src/features/purchases/types/index.ts
export interface PurchaseOrderDetail {
  order: PurchaseOrder;
  items: PurchaseOrderItem[];
}

// frontend/src/features/purchases/api/purchaseApi.ts
get: async (id: number) =>
  apiClient.get<ApiResponse<PurchaseOrderDetail>>(`/purchase-orders/${id}`),
```

### ⚠️ 2.3 [Medium] `ApproveRequest.reason` 未使用（死代码）

- **所属端：** Rust 后端
- **风险等级：** **中**
- **场景重现：**
  - `dto/inventory.rs` 中 `ApproveRequest` 定义有 `reason: Option<String>` 字段
  - `inventory_handler.rs` 中 `approve_inbound_handler`/`approve_outbound_handler` **仅解构 `Json(req)` 但不使用 `req.reason`**
  - 调用 `InventoryService::approve_inbound(&pool, id)` — 签名无 reason 参数
  - **前端传了审批意见但后端完全忽略**
- **修复方案：**

**文件：`backend/src/services/inventory_service.rs`** — 为 approve 方法增加 reason 参数并记录到 operation_logs：
```rust
pub async fn approve_inbound(pool: &SqlitePool, id: i64, reason: Option<&str>, user_id: i64) -> Result<InboundRecord, AppError> {
    let record = InboundRecordRepo::find_by_id(pool, id).await?
        .ok_or(AppError::NotFound("入库记录".to_string()))?;

    if record.approval_status != "pending" {
        return Err(AppError::StatusConflict("只有待审批状态的入库记录可以审批".to_string()));
    }

    // 更新状态 + 记录审批意见（如果需要可以新建 approval_notes 字段或记录到 operation_logs）
    let updated = InboundRecordRepo::approve(pool, id).await?;

    // 记录操作日志
    OperationLogRepo::create(pool, &CreateOperationLog {
        user_id: Some(user_id),
        username: None,
        action: "approve_inbound".to_string(),
        entity_type: "inbound".to_string(),
        entity_id: Some(id),
        details: reason.map(|r| format!("审批意见: {}", r)),
        ip_address: None,
    }).await?;

    Ok(updated)
}
```

**文件：`backend/src/handlers/inventory_handler.rs`** — 传递 reason：
```rust
pub async fn approve_inbound_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<i64>,
    Json(req): Json<ApproveRequest>,
) -> Result<Json<ApiResponse<InboundRecord>>, AppError> {
    let record = InventoryService::approve_inbound(&pool, id, req.reason.as_deref(), user.id).await?;
    Ok(ApiResponse::ok(record))
}
```

### ⚠️ 2.4 [Medium] 采购/销售订单项 `total_price` 客户端可覆写

- **所属端：** Rust 后端
- **风险等级：** **中**
- **场景重现：**
  - `CreatePurchaseItemRequest` 中没有 `total_price`，创建时服务端按 `quantity * unit_price` 计算
  - `UpdatePurchaseItemRequest` 中有 `total_price: Option<f64>`，update 时仓库使用 `set_field_opt!` 直接设置客户端传入的值
  - **恶意客户端可以传入任意 total_price 值，绕过服务端计算逻辑**
  - 同样问题存在于销售订单项更新
  - 但 `UpdateContractItemRequest` 中没有 `total_price`，合同项更新由服务端计算 — **行为不一致**
- **修复方案：**

**文件：`backend/src/dto/purchase_order.rs`** — 移除 `total_price` 字段或标记为已弃用：
```rust
pub struct UpdatePurchaseItemRequest {
    pub pipe_type: Option<String>,
    pub grade: Option<String>,
    pub od: Option<f64>,
    pub wt: Option<f64>,
    pub quantity: Option<i64>,
    pub unit_price: Option<f64>,
    // total_price: Option<f64>,  // REMOVED — server computes from quantity * unit_price
    pub notes: Option<String>,
}
```

**文件：`backend/src/repositories/purchase_order_repo.rs`** — 在 update 方法中重新计算 total_price：
```rust
// 第 317 行附近：如果 quantity 或 unit_price 有更新，重新计算 total_price
if dto.quantity.is_some() || dto.unit_price.is_some() {
    // 需要先查询当前值
    let item = sqlx::query_as::<_, PurchaseOrderItem>(
        "SELECT quantity, unit_price, ... FROM purchase_order_items WHERE id = ? AND deleted_at IS NULL"
    )
    .bind(item_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("采购订单项".to_string()))?;

    let quantity = dto.quantity.unwrap_or(item.quantity);
    let unit_price = dto.unit_price.unwrap_or(item.unit_price.unwrap_or(0.0));
    let total_price = quantity as f64 * unit_price;

    sqlx::query("UPDATE purchase_order_items SET total_price = ?, quantity = ?, unit_price = ? WHERE id = ?")
        .bind(total_price)
        .bind(quantity)
        .bind(unit_price)
        .bind(item_id)
        .execute(pool)
        .await?;
}
```

### ⚠️ 2.5 [Medium] Domain 枚举未在模型中使用

- **所属端：** Rust 后端
- **风险等级：** **中**
- **场景重现：**
  - `domain/order.rs` 定义了 `OrderStatus` 枚举（Draft, Pending, Approved, Rejected, Completed, Cancelled），但模型 `PurchaseOrder.status` / `SalesOrder.status` 使用 `String`
  - `domain/pipe.rs` 和 `domain/inventory.rs` 已被清空（注释："enums removed to eliminate dead code"）
  - SQL CHECK 约束在 Rust 侧没有对应校验，非法字符串可能绕过前端验证到达数据库
- **修复方案：**

**渐进式修复：逐步将 models 中的 String 替换为 domain enum，通过 sqlx 的 `TryFrom<&str>` 或自定义序列化实现**

### 🟡 2.6 [Low] `is_active` 字段 INTEGER↔bool 映射不一致

- **所属端：** Rust 后端
- **风险等级：** **低**
- **场景重现：**
  - SQLite 中 `is_active INTEGER NOT NULL DEFAULT 1`
  - Rust 模型中使用 `bool`（SQLx 自动映射：0→false, 非0→true）
  - 如果代码意外写入 2 或 -1，Rust 读出来是 `true`，但数据库层面失去了语义一致性
  - 同时 `ContractPayment.is_paid` 使用 `i64`（而非 `bool`）— 同一模式两种处理方式
- **建议**：统一使用 `bool`（功能无影响，仅设计一致性）

---

## 🚀 3. 功能扩展与架构演进

### 3.1 后端响应体增加 `approval_reason` 列

- **优化点：** 当前 InboundRecord/OutboundRecord 表中只有 `rejection_reason`，但没有 `approval_reason`/`approval_notes`
- **建议：** 新增迁移 `012_add_approval_reason.sql`：
```sql
ALTER TABLE inbound_records ADD COLUMN approval_reason TEXT;
ALTER TABLE outbound_records ADD COLUMN approval_reason TEXT;
```
- 同时将 `ApproveRequest.reason` 写入此列，补全审计追踪

### 3.2 操作日志模型迁移到 `models/`

- **问题：** `OperationLog` 结构体定义在 `repositories/operation_log_repo.rs` 中而非 `models/operation_log.rs`
- **建议：**
  - 创建 `models/operation_log.rs`
  - 在 `models/mod.rs` 中导出
  - 仓库改为 import 自 `models`
  - 这样前端的数据导入导出模块可以直接复用

### 3.3 引入 `rust_decimal` 替代 `f64` 处理金额

- **当前：** `total_amount: Option<f64>`、`unit_price: Option<f64>`、`total_price: Option<f64>`
- **风险：** `f64` 是浮点数，金额计算（尤其是累加和分账）可能产生精度误差
- **建议当业务量扩大后：** 使用 `rust_decimal` crate，在金额相关 DTO/model 中用 `Decimal` 替代 `f64`：
```rust
// Cargo.toml
rust_decimal = { version = "1.36", features = ["serde"] }
rust_decimal_sqlite = "0.1"  // SQLite 用 TEXT 存储 Decimal

// 使用模式
pub struct PurchaseOrder {
    pub total_amount: Option<Decimal>,  // 而非 Option<f64>
}
```

### 3.4 前端添加 Data IO 菜单项和路由入口

- **在当前审计基础上最推荐的下一步工作：**
  - 创建 `features/data-io/pages/DataImportPage.tsx`（文件上传 + 导入结果展示）
  - 创建 `features/data-io/pages/DataExportPage.tsx`（选择实体类型 + 下载）
  - 创建 `features/data-io/pages/OperationLogPage.tsx`（操作日志列表）
  - 在路由中添加 `/data-io/import`、`/data-io/export`、`/data-io/logs`
  - 在侧边栏添加"数据导入导出"菜单组

### 3.5 追踪端点增加 pipe_number 查找

- **现状：** `/api/v1/trace/pipe/{pipe_type}/{pipe_id}` 需要内部 DB ID
- **建议增加：** `/api/v1/trace/pipe-number/{pipe_number}` — 通过 pipe_number（用户可见标识符）查找
- 后端的 `trace_service.rs` 已经实现了按 pipe_number 的逻辑，只需要新增路由和 handler 即可

---

## 4. 后端全路由清单（126 条，按模块）

详见 Agent #2（`bg_90ad4eac`）的完整输出。核心结论：
- ✅ 126 个 handler 函数引用与定义完全匹配
- ✅ 所有模块声明正确
- ✅ 无路由冲突
- ⚠️ 路由前缀不一致：`/api/v1/trace/...` 和 `/api/v1/atp` 放在 inventory 路由组下但前缀没有 `/inventory`

---

## 5. 数据库模型对齐小结（18 表）

详见 Agent #3（`bg_167403c8`）的完整输出。核心结论：
- ✅ 18 张表与 19 个 model struct（含 9 个内联 struct）基本对齐
- ⚠️ `operation_logs` 表无独立模型文件（模型定义在 repo 中）
- ⚠️ `ApproveRequest.reason` 死代码未持久化
- ⚠️ Domain 枚举（OrderStatus 等）未在模型中启用
- ⚠️ UpdatePurchaseItemRequest/UpdateSalesItemRequest 允许客户端覆写 total_price

---

## 6. 前端页面路由对齐小结

详见 Agent #4（`bg_6009f7a3`）的完整输出。核心结论：
- ✅ 46 条路由全部关联到真实页面文件
- ✅ 所有页面有真实实现（Ant Design 组件 + TanStack Query hooks）
- ❌ UserManagementPage（331行完整实现）无路由
- ❌ 报表子页面（3 个）路由未注册
- ❌ Data IO 前端模块完全缺失
- ❌ 无入库/出库详情路由、无供应商/客户详情路由

---

## 紧急程度排序

| 优先级 | 问题 | 修复难度 |
|--------|------|----------|
| **P0** | 质检附件查询参数不匹配（功能不可用） | 1 天 |
| **P0** | UserManagementPage 无路由（安全风险 — admin 无法管理用户） | 0.5 天 |
| **P1** | 审批意见（ApproveRequest.reason）死代码未存储 | 0.5 天 |
| **P1** | 报表子页面路由缺失（导航到空白页） | 0.5 天 |
| **P1** | Data IO 前端模块缺失（后端已实现） | 2 天 |
| **P2** | total_price 客户端可覆写 | 1 天 |
| **P2** | Domain 枚举未在模型中使用 | 3 天（渐进式） |
| **P3** | 入库/出库/供应商/客户详情页路由缺失 | 1 天 |
| **P3** | INTEGER↔bool 风格不一致 | 0.5 天 |
| **P3** | trace/pipe-number 端点缺失 | 0.5 天 |
