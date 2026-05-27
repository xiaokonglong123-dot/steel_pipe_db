# `frontend/src/` — App Structure & Shared Infrastructure

How everything's wired up: entry points, shared modules, and the recipe for adding new features.

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

- Side-effect imports i18n before anything renders.
- Creates QueryClient with default staleTime/gcTime.
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

- `ConfigProvider` applies the Ant Design theme globally.
- `QueryClientProvider` provides TanStack Query to the whole tree.
- `RouterProvider` picks the right route from `createBrowserRouter`.

## Shared Infrastructure

### `api/` — Axios Instance

```ts
const api = axios.create({ baseURL: '/api/v1' })
// Interceptor: attach JWT token from localStorage
// Interceptor: handle 401 → redirect to login
```

- Single axios instance shared across all features.
- Auto-attaches `Authorization: Bearer <token>`.
- On 401, clears auth state and redirects to `/login`.

### `lib/` — Runtime Validation

- `validateResponse.ts` wraps Zod schemas for runtime API response validation.
- Uses a `zod.response()` pattern — feature API modules import this for type-safe fetching.

### `hooks/` — Shared Hooks

- `useAuth.ts` — Login/logout/current user tracking.
- `usePagination.ts` — Pagination controls.

### `stores/` — Zustand State Management

- `authStore.ts` — Token, user info, login/logout actions.
- `appStore.ts` — Global UI state (sidebar collapse, theme).
- `unitStore.ts` — Metric/imperial toggle.

### `i18n/` — Translations

```
i18n/
├── index.ts        ← i18next init
├── zh/             ← Chinese translations (15 namespaces)
│   ├── common.json
│   ├── pipes.json
│   ├── inventory.json
│   ├── purchase.json
│   ├── sales.json
│   ├── quality.json
│   ├── contracts.json
│   ├── suppliers.json
│   ├── customers.json
│   ├── reports.json
│   ├── labels.json
│   ├── profile.json
│   ├── search.json
│   ├── system.json
│   └── validation.json
└── en/             ← English translations (same structure)
```

- 15 namespaces, mirrored in zh/ and en/.
- Namespace per feature: `'common'`, `'pipes'`, `'inventory'`, etc.
- Use `useTranslation('feature_name')` in components.

### `routes/` — Route Config (react-router-dom v7)

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

- Uses `createBrowserRouter` (not a flat array).
- `ProtectedRoute` gates access behind auth check.
- `Outlet` for nested layouts.
- No lazy loading yet — everything's eagerly loaded.

### `shared/` — Shared Components & Utilities

- `components/` — 9 reusable UI components:
  - `ConfirmModal` — Custom confirmation dialog
  - `EmptyState` — Empty list placeholder
  - `ErrorBoundary` — Catches render errors with a fallback
  - `FileUploader` — Drag-and-drop file upload
  - `LoadingSpin` — Centered spinner
  - `PageContainer` — Standard page layout wrapper
  - `PageHeader` — Title + breadcrumb + action buttons
  - `SearchBar` — Debounced search input
  - `StatusTag` — Colored status badge
- `hooks/` — Shared hooks:
  - `useDebounce` — Debounce a value

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
├── core.ts        ← Common types (PaginatedResponse, ApiResponse)
├── orders.ts      ← Purchase/Sales order schemas
├── inventory.ts   ← Inventory, inbound, outbound
├── quality.ts     ← Quality certificate schemas
├── reports.ts     ← Report parameter schemas
├── labels.ts      ← Label data schemas
```

- Each file exports Zod types for request/response validation.
- Used by `lib/validateResponse.ts` for runtime checking.
- Complements TypeScript static types with actual runtime enforcement.

### `utils/` — Utility Functions

- `formatters.ts` — Date, currency, decimal formatting.
- `validators.ts` — Legacy form validation helpers.
- `constants.ts` — API paths, status enums.
- Primary validation lives in `zod-schemas/`.

## How to Add a New Feature Page

1. Create the feature module in `src/features/{feature}/` (see features/AGENTS.md).
2. Add its route in `src/routes/index.tsx`.
3. Add i18n files: `src/i18n/zh/{feature}.json` and `src/i18n/en/{feature}.json`.
4. Import the shared `api` instance from `src/api/` for data fetching.
5. If you need client-side state, add a Zustand store in `src/stores/`.
