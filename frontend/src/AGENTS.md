# `frontend/src/` — App Structure & Shared Infrastructure

How the app is wired up, what the shared pieces do, and how to add new stuff. Frontend of the **ERP（通用企业资源计划系统）** monorepo — backend crate `erp-server`, backend data layer **SQLite3** (`sqlite://data/erp.db?mode=rwc`, sqlx 0.8 sqlite feature).

## Entry Points

### `main.tsx`

```tsx
import './i18n'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from 'react-router-dom'
import { router } from './routes'

// Create QueryClient with defaults
// Render: <QueryClientProvider> → <RouterProvider router={router} />
```

- Imports i18n as a side-effect before anything else renders.
- Creates the QueryClient with default staleTime/gcTime.
- Renders the app into `#root` via RouterProvider.

### `App.tsx`

```tsx
function App() {
  return (
    <ConfigProvider theme={theme}>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </ConfigProvider>
  )
}
```

- Ant Design `ConfigProvider` wraps everything with the custom theme.
- `QueryClientProvider` gives TanStack Query context to the whole tree.
- `RouterProvider` renders whatever route matches from `createBrowserRouter`.

## Shared Infrastructure

### `api/` — Native fetch wrapper

```ts
import api from '@/api/client'  // native fetch wrapper, baseURL '/api/v1'
// Authorization: Bearer token auto-attached from auth store
// On 401: clears auth and redirects to /login
```

- One fetch wrapper (`src/api/client.ts`), used everywhere.
- Auto-attaches `Authorization: Bearer <token>`.
- On 401, clears auth and redirects to `/login`.

### `lib/` — Runtime Validation

- `validateResponse.ts` validates API responses at runtime using `schema.safeParse(data)`.

### `hooks/` — Shared Hooks

- `useAuth.ts` — Login/logout/current user.
- `usePagination.ts` — Pagination state management.

### `stores/` — Zustand State Management

- `authStore.ts` — Token, user, login/logout.
- `appStore.ts` — Global UI state (sidebar collapsed, theme).

### `i18n/` — Translations

```
i18n/
├── index.ts        ← i18next init
├── zh/             ← Chinese translations (per-feature namespaces)
│   ├── common.json
│   ├── auth.json
│   ├── inventory.json
│   ├── inbound.json
│   ├── outbound.json
│   ├── stock.json
│   ├── location.json
│   ├── inventory_check.json
│   ├── purchase.json
│   ├── sales.json
│   ├── contracts.json
│   ├── suppliers.json
│   ├── customers.json
│   ├── manufacturing.json
│   ├── project.json
│   ├── assets.json
│   ├── hr.json
│   ├── finance.json
│   ├── procurement.json
│   ├── workflow.json
│   ├── notification.json
│   ├── portal.json
│   ├── reports.json
│   ├── bi.json
│   ├── search.json
│   ├── profile.json
│   ├── data_io.json
│   ├── system.json
│   └── validation.json
└── en/             ← English translations (same structure)
```

- Namespaces mirrored in zh/ and en/.
- Namespace per feature: `'common'`, `'inventory'`, `'purchase'`, `'sales'`, etc.
- Components use `useTranslation('feature_name')`.

### `routes/` — Route Config (react-router-dom v7)

```
/login                     ← public
/                          ← ProtectedRoute → MainLayout → Outlet
  /inventory/inbound       ← InboundListPage
  /inventory/inbound/new   ← InboundFormPage
  /inventory/outbound      ← OutboundListPage
  /inventory/outbound/new  ← OutboundFormPage
  /inventory/stock         ← StockQueryPage
  /inventory/locations     ← LocationListPage
  /inventory/check         ← InventoryCheckListPage
  /inventory/logs          ← InventoryLogsPage
  /inventory/atp           ← InventoryAtpPage
  /suppliers               ← SupplierListPage (+ /new, /:id/edit)
  /customers               ← CustomerListPage (+ /new, /:id/edit)
  /purchases               ← (+ /new, /:id, /:id/edit)
  /sales                   ← (+ /new, /:id, /:id/edit)
  /sales/atp               ← AtpPage
  /sales/crm               ← SalesCrmPage
  /contracts               ← (+ /new, /:id, /:id/edit)
  /manufacturing           ← ManufacturingPage
  /projects                ← ProjectPage
  /assets                  ← AssetsPage
  /hr/employees            ← EmployeeListPage
  /hr/salaries             ← SalaryPage
  /finance                 ← FinancePage
  /procurement             ← ProcurementPage
  /workflow/definitions    ← WorkflowListPage
  /workflow/my-tasks       ← MyTasksPage
  /notifications           ← NotificationsPage
  /portal                  ← PortalAdminPage
  /reports                 ← ReportListPage
  /reports/dashboard       ← DashboardPage
  /reports/inventory       ← InventoryReportPage
  /reports/orders          ← OrderReportPage
  /reports/trends          ← TrendsPage
  /bi                      ← BiDashboardPage
  /data-io/import          ← DataImportPage
  /data-io/export          ← DataExportPage
  /data-io/logs            ← OperationLogPage
  /system/users            ← UserManagementPage
  /system/roles            ← RoleManagementPage
  /system/departments      ← DepartmentPage
  /profile/settings        ← ProfileSettingsPage
  /search                  ← SearchPage
```

- Uses `createBrowserRouter` (not a flat route array).
- `ProtectedRoute` checks auth before `MainLayout` renders.
- Nested layouts via `Outlet`.
- No lazy loading yet — all pages are eagerly loaded.

### `shared/` — Shared Components & Utilities

- `components/` — 13 reusable UI components:
  - `ActionButton` — Standard action button with loading state
  - `Can` — Permission-based render guard
  - `DataTable` — Generic Ant Design table wrapper
  - `EmptyState` — Placeholder for empty lists
  - `ErrorBoundary` — Catches render errors with fallback UI
  - `FormField` — Form field wrapper
  - `ItemPicker` — Item/SKU selector
  - `ListPageTemplate` — Standard list page scaffolding
  - `PageLayout` — Standard page wrapper
  - `RouteBoundary` — Route-level error boundary
  - `SearchBar` — Debounced search input
  - `StatusBadge` — Colored status badge
  - `StatusTag` — Colored status tag
- `hooks/` — Shared hooks:
  - `useDebounce` — Debounce a changing value

### `theme/` — Ant Design Theme

```ts
const theme: ThemeConfig = {
  token: {
    colorPrimary: '#1677ff',
    borderRadius: 6,
    // Ant Design 5 theme tokens
  }
}
```

- Consistent brand colors and spacing.
- CSS overrides via Less variables in `vite.config.ts`.

### `zod-schemas/` — Zod Validation Schemas

```
zod-schemas/
├── core.ts        ← Common wrappers (PaginatedResponse, ApiResponse)
├── orders.ts      ← 采购订单/销售订单 schemas
├── inventory.ts   ← 商品/SKU, 入库, 出库
├── reports.ts     ← Report parameter schemas
├── search.ts      ← Search query schemas
```

- Each file exports Zod types for request/response validation.
- Used by `lib/validateResponse.ts` for runtime checking.
- Complements TypeScript types with actual runtime enforcement.

### `utils/` — Utility Functions

- `formatters.ts` — Date, currency, decimal formatting.
- `validators.ts` — Legacy form validation helpers.
- `constants.ts` — API paths, status enums.
- Primary validation schemas live in `zod-schemas/`, not here.

## How to Add a New Feature Page

1. Create feature files in `src/features/{feature}/` (see features/AGENTS.md for the pattern).
2. Add the route in `src/routes/index.tsx`.
3. Add i18n namespace files in `src/i18n/zh/{feature}.json` and `src/i18n/en/{feature}.json`.
4. Import the shared `api` wrapper from `src/api/client.ts` for data fetching.
5. If the feature needs client-side state, create a Zustand store in `src/stores/`.
