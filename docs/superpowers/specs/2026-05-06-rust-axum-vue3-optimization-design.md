# Steel Pipe Inventory Management System — Optimization & Extension Design

## Overview

A comprehensive optimization and extension plan for the `rust-axum-vue3` sub-project, covering architecture refactoring, permission system, frontend stack upgrade, feature enhancements, UI/UX improvements, and performance tuning.

## Strategy

**Parallel dual-track development**: Backend (Rust/Axum) and frontend (Vue 3) developed in sync, sharing an API contract for compatibility. Delivered in three iterative phases.

## Backend Design

### Module Splitting

Split `db.rs` (1079 lines) into domain-specific modules:

```
server/src/
├── main.rs              # Route definitions + startup
├── db/
│   ├── mod.rs           # DB init, connection management, migrations
│   ├── pipes.rs         # Pipe CRUD queries
│   ├── records.rs       # Inbound/outbound records
│   ├── productions.rs   # Production input
│   ├── logs.rs          # Operation logs
│   └── reports.rs       # Report/statistics queries
├── handlers/
│   ├── mod.rs        # Route→handler mappings (re-exports all handlers)
│   ├── pipes.rs      # Pipe CRUD handlers
│   ├── records.rs    # Inbound/outbound record handlers
│   ├── productions.rs# Production input handlers
│   ├── auth.rs       # Auth handlers
│   ├── export.rs     # Export handlers
│   ├── import.rs     # Import handlers
│   └── system.rs     # Backup/restore + report handlers
├── models.rs            # Data models + validation
├── auth.rs              # JWT auth middleware + role checks
├── error.rs             # AppError enum + IntoResponse
└── types.rs             # Shared types: pagination, sorting, etc.
```

Splitting rule: each module exposes `pub fn query_(…) -> Result<…>`, uses `rusqlite::Connection` internally, with the parent `mod.rs` managing transactions.

### Auth & Authorization

- **JWT**: `jsonwebtoken` crate, HS256 signing, 24h expiry
- **Passwords**: `bcrypt` crate for hashing
- **Three roles**:
  | Role | Permissions |
  |------|-------------|
  | `admin` | Everything: CRUD, import/export, backup/restore, user management |
  | `operator` | Daily ops: inbound/outbound/production input, view data |
  | `viewer` | Read-only: dashboard, inventory, records, reports |
- **Middleware**: Axum `FromRequestParts` implementation for `RequireAuth`, extracts token and injects user info into request extensions
- **Route protection**: `axum::middleware::from_fn` or layered `Router` in `main.rs`

### New Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/auth/login` | Login, returns JWT + user info |
| POST | `/api/auth/register` | Register (admin only) |
| GET  | `/api/auth/me` | Get current user info |
| PUT  | `/api/auth/password` | Change password |

### Performance Optimizations

- **Connection pool**: `r2d2 + r2d2_sqlite`, pool size 4-8
- **Query optimization**: Add composite indexes on critical queries (dashboard stats, inventory lists, record filtering) — `(status, material)`, `(material, entry_date)`, `(operation_type, operation_date)`. Use `EXPLAIN QUERY PLAN` to verify index usage
- **Dictionary caching**: Cache material/location lookups in memory with `once_cell` or `dashmap`, refresh periodically

### Testing

- Unit tests: `models.rs` validation logic, `error.rs` error conversion
- Integration tests: `tests/` directory, SQLite in-memory DB for each module
- API tests: Axum `test` utilities for route-level testing

### New Cargo Dependencies

```toml
jsonwebtoken = "9"
bcrypt = "0.15"
r2d2 = "0.8"
r2d2_sqlite = "0.24"
uuid = { version = "1", features = ["v4"] }
```

## Frontend Design

### TypeScript Migration

Migrate all `.js` / `.vue` files to TypeScript:
- Interface type definitions in `src/types/`
- `.vue` files use `<script setup lang="ts">`
- `vite.config.ts` enables TypeScript support

### TypeScript Configuration

- `tsconfig.json`: `strict: true`, `target: ES2022`, `module: ESNext`
- `tsconfig.node.json`: TS support for Vite config files
- `vue-tsc` for type checking, wired into `npm run typecheck`

### Type Definitions

```
types/
├── pipe.ts       # SteelPipe, PipeQuery, PipeFormData
├── auth.ts       # User, UserRole, LoginRequest, LoginResponse
├── record.ts     # InventoryRecord, RecordQuery
├── production.ts # Production, ProductionFormData
├── stats.ts      # DashboardStats, MaterialStats, TrendData
└── common.ts     # PaginatedResponse, ApiError, DictData
```

### Pinia State Management

```
stores/
├── index.ts       # Aggregated exports
├── auth.ts        # user, token, isAuthenticated, login(), logout(), checkAuth()
├── pipes.ts       # pipes[], query, pagination, fetchPipes(), createPipe(), …
├── records.ts     # records[], filters, fetchRecords()
├── production.ts  # productions[], fetchProductions(), createProduction()
└── ui.ts          # theme, sidebarCollapsed, notifications
```

`auth` store key behaviors:
- On init: read token from `localStorage`
- `login()`: call API → store token + user → set Axios default header
- `logout()`: clear all state → router redirect to `/login`
- Axios response interceptor: 401 → auto logout

### Routes & Guards

```
router/
├── index.ts       # Route definitions
└── guard.ts       # beforeEach: auth check + role check
```

Route meta:
```typescript
meta: {
  requiresAuth: boolean
  roles?: UserRole[]
  title: string
}
```

### UI/UX Enhancements

**Dark mode:**
- CSS variables for light/dark color palettes
- `useUIStore` state: `theme: 'light' | 'dark'`
- Listen to `prefers-color-scheme` media query
- Toggle button at sidebar bottom
- Persist to `localStorage`

**Responsive layout:**
- Breakpoints: 768px / 1024px / 1440px
- Sidebar: ≥1024px fixed open, <1024px drawer-style
- Tables: horizontal scroll + responsive column hiding (xsmall hides non-critical columns)
- Forms: grid layout, single column on small screens

**Interaction improvements:**
- Table row hover highlight + click to expand details
- Batch operations: checkbox selection → floating action bar
- Delete confirmations: Modal with two-step confirmation
- Action feedback: global toast notifications
- Loading states: skeleton screens instead of spinners
- Empty states: illustrations + guidance text

**Visualization (ECharts):**
- Dashboard: inventory overview donut chart, 7-day trend line chart, material distribution bar chart
- Stats page: inbound/outbound comparison bar chart, inventory turnover rate

### New Frontend Dependencies

```json
{
  "dependencies": {
    "pinia": "^2.1",
    "echarts": "^5.5",
    "vue-echarts": "^6.6"
  },
  "devDependencies": {
    "typescript": "~5.6",
    "@vitejs/plugin-vue": "^6.0",
    "vue-tsc": "^2.1",
    "@types/node": "^22"
  }
}
```

### Component Directory Restructure

```
components/
├── common/
│   ├── AppButton.vue
│   ├── AppModal.vue
│   ├── AppTable.vue
│   ├── AppForm.vue
│   ├── AppSkeleton.vue
│   └── AppToast.vue
├── layout/
│   ├── AppSidebar.vue
│   ├── AppHeader.vue
│   └── AppLayout.vue
└── business/
    ├── PipeForm.vue
    ├── PipeTable.vue
    ├── RecordTable.vue
    └── ProductionForm.vue
```

## Execution Plan

### Phase 1: Infrastructure + Core New Features

| Backend Track A | Frontend Track B |
|---|---|
| db.rs → db/mod.rs + pipes.rs + records.rs + productions.rs + logs.rs + reports.rs | TS type definitions + Pinia stores setup |
| auth.rs: JWT issue/verify middleware + users table migration | Login page + route guard + auth store |
| handlers.rs splitting | API layer TS-ified + Axios interceptors |
| Main router auth protection layer | Base UI components (Button, Modal, Table) |
| **Deliverable: working login, modular API** | **Deliverable: TS ported frontend, routes protected** |

### Phase 2: Feature Stacking

| Backend Track A | Frontend Track B |
|---|---|
| Rust test coverage (unit + integration) | ECharts visualization (dashboard + stats) |
| Query optimization + composite indexes | Dark mode |
| Connection pool (r2d2) | Responsive layout adaptation |
| Dictionary caching | Skeleton screens + empty states + Toasts |
| **Deliverable: performance bump, feature complete** | **Deliverable: modern UI** |

### Phase 3: Polish & Ship

| Backend Track A | Frontend Track B |
|---|---|
| CI pipeline (Rust check + test) | Interaction polish |
| Performance stress testing | Integration & end-to-end testing |
| Final bug fixes | Final testing |

## Out of Scope

- WebSocket real-time push (future consideration)
- Mobile native apps (future consideration)
- Multi-language i18n (currently Chinese-only requirement)
- Docker containerized deployment (can be added later)
