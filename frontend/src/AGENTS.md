# `frontend/src/` — App Structure & Shared Infrastructure

How the app is wired up, what the shared pieces do, and how to add new stuff.

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

### `api/` — Axios Instance

```ts
const api = axios.create({ baseURL: '/api/v1' })
// Interceptor: attach JWT token from localStorage
// Interceptor: handle 401 → redirect to login
```

- One axios instance, used everywhere.
- Auto-attaches `Authorization: Bearer <token>`.
- On 401, clears auth and redirects to `/login`.

### `lib/` — Runtime Validation

- `validateResponse.ts` wraps Zod schemas to validate API responses at runtime.
- Uses a `zod.response()` pattern — imported by feature API modules for type-safe data fetching.

### `hooks/` — Shared Hooks

- `useAuth.ts` — Login/logout/current user.
- `usePagination.ts` — Pagination state management.

### `stores/` — Zustand State Management

- `authStore.ts` — Token, user, login/logout.
- `appStore.ts` — Global UI state (sidebar collapsed, theme).
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

- 15 namespaces mirrored in zh/ and en/.
- Namespace per feature: `'common'`, `'pipes'`, `'inventory'`, etc.
- Components use `useTranslation('feature_name')`.

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

- Uses `createBrowserRouter` (not a flat route array).
- `ProtectedRoute` checks auth before `MainLayout` renders.
- Nested layouts via `Outlet`.
- No lazy loading yet — all pages are eagerly loaded.

### `shared/` — Shared Components & Utilities

- `components/` — 9 reusable UI components:
  - `ConfirmModal` — Confirmation dialog with custom content
  - `EmptyState` — Placeholder for empty lists
  - `ErrorBoundary` — Catches render errors with fallback UI
  - `FileUploader` — Drag-and-drop file upload
  - `LoadingSpin` — Centered spinner
  - `PageContainer` — Standard page wrapper
  - `PageHeader` — Title + breadcrumb + action buttons
  - `SearchBar` — Debounced search input
  - `StatusTag` — Colored status badge
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
├── orders.ts      ← Purchase/Sales order schemas
├── inventory.ts   ← Inventory, inbound, outbound
├── quality.ts     ← Quality certificate schemas
├── reports.ts     ← Report parameter schemas
├── labels.ts      ← Label data schemas
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
4. Import the shared `api` instance from `src/api/` for data fetching.
5. If the feature needs client-side state, create a Zustand store in `src/stores/`.
