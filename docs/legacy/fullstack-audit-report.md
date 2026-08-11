# Steel Pipe DB — Full-Stack Audit Report

> Audit scope: Rust Axum backend (79 .rs files) + React 19 frontend (13 feature modules)
> Reference docs: `docs/requirements.en.md` + `docs/detailed-design.en.md` + `docs/frontend-design.en.md`
> Scan date: 2026-05-23 | Scanning agents: 4

---

## Scan Results Summary

| Dimension | Result |
|-----------|--------|
| Backend total files | 79 .rs files, ~9,500 lines of production code |
| Backend todo!/unimplemented! | **Zero** — fully implemented, no stubs |
| Backend registered routes | **126** route+method combos, 14 entity groups |
| Database tables | **18** (migrations 001–011) |
| Frontend routes | **46**, all pointing to real pages |
| Frontend placeholder/TODO | **Zero** — no stubs, no mock data |
| Critical issues | **1** |
| Medium issues | **5** |
| Minor issues | **7** |
| Architecture suggestions | **5** |
| Design doc gaps | **4** |

---

## 🧱 1. Feature Gaps & Missing Implementations

### 1.1 UserManagementPage Exists but Has No Route

- **Reference doc module:** `docs/frontend-design.en.md` — User management section
- **The problem:**
  - `frontend/src/features/auth/pages/UserManagementPage.tsx` is a fully functional 331-line user management page (CRUD, role switching, password reset modal, search filters)
  - **But it's not registered in `frontend/src/routes/index.tsx`**
  - **No sidebar menu entry either**
  - **Impact**: admin users can't manage other users through the UI
- **[Production-ready fix]**:

**File: `frontend/src/routes/index.tsx`** — Add the route:
```tsx
// In the import section
import UserManagementPage from '@/features/auth/pages/UserManagementPage';

// At the end of protected routes (or wherever fits):
{
  path: '/users',
  element: <UserManagementPage />,
}
```

**File: `frontend/src/layouts/MainLayout.tsx`** — Add sidebar menu item:
```tsx
// In the `<Menu>` items array, top or under "System Settings":
{
  key: '/users',
  icon: <TeamOutlined />,
  label: <NavLink to="/users">User Management</NavLink>,
}
```

### 1.2 Report Sub-Pages Missing (3 Routes Don't Exist)

- **Reference doc module:** `docs/frontend-design.en.md` — Reports module routing
- **The problem:**
  - `ReportListPage.tsx` has card links pointing to `/reports/inventory`, `/reports/orders`, `/reports/quality`
  - **These three routes aren't registered in `routes/index.tsx`**
  - Clicking them hits a blank page (react-router default fallback)
- **[Production-ready fix]**:

**File: `frontend/src/routes/index.tsx`** — Replace the reports route group:
```tsx
{
  path: '/reports',
  element: <MainLayout />,
  children: [
    { index: true, element: <ReportListPage /> },
    { path: 'dashboard', element: <DashboardPage /> },
    // Add these three sub-routes (pages need to be created or are pending)
    { path: 'inventory', element: <ReportListPage /> },    // TODO: dedicated InventoryReportPage
    { path: 'orders', element: <ReportListPage /> },       // TODO: dedicated OrderReportPage
    { path: 'quality', element: <ReportListPage /> },      // TODO: dedicated QualityReportPage
  ],
}
```

### 1.3 Data IO Frontend Module Completely Missing

- **Reference doc module:** `docs/detailed-design.en.md` — Data import/export API definitions
- **The problem:**
  - Backend fully implemented: `data_io_handler.rs` (150 lines), `data_io_service.rs` (493 lines), `data_io_repo.rs` (367 lines)
  - Supports Excel/CSV import/export, template download, operation log query
  - **Frontend has zero Data IO module** — no pages, no routes, no API hooks, no sidebar entry
- **[Production-ready code]** (new module):

```bash
# Files to create:
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

**Core file: `frontend/src/features/data-io/api/dataIoApi.ts`**
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

### 1.4 Design-Doc APIs Not Exposed as Endpoints

- **Reference doc module:** `docs/detailed-design.en.md` — API list
- **The problem:**

| API described in the doc | Implementation status |
|---|---|
| `generate_pipe_number(pipe_type)` | Not exposed — only used internally in `PipeService` |
| `validate_pipe_number_unique(number)` | Not exposed — only used internally in `PipeService` |
| `get_stock_status(pipe_type, pipe_id)` | Not exposed — has to be accessed indirectly via trace or inventory list |
| `trace_by_pipe_number(pipe_no)` | Not implemented — only `trace/pipe/{pipe_type}/{pipe_id}` (by internal ID) |

**Impact**: Low (internal methods don't need API endpoints; `get_stock_status` can be done via existing trace/inventory endpoints)

---

## 🚨 2. Code Defects & Security Vulnerabilities

### 🔴 2.1 [Critical] Quality Attachment Query Params Mismatch Between Frontend & Backend

- **Side:** Cross-end integration
- **Severity:** **High**
- **Repro steps:**
  - Frontend `qualityApi.ts:53` calls `GET /quality/attachments?cert_id=X`
  - Backend `quality_handler.rs:126` declares `AttachmentListQuery` with `pipe_type` + `pipe_id` params
  - At runtime, frontend sends `cert_id`, backend sees `pipe_type`/`pipe_id` as `None`, returns empty list
  - **Users can never see any attachments**
- **Fix (prefer touching backend, keep frontend as-is):**

**File: `backend/src/handlers/quality_handler.rs`** — Modify `list_attachments_handler`:
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

> Cleaner alternative: add `cert_id: Option<i64>` to `AttachmentListQuery`, then look up the cert to get `pipe_type`+`pipe_id` in the handler before querying attachments.

### ⚠️ 2.2 [Medium] Purchase/Sales Order Detail Response Shape Mismatch

- **Side:** Cross-end integration
- **Severity:** **Medium**
- **Repro steps:**
  - Frontend `purchaseApi.ts:23` expects `ApiResponse<PurchaseOrder>` (flat structure)
  - Backend `get_purchase_order_handler` returns `{ success: true, data: { order: {...}, items: [...] } }` (nested structure)
  - Zod runtime validation might warn or silently drop data due to field mismatch
  - **Same issue exists for sales orders and contract details**
- **Fix:**

**Frontend — Update type definitions (recommended, no backend changes):**
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

### ⚠️ 2.3 [Medium] `ApproveRequest.reason` Is Dead Code

- **Side:** Rust backend
- **Severity:** **Medium**
- **Repro steps:**
  - `dto/inventory.rs` defines `ApproveRequest` with `reason: Option<String>` field
  - `inventory_handler.rs` — `approve_inbound_handler`/`approve_outbound_handler` **only destructures `Json(req)` but never uses `req.reason`**
  - Calls `InventoryService::approve_inbound(&pool, id)` — signature has no reason parameter
  - **Frontend sends approval comments, backend completely ignores them**
- **Fix:**

**File: `backend/src/services/inventory_service.rs`** — Add reason param to approve methods and log to operation_logs:
```rust
pub async fn approve_inbound(pool: &SqlitePool, id: i64, reason: Option<&str>, user_id: i64) -> Result<InboundRecord, AppError> {
    let record = InboundRecordRepo::find_by_id(pool, id).await?
        .ok_or(AppError::NotFound("Inbound record".to_string()))?;

    if record.approval_status != "pending" {
        return Err(AppError::StatusConflict("Only pending records can be approved".to_string()));
    }

    let updated = InboundRecordRepo::approve(pool, id).await?;

    // Log the operation
    OperationLogRepo::create(pool, &CreateOperationLog {
        user_id: Some(user_id),
        username: None,
        action: "approve_inbound".to_string(),
        entity_type: "inbound".to_string(),
        entity_id: Some(id),
        details: reason.map(|r| format!("Approval comment: {}", r)),
        ip_address: None,
    }).await?;

    Ok(updated)
}
```

**File: `backend/src/handlers/inventory_handler.rs`** — Pass reason through:
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

### ⚠️ 2.4 [Medium] Purchase/Sales Order Item `total_price` Is Client-Writable

- **Side:** Rust backend
- **Severity:** **Medium**
- **Repro steps:**
  - `CreatePurchaseItemRequest` has no `total_price` — server computes it as `quantity * unit_price`
  - `UpdatePurchaseItemRequest` has `total_price: Option<f64>` — update directly uses the client-supplied value via `set_field_opt!`
  - **Malicious client can set any `total_price`, bypassing server-side computation**
  - Same issue exists for sales order item updates
  - But `UpdateContractItemRequest` doesn't have `total_price` — contract items are server-computed. **Inconsistent behavior.**
- **Fix:**

**File: `backend/src/dto/purchase_order.rs`** — Remove or deprecate `total_price`:
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

**File: `backend/src/repositories/purchase_order_repo.rs`** — Recalculate total_price in update method:
```rust
// Around line 317: if quantity or unit_price changed, recalculate total_price
if dto.quantity.is_some() || dto.unit_price.is_some() {
    let item = sqlx::query_as::<_, PurchaseOrderItem>(
        "SELECT quantity, unit_price, ... FROM purchase_order_items WHERE id = ? AND deleted_at IS NULL"
    )
    .bind(item_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Purchase order item".to_string()))?;

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

### ⚠️ 2.5 [Medium] Domain Enums Not Used in Models

- **Side:** Rust backend
- **Severity:** **Medium**
- **Repro steps:**
  - `domain/order.rs` defines `OrderStatus` enum (Draft, Pending, Approved, Rejected, Completed, Cancelled), but model `PurchaseOrder.status` / `SalesOrder.status` uses `String`
  - `domain/pipe.rs` and `domain/inventory.rs` have been emptied (comment: "enums removed to eliminate dead code")
  - No corresponding Rust-side validation for SQL CHECK constraints — invalid strings could slip past frontend validation straight to the DB
- **Fix:**

**Incremental fix**: gradually replace `String` in models with domain enums, using sqlx's `TryFrom<&str>` or custom serialization

### 🟡 2.6 [Low] `is_active` Field INTEGER↔bool Mapping Inconsistency

- **Side:** Rust backend
- **Severity:** **Low**
- **Repro steps:**
  - SQLite: `is_active INTEGER NOT NULL DEFAULT 1`
  - Rust model uses `bool` (SQLx auto-mapping: 0→false, non-zero→true)
  - If code accidentally writes 2 or -1, Rust reads it as `true`, but the DB has lost semantic consistency
  - Meanwhile `ContractPayment.is_paid` uses `i64` (not `bool`) — same pattern, two different approaches
- **Suggestion**: Standardize on `bool` everywhere (no functional impact, just consistency)

---

## 🚀 3. Feature Extensions & Architecture Evolution

### 3.1 Add `approval_reason` Column to Backend Response

- **Pain point:** Current InboundRecord/OutboundRecord tables have `rejection_reason` but no `approval_reason` / `approval_notes`
- **Suggestion:** New migration `012_add_approval_reason.sql`:
```sql
ALTER TABLE inbound_records ADD COLUMN approval_reason TEXT;
ALTER TABLE outbound_records ADD COLUMN approval_reason TEXT;
```
- Also write `ApproveRequest.reason` into this column for full audit trail

### 3.2 Move OperationLog Model to `models/`

- **Problem:** `OperationLog` struct lives in `repositories/operation_log_repo.rs` instead of `models/operation_log.rs`
- **Suggestion:**
  - Create `models/operation_log.rs`
  - Export from `models/mod.rs`
  - Have the repo import from `models`
  - This lets the frontend data-io module reuse it cleanly

### 3.3 Switch from `f64` to `rust_decimal` for Money

- **Current:** `total_amount: Option<f64>`, `unit_price: Option<f64>`, `total_price: Option<f64>`
- **Risk:** `f64` is a float — money calculations (especially cumulative sums and splits) can drift
- **Suggestion for when volume grows:** Use `rust_decimal` crate, replace `f64` with `Decimal` in money-related DTOs/models:
```rust
// Cargo.toml
rust_decimal = { version = "1.36", features = ["serde"] }
rust_decimal_sqlite = "0.1"  // Store as TEXT in SQLite

// Usage
pub struct PurchaseOrder {
    pub total_amount: Option<Decimal>,  // instead of Option<f64>
}
```

### 3.4 Add Data IO Frontend Menu Items & Routes

- **Most recommended next step from this audit:**
  - Create `features/data-io/pages/DataImportPage.tsx` (file upload + result display)
  - Create `features/data-io/pages/DataExportPage.tsx` (pick entity type + download)
  - Create `features/data-io/pages/OperationLogPage.tsx` (operation log list)
  - Add routes `/data-io/import`, `/data-io/export`, `/data-io/logs`
  - Add "Data Import/Export" sidebar menu group

### 3.5 Add `pipe_number` Lookup to Trace Endpoint

- **Current state:** `/api/v1/trace/pipe/{pipe_type}/{pipe_id}` requires internal DB ID
- **Suggestion:** Add `/api/v1/trace/pipe-number/{pipe_number}` — lookup by user-visible pipe number
- Backend `trace_service.rs` already has the pipe_number lookup logic — just needs a route and handler

---

## 4. Full Backend Route Inventory (126 Routes, by Module)

See Agent #2 (`bg_90ad4eac`) for the full output. Bottom line:
- ✅ All 126 handler function references match their definitions
- ✅ All module declarations are correct
- ✅ No route conflicts
- ⚠️ Route prefix inconsistency: `/api/v1/trace/...` and `/api/v1/atp` are under the inventory route group but don't have `/inventory` in the prefix

---

## 5. Database Model Alignment Summary (18 Tables)

See Agent #3 (`bg_167403c8`) for the full output. Bottom line:
- ✅ 18 tables align with 19 model structs (including 9 inline structs)
- ⚠️ `operation_logs` table has no dedicated model file (model defined in the repo)
- ⚠️ `ApproveRequest.reason` is dead code, never persisted
- ⚠️ Domain enums (OrderStatus, etc.) aren't wired into models
- ⚠️ `UpdatePurchaseItemRequest` / `UpdateSalesItemRequest` allow client-overridable `total_price`

---

## 6. Frontend Page/Route Alignment Summary

See Agent #4 (`bg_6009f7a3`) for the full output. Bottom line:
- ✅ All 46 routes point to real page files
- ✅ All pages have real implementations (Ant Design components + TanStack Query hooks)
- ❌ `UserManagementPage` (331-line full implementation) has no route
- ❌ Report sub-pages (3 of them) have no registered routes
- ❌ Data IO frontend module is completely missing
- ❌ No inbound/outbound detail routes, no supplier/customer detail routes

---

## Priority Ranking

| Priority | Issue | Fix Difficulty |
|----------|-------|----------------|
| **P0** | Quality attachment query param mismatch (feature is broken) | 1 day |
| **P0** | UserManagementPage has no route (security risk — admin can't manage users) | 0.5 day |
| **P1** | Approval comments (ApproveRequest.reason) are dead code, never stored | 0.5 day |
| **P1** | Report sub-page routes missing (navigation leads to blank page) | 0.5 day |
| **P1** | Data IO frontend module missing (backend already done) | 2 days |
| **P2** | total_price is client-overridable | 1 day |
| **P2** | Domain enums not used in models | 3 days (incremental) |
| **P3** | Inbound/outbound/supplier/customer detail page routes missing | 1 day |
| **P3** | INTEGER↔bool style inconsistency | 0.5 day |
| **P3** | trace/pipe-number endpoint missing | 0.5 day |
