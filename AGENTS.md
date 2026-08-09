# ERP — Project Index

> 历史沿革：本系统由钢管行业系统重构而来，已重构为通用 ERP（企业资源计划系统）。

## Quick Start

```
# Backend (Rust Axum on :3000, crate erp-server)
cd backend
cp .env.example .env
cargo run

# Frontend (React 19 + Vite on :5173)
cd frontend
npm install
npm run dev

# Login: admin / admin123
```

Backend runs on `http://localhost:3000`, frontend dev on `http://localhost:5173`.
Database is SQLite3 (single file `data/erp.db`, connection string `sqlite://data/erp.db?mode=rwc`) — no external DB server required.

## Build & Verify

| What | How | CI checks |
| ------ | ----- | ----------- |
| Backend type-check | `cd backend && cargo check` | `cargo check` |
| Backend tests | `cd backend && cargo test` | — |
| Frontend type-check | `cd frontend && npx tsc --noEmit` | `tsc --noEmit` |
| Frontend build | `cd frontend && npm run build` | `npm run build` |
| Frontend chunk analysis | `cd frontend && npx vite build --analyze` (manualChunks via vite.config.ts) | — |
| Full CI pipeline | `cargo check` + `tsc --noEmit` + `npm run build` (parallel) | `.github/workflows/ci.yml` |

**Heads up**: There's **no Makefile** despite what the README says. Just use cargo/npm directly.

## Architecture

```
erp/
├── backend/          ← Rust Axum 0.8 REST API (SQLite3, JWT/Argon2) — crate `erp-server` (documented target for the code phase)
│   └── src/
│       ├── main.rs         ← Entry: tracing, DB pool, migrate, start server
│       ├── lib.rs          ← Module declarations re-exported
│       ├── router.rs       ← ~70 endpoints, all routes assembled here
│       ├── handlers/       ← 1 file per entity (thin: extract → call service → respond)
│       ├── services/       ← business logic (unit structs, static methods)
│       ├── repositories/   ← pure SQL, soft-delete aware
│       ├── models/         ← DB row structs (sqlx::FromRow)
│       ├── dto/            ← request/response types
│       ├── domain/         ← enums/domain types
│       ├── middleware/     ← auth.rs + rbac.rs + rate_limit.rs
│       ├── auth/           ← RBAC: repos.rs + services.rs (IdentityService) + handlers.rs (roles/permissions/departments/tenants)
│       ├── workflow/       ← approval engine: repos.rs + services.rs (WorkflowService) + handlers.rs (definitions/instances/tasks)
│       ├── hr/             ← HR: repos.rs + services.rs (HrService) + handlers.rs (employees/attendance/salaries/labor contracts)
│       ├── finance/        ← Finance: repos.rs + services.rs (FinanceService) + handlers.rs (accounts/journal/invoices/payments/trial-balance)
│       ├── procurement/    ← Procurement: repos.rs + services.rs + handlers.rs (requisitions/receipts/supplier quotes/scorecard)
│       ├── sales_crm/      ← Sales CRM: repos.rs + services.rs + handlers.rs (shipments/customer quotes/customer credit)
│       ├── inventory_atp/  ← Inventory (商品/Item+SKU): repos.rs + services.rs + handlers.rs (item master, stock, reservations, transfers, count sessions)
│       ├── manufacturing/  ← Manufacturing: repos.rs + services.rs + handlers.rs (BOMs/work orders/inspections/NCRs)
│       ├── project/        ← Projects: repos.rs + services.rs + handlers.rs (projects/WBS/budget)
│       ├── assets/         ← Fixed assets: repos.rs + services.rs + handlers.rs (registration/straight-line depreciation/disposal)
│       ├── notification/   ← Notifications: repos.rs + services.rs + handlers.rs (inbox/templates/preferences)
│       ├── portal/         ← Portal: repos.rs + services.rs + handlers.rs (portal accounts/party JWT/PO accept/SO ack)
│       ├── bi/             ← BI analytics: services.rs + handlers.rs (sales trend/inventory value/finance summary/supplier perf)
│       ├── config.rs       ← Env-based config (DATABASE_URL=sqlite://data/erp.db?mode=rwc, JWT_SECRET, etc.)
│       ├── error.rs        ← AppError enum, numeric error codes; ApiErrorResponse with success+request_id
│       └── response.rs     ← ApiResponse<T>, PaginatedResponse<T>, Meta struct, request_id (uuid v4), ::created(), no_content()
├── frontend/         ← React 19 + Vite + Ant Design + TanStack Query
│   └── src/
│       ├── main.tsx        ← React DOM entry
│       ├── App.tsx         ← ConfigProvider + QueryClientProvider + RouterProvider
│       ├── api/            ← Axios instance + QueryClient config
│       ├── routes/         ← createBrowserRouter + ProtectedRoute
│       ├── features/       ← per-module features (auth, items, inventory, purchases, sales, workflow, hr, finance, ...)
│       ├── layouts/        ← MainLayout (sidebar + header + Outlet)
│       ├── stores/         ← Zustand authStore, appStore (global state), unitStore (unit conversion)
│       ├── lib/            ← validateResponse.ts, runtime zod response validation
│       ├── styles/         ← Ant Design theme config
│       ├── zod-schemas/    ← Zod schema files for response validation
│       ├── shared/         ← hooks (useDebounce), components/ (9 shared components), utils/
│       └── i18n/           ← react-i18next (zh-CN primary)
└── docs/             ← PRD, design docs, task breakdown
```

## Tech Stack (verified from Cargo.toml / package.json)

### Backend

- **Rust** edition 2021, nightly 2024-02-08
- **Axum 0.8** with macros + multipart features
- **SQLx 0.8** with `sqlite` feature, runtime-tokio, chrono
- **Auth**: jsonwebtoken 9 + argon2 0.5 (NOT bcrypt)
- **Validation**: validator 0.19 with derive
- **Tracing**: tracing + tracing-subscriber with env-filter + json
- **tower-http 0.6**: CORS, trace, request-id
- **Import/Export**: calamine 0.26 (Excel), rust_xlsxwriter 0.80, csv 1.3
- **No `rust_decimal` or `bigdecimal`** — decimals handled via f64 in current code
- **No `build.rs`** — despite being mentioned in subordinate AGENTS.md
- **DB**: SQLite3, connection string `sqlite://data/erp.db?mode=rwc` (WAL mode)

### Frontend

- **React 19** with react-router-dom v7 (createBrowserRouter)
- **Ant Design 5** with @ant-design/icons
- **TanStack Query 5** — server state, 2min staleTime, 5min gcTime
- **Zustand 5** — client auth state (NOT just TanStack Query)
- **Axios** instance at `/api/v1`, auto-attaches Bearer token
- **TypeScript strict** — noUnusedLocals, noUnusedParameters enforced
- **Path alias**: `@/` → `./src/*`
- **i18n**: react-i18next, zh-CN primary, per-feature namespaces
- **zod** — schema validation
- **zod runtime validation** — `src/lib/validateResponse.ts` wraps `zod.response()` for API response validation

## Backend Patterns (what actually runs, not what the docs pretend)

### DI Pattern: Extension layers, NOT State<Arc<AppState>>

```
router.rs: .layer(Extension(pool)).layer(Extension(JwtSecret(jwt_secret)))
Handler:   Extension(pool): Extension<SqlitePool>
Auth:      Extension(jwt_secret): Extension<JwtSecret>
```

No `AppState` struct exists. The pool is injected directly, while the JWT secret uses the `JwtSecret` newtype so it cannot collide with other `String` extensions and redacts itself in debug output.

### Response Shapes

```rust
// Success:    { "success": true, "request_id": "req_...", "data": T }
// Paginated:  { "success": true, "request_id": "req_...", "data": { "items": [], ... }, "meta": { "total": N, "page": P, "page_size": S, "total_pages": N } }
// Error:      { "success": false, "code": 11001, "request_id": "req_...", "message": "...", "details": null }
```

`request_id` is a uuid v4. `Meta` struct has total/page/page_size/total_pages. `ApiErrorResponse` always includes `success: false` and `request_id` — filled automatically by `AppError::into_response()`.
The backend also propagates an `x-request-id` response header via `tower-http`; CORS exposes that header for browser debugging.

### Handler Pattern

```rust
pub async fn list_items_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<ItemFilterParams>,
) -> Result<Json<PaginatedResponse<Item>>, AppError> {
    let (items, total) = ItemService::list_items(&pool, &filter, &pagination).await?;
    Ok(PaginatedResponse::ok(items, total, page, page_size))
}
```

Handlers return `Result<Json<...>, AppError>` (NOT `impl IntoResponse`). Errors propagate via `?`.

### Service Pattern: Unit struct + static methods

```rust
pub struct ItemService;  // No fields, no constructor, no DI

impl ItemService {
    pub async fn create_item(pool: &SqlitePool, dto: &CreateItemRequest) -> Result<Item, AppError> {
        // Business logic here
    }
}
```

Services are **unit structs with static methods**, taking `pool: &SqlitePool` directly. Forget the fancy constructor DI pattern you read about in some blog — this is what we actually do.

### Repository Pattern

```rust
ItemRepo::find_by_sku(pool, sku).await
```

Same deal — static methods, `pool: &SqlitePool`. Soft-delete: `WHERE deleted_at IS NULL`.

### Error Codes (numeric, domain-prefixed)

| Range | Domain |
| ------- | -------- |
| 100xx | General (Internal, Validation, NotFound) |
| 110xx | Auth (Unauthorized, TokenExpired, Forbidden) |
| 120xx | Item/商品 (NotFound, Duplicate, StatusConflict) |
| 130xx | Inventory (InsufficientStock, LocationNotFound) |
| 140xx | Orders (CannotModify, NotFound) |
| 150xx | Inspection/质检 (NotFound, AttachmentNotFound) |
| 160xx | Supplier (NotFound, CodeDuplicate) |
| 170xx | Customer (NotFound, CodeDuplicate) |
| 180xx | Data IO (ImportError, ExportError) |
| 50001 | Database |

### Handler Files (per module)

`auth_handler`, `item_handler`, `inventory_handler`, `purchase_handler`, `sales_handler`, `contract_handler`, `customer_handler`, `supplier_handler`, `report_handler`, `data_io_handler`, `atp_handler`, `workflow_handler`, `hr_handler`, `finance_handler`, `procurement_handler`, `manufacturing_handler`, `project_handler`, `asset_handler`, `notification_handler`, `portal_handler`, `bi_handler`

### Service Files (per module)

`auth_service`, `item_service`, `inbound_service`, `outbound_service`, `check_service`, `inventory_query_service`, `location_service`, `reservation_service`, `transfer_service`, `purchase_service`, `sales_service`, `contract_service`, `customer_service`, `supplier_service`, `report_service`, `data_io_service`, `trace_service`, `workflow_service`, `hr_service`, `finance_service`, `procurement_service`, `manufacturing_service`, `project_service`, `asset_service`, `notification_service`, `portal_service`, `bi_service`

### Repository Files (per module)

`item_repo`, `inventory_repo`, `location_repo`, `warehouse_repo`, `inbound_repo`, `outbound_repo`, `inventory_log_repo`, `check_repo`, `reservation_repo`, `transfer_repo`, `purchase_order_repo`, `sales_order_repo`, `contract_repo`, `customer_repo`, `supplier_repo`, `report_repo`, `data_io_repo`, `user_repo`, `operation_log_repo`, `refresh_token_repo`, `workflow_repo`, `hr_repo`, `finance_repo`, `procurement_repo`, `manufacturing_repo`, `project_repo`, `asset_repo`, `notification_repo`, `portal_repo`, `bi_repo`

### DB Migrations (rewrite target: 37 legacy files → SQLite)

The legacy 37 migrations (including pipe-specific tables and other legacy tables) are **rewritten to SQLite syntax** in the code phase; pipe-specific tables are dropped. The target migration set covers: users/RBAC, items (SKU/名称/分类/单位/规格), warehouses/locations, inventory/inbound/outbound/logs/checks/reservations, suppliers/customers, purchase/sales orders, contracts, HR, finance, procurement, manufacturing (BOMs/work orders/inspections/NCRs), projects, fixed assets, notifications, portal, workflow definitions/instances/tasks, operation logs, refresh tokens.

DB is SQLite3 at `sqlite://data/erp.db?mode=rwc` (sqlx 0.8 `sqlite` feature). No external DB server, no per-test schemas — this is a docs-only description of the migration strategy; the code phase implements it.

## Frontend Patterns

### Routing (react-router-dom v7, createBrowserRouter + RouterProvider)

```
/login                     ← public
/                          ← ProtectedRoute → MainLayout → Outlet
  /items                   ← ItemListPage (+ /new, /:id, /:id/edit)
  /inventory/inbound       ← InboundListPage
  /inventory/inbound/new   ← InboundFormPage
  /inventory/outbound      ← OutboundListPage
  /inventory/outbound/new  ← OutboundFormPage
  /inventory/stock         ← StockQueryPage
  /inventory/locations     ← LocationListPage
  /inventory/check         ← InventoryCheckListPage
  /suppliers               ← SupplierListPage (+ /new, /:id/edit)
  /customers               ← CustomerListPage (+ /new, /:id/edit)
  /purchases               ← (+ /new, /:id, /:id/edit)
  /sales                   ← (+ /new, /:id, /:id/edit)
  /contracts               ← (+ /new, /:id, /:id/edit)
  /workflow                ← WorkflowListPage (+ definitions/instances/tasks)
  /hr                      ← EmployeeListPage (+ attendance/salaries/labor contracts)
  /finance                 ← FinancePage (accounts/journal/invoices/payments/trial-balance)
  /procurement             ← RequisitionListPage (+ receipts/supplier quotes/scorecard)
  /manufacturing           ← WorkOrderListPage (+ BOMs/inspections/NCRs)
  /projects                ← ProjectListPage (+ WBS/budget)
  /assets                  ← AssetListPage (+ depreciation/disposal)
  /notifications           ← NotificationInboxPage
  /portal                  ← PortalAccountListPage
  /reports                 ← ReportListPage
  /reports/dashboard       ← DashboardPage
  /profile/settings        ← ProfileSettingsPage
  /search                  ← SearchPage
```

### Feature Modules

`auth`, `items`, `inventory`, `suppliers`, `customers`, `purchases`, `sales`, `contracts`, `workflow`, `hr`, `finance`, `procurement`, `manufacturing`, `projects`, `assets`, `notifications`, `portal`, `reports`, `search`, `profile`

Each feature has: `api/` (TanStack Query hooks), `pages/` (ListPage, FormPage, DetailPage), `types/` (TS interfaces), and usually `queryKeys.ts` for TanStack Query key factories. Some also have `hooks/` or `store/` or `stores/`.

### Auth Flow

- `authStore` (Zustand, localStorage-backed): stores `auth_token` + `auth_user`
- `apiClient` interceptor auto-attaches `Authorization: Bearer <token>`
- 401 response → clear storage, redirect to `/login`
- `ProtectedRoute` component redirects unauthenticated users

### QueryClient Defaults

```ts
{ staleTime: 2min, gcTime: 5min, retry: 1, refetchOnWindowFocus: false }
```

### API Base

- Axios `baseURL: '/api/v1'`, 30s timeout
- Vite dev proxy: `/api/*` → `http://localhost:3000`

## Conventions & Gotchas

- **No `.opencode.json`** config found — default OpenCode behavior applies
- **No Makefile** — don't try `make backend`, just `cargo run`
- **License**: GPLv2 (was MIT, recently changed)
- **i18n**: zh-CN primary. Namespace per feature. `AGENTS_zh.md` files exist for Chinese-language agent sessions
- **`AGENTS_zh.md`** files exist alongside most `AGENTS.md` for Chinese-language development
- **Type safety**: CI enforces `cargo check` (not build) + `tsc --noEmit`. No Rust tests run in CI
- **Dead code cleanup**: 26 unused items removed from domain/dto/error/response/repo modules. `#![allow(dead_code)]` retained at crate root to suppress legitimate false positives.
- **Path params**: Axum 0.8 uses `{id}` syntax (not `:id` as in Axum 0.7)
- **JWT secret uses `JwtSecret` newtype** — no bare `Extension<String>` for auth secrets; missing secret extension fails closed with 500
- **No State extractor** anywhere — all DI via Extension
- **Frontend query keys**: feature hooks use per-module `queryKeys.ts` factories; avoid inline `queryKey: [...]` literals in feature API code
- **`shared/components/` is populated** — 9 shared components: ConfirmModal, EmptyState, ErrorBoundary, FileUploader, LoadingSpin, PageContainer, PageHeader, SearchBar, StatusTag
- **`docs/AGENTS.md`** exists as index for design docs in Chinese
- **Seed data**: `backend/seed_data.py` and `backend/seed_data_enhanced.py` available
- **Terminology**: use only the terms from `specs/UBIQUITOUS_LANGUAGE_LATEST.md` (商品/Item+SKU, 采购订单, 销售订单, 质检/Inspection, 工单, etc.)
- **New i18n namespaces**: items, inventory, profile, purchase, sales, search, system, validation, workflow, hr, finance, procurement, manufacturing, projects, assets, notifications, portal, reports, contracts, customers, suppliers (zh + en each)
