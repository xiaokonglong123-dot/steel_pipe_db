# Steel Pipe DB — Project Index

## Quick Start

```
# Backend (Rust Axum on :3000)
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

## Build & Verify

| What | How | CI checks |
|------|-----|-----------|
| Backend type-check | `cd backend && cargo check` | `cargo check` |
| Backend tests | `cd backend && cargo test` | — |
| Frontend type-check | `cd frontend && npx tsc --noEmit` | `tsc --noEmit` |
| Frontend build | `cd frontend && npm run build` | `npm run build` |
| Frontend chunk analysis | `cd frontend && npx vite build --analyze` (manualChunks via vite.config.ts) | — |
| Full CI pipeline | `cargo check` + `tsc --noEmit` + `npm run build` (parallel) | `.github/workflows/ci.yml` |

**Heads up**: There's **no Makefile** despite what the README says. Just use cargo/npm directly.

## Architecture

```
steel-pipe-db/
├── backend/          ← Rust Axum 0.8 REST API (SQLite, JWT/Argon2)
│   └── src/
│       ├── main.rs         ← Entry: tracing, DB pool, migrate, start server
│       ├── lib.rs          ← Module declarations re-exported
│       ├── router.rs       ← ~70 endpoints, all routes assembled here
│       ├── handlers/       ← 13 files, 1 per entity (thin: extract → call service → respond)
│       ├── services/       ← 12 files, business logic (unit structs, static methods)
│       ├── repositories/   ← 13 files, pure SQL, soft-delete aware
│       ├── models/         ← 11 files, DB row structs (sqlx::FromRow)
│       ├── dto/            ← 14 files, request/response types
│       ├── domain/         ← 4 files, enums/domain types
│       ├── middleware/     ← auth.rs + rbac.rs (updated with success/request_id in error responses)
│       ├── config.rs       ← Env-based config (DATABASE_URL, JWT_SECRET, etc.)
│       ├── error.rs        ← AppError enum, numeric error codes; ApiErrorResponse with success+request_id
│       └── response.rs     ← ApiResponse<T>, PaginatedResponse<T>, Meta struct, request_id (uuid v4), ::created(), no_content()
├── frontend/         ← React 19 + Vite + Ant Design + TanStack Query
│   └── src/
│       ├── main.tsx        ← React DOM entry
│       ├── App.tsx         ← ConfigProvider + QueryClientProvider + RouterProvider
│       ├── api/            ← Axios instance + QueryClient config
│       ├── routes/         ← createBrowserRouter + ProtectedRoute
│       ├── features/       ← 13 feature modules (auth, contracts, customers, search, profile, ...)
│       ├── layouts/        ← MainLayout (sidebar + header + Outlet)
│       ├── stores/         ← Zustand authStore, appStore (global state), unitStore (unit conversion)
│       ├── lib/            ← validateResponse.ts, runtime zod response validation
│       ├── styles/         ← Ant Design theme config
│       ├── zod-schemas/    ← 7 Zod schema files for response validation
│       ├── shared/         ← hooks (useDebounce), components/ (9 shared components), utils/
│       └── i18n/           ← react-i18next (zh-CN primary)
└── docs/             ← PRD, design docs, task breakdown
```

## Tech Stack (verified from Cargo.toml / package.json)

### Backend
- **Rust** edition 2021, nightly 2024-02-08
- **Axum 0.8** with macros + multipart features
- **SQLx 0.8** with SQLite, runtime-tokio-rustls, chrono
- **Auth**: jsonwebtoken 9 + argon2 0.5 (NOT bcrypt)
- **Validation**: validator 0.19 with derive
- **Tracing**: tracing + tracing-subscriber with env-filter + json
- **tower-http 0.6**: CORS, trace, request-id
- **Import/Export**: calamine 0.26 (Excel), rust_xlsxwriter 0.80, csv 1.3
- **No `rust_decimal` or `bigdecimal`** — decimals handled via f64 in current code
- **No `build.rs`** — despite being mentioned in subordinate AGENTS.md

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
pub async fn list_seamless_pipes_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<PipeFilterParams>,
) -> Result<Json<PaginatedResponse<SeamlessPipe>>, AppError> {
    let (items, total) = PipeService::list_seamless_pipes(&pool, &filter, &pagination).await?;
    Ok(PaginatedResponse::ok(items, total, page, page_size))
}
```
Handlers return `Result<Json<...>, AppError>` (NOT `impl IntoResponse`). Errors propagate via `?`.

### Service Pattern: Unit struct + static methods
```rust
pub struct PipeService;  // No fields, no constructor, no DI

impl PipeService {
    pub async fn create_seamless_pipe(pool: &SqlitePool, dto: &CreateSeamlessPipeRequest) -> Result<SeamlessPipe, AppError> {
        // Business logic here
    }
}
```
Services are **unit structs with static methods**, taking `pool: &SqlitePool` directly. Forget the fancy constructor DI pattern you read about in some blog — this is what we actually do.

### Repository Pattern
```rust
SeamlessPipeRepo::find_by_pipe_number(pool, pn).await
```
Same deal — static methods, `pool: &SqlitePool`. Soft-delete: `WHERE deleted_at IS NULL`.

### Error Codes (numeric, domain-prefixed)
| Range | Domain |
|-------|--------|
| 100xx | General (Internal, Validation, NotFound) |
| 110xx | Auth (Unauthorized, TokenExpired, Forbidden) |
| 120xx | Pipe (NotFound, Duplicate, StatusConflict) |
| 130xx | Inventory (InsufficientStock, LocationNotFound) |
| 140xx | Orders (CannotModify, NotFound) |
| 150xx | Quality (CertNotFound, AttachmentNotFound) |
| 160xx | Supplier (NotFound, CodeDuplicate) |
| 170xx | Customer (NotFound, CodeDuplicate) |
| 180xx | Data IO (ImportError, ExportError) |
| 50001 | Database |

### Handler Files (13)
`auth_handler`, `pipe_handler`, `inventory_handler`, `purchase_handler`, `sales_handler`, `quality_handler`, `contract_handler`, `customer_handler`, `supplier_handler`, `report_handler`, `label_handler`, `data_io_handler`, `atp_handler`

### Service Files (12)
`auth_service`, `pipe_service`, `inventory_service`, `purchase_sales_service`, `quality_service`, `contract_service`, `customer_service`, `supplier_service`, `label_service`, `report_service`, `data_io_service`, `trace_service`

### Repository Files (13)
`pipe_repo`, `inventory_repo`, `purchase_order_repo`, `sales_order_repo`, `quality_repo`, `contract_repo`, `customer_repo`, `supplier_repo`, `label_repo`, `report_repo`, `data_io_repo`, `user_repo`, `operation_log_repo`

### DB Migrations (11 files in `backend/migrations/`)
`001_create_users` → `002_create_seamless_pipes` → `003_create_screen_pipes` → `004_create_locations` → `005_create_inventory` → `006_create_orders` → `007_create_quality` → `008_create_logs` → `009_create_ref_data` → `010_seed_api_5ct_data` → `011_add_rejection_reason`

## Frontend Patterns

### Routing (react-router-dom v7, createBrowserRouter + RouterProvider)
```
/login                     ← public
/                          ← ProtectedRoute → MainLayout → Outlet
  /pipes/seamless          ← SeamlessPipeListPage
  /pipes/seamless/new      ← SeamlessPipeFormPage
  /pipes/seamless/:id      ← SeamlessPipeDetailPage
  /pipes/seamless/:id/edit ← SeamlessPipeFormPage
  /pipes/screen/*          ← same pattern
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
  /quality/certs           ← (+ /new, /:id, /:id/edit)
  /contracts               ← (+ /new, /:id, /:id/edit)
  /reports                 ← ReportListPage
  /reports/dashboard       ← DashboardPage
  /labels                  ← LabelPrintPage
  /profile/settings        ← ProfileSettingsPage
  /search                  ← SearchPage
```

### Feature Modules (13)
`auth`, `pipes`, `inventory`, `suppliers`, `customers`, `purchases`, `sales`, `quality`, `contracts`, `reports`, `labels`, `search`, `profile`

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
- **New routes**: InboundFormPage, OutboundFormPage, ProfileSettingsPage, SearchPage added
- **New i18n namespaces**: inventory, pipes, profile, purchase, quality, sales, search, system, validation (zh + en each)
