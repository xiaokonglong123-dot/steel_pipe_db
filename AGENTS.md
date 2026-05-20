# Seamless Pipe & Screen Pipe Management System

> Oil & gas pipe inventory management — Rust + React, full lifecycle from procurement to sales.

## Quick Start

```bash
# Backend (port 8080)
cd backend && cargo run

# Frontend (port 3000, proxies /api → :8080)
cd frontend && npm run dev
```

Default admin: `admin` / `admin123` (seeded on first run).

## Build & Check Commands

| Command | Layer | When |
|---------|-------|------|
| `cargo check` | backend | After any Rust change |
| `cargo build` | backend | Before run |
| `cargo run` | backend | Dev server |
| `npx tsc --noEmit` | frontend | After any TS change |
| `npm run build` | frontend | `tsc -b && vite build` |
| `npm run dev` | frontend | Vite dev server |

## Project State (source code exists, not design-only)

**~10,700 lines Rust** | **~5,000 lines TypeScript** | All 3 phases covered.

### Backend (Rust + Axum 0.8 + SQLx 0.8 + SQLite WAL)

4-layer: `Handler → Service → Repository → Domain`

```
handlers/       # Route handlers (param validation, JSON response)
services/       # Business logic, transaction orchestration
repositories/   # SQLx queries, data mapping
domain/         # Structs, enums (all in common.rs)
```

Modules: auth, pipes (seamless+screen), inventory (inbound/outbound/check), **purchases** (suppliers, purchase-orders, contracts), **sales** (customers, sales-orders, atp), quality (certs, api5ct refs), data-io (import/export), reports, labels, system/auth.

API root: `/api/v1/` — JWT Bearer auth. Error format: `{ success: false, error: { code, message } }`.

### Frontend (React 19 + Vite 6 + Ant Design 5 + TanStack Query 5 + Zustand 5)

Feature-sliced: `features/{name}/{pages/, components/, api/, hooks/}`

Shared: `shared/components`, `shared/types/`, `shared/utils/`

i18n: react-i18next, zh+en via `i18n/resources/`. Currently only `common` namespace loaded — per-module namespaces expected by route config but not yet created.

`staleTime: 120s` (TanStack Query). `@/` → `src/` path alias.

## Known Issues

### Orders→Purchases+Sales Split ✅

Complete. Backend `cargo check` passes cleanly, frontend `tsc --noEmit` has only pre-existing errors (below).

- Backend: 4 modules fully split (handler/service/repo+domain), old dead files deleted
- Frontend: `features/purchases/` and `features/sales/` fully wired in `App.tsx`, old `features/orders/` deleted

### Pre-existing TS errors (not from split)
- `features/data-io/pages/ImportPage.tsx` — `previewSeamlessPipes`/`previewScreenPipes` missing from api type
- `features/inventory/pages/InboundFormPage.tsx` — `listAllPipes` missing from api type
- `features/inventory/pages/OutboundFormPage.tsx` — `listAllPipes` missing from api type

### Other gaps
- `mocks/` directory exists but is empty (MSW planned, not started)
- `shared/components/` exists but is empty
- No tests (empty `backend/tests/`)
- `report_service.rs` is an empty stub (0 bytes logic)

## Key Design Conventions

### Database (SQLite WAL)
- No foreign key constraints (app-level integrity)
- Soft deletes via `deleted_at TEXT`
- All timestamps as ISO 8601 TEXT (`datetime('now')`)
- Enums stored as TEXT, not INTEGER
- PRAGMAs: `busy_timeout=5000`, `cache_size=-64000`, `temp_store=MEMORY`, `foreign_keys=OFF`

### API
- `sort_by` has an allowlist (no arbitrary SQL injection)
- No PATCH endpoints — only PUT for full updates
- Response: `{ success, data, meta?, request_id }` | `{ success: false, error: { code, message } }`

### Frontend
- No custom fonts (system font stack)
- Industrial blue theme (#1B3A5C primary, #0F1A2E sidebar)
- Zustand is pure state — no API calls in stores
- Auth token refresh via Axios interceptor with request queue

### Code style
- `#![allow(dead_code)]` at top of `main.rs` (intentional)
- `noUnusedLocals: false`, `noUnusedParameters: false` in tsconfig
- Pipe number format follows API 5CT (e.g. `J55 4.500in×11.60lb SC-H2405-000001`)

## Roles (RBAC)

| Role | Code | Main modules |
|------|------|-------------|
| Admin | `admin` | Full access |
| Warehouse | `warehouse` | Pipes, inventory, quality |
| QC | `qc` | Pipe view, quality management |
| Sales | `sales` | Inventory view, orders, customers/suppliers |

## Phase Structure

| Phase | Modules | Status |
|-------|---------|--------|
| P0 (MVP) | Pipes, inventory, auth, tracing | ✅ Built (compiles) |
| P1 (Business) | Quality, purchases, sales, data-io | ✅ Built (split incomplete) |
| P2 (Enhancement) | Contracts, reports, labels, i18n | ✅ Built |
