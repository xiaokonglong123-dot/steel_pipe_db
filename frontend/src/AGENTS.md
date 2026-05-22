# `frontend/src/` — App Structure & Shared Infrastructure

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
- Imports i18n (side-effect import) before rendering
- Creates QueryClient with default staleTime
- Renders React app into `#root` with RouterProvider

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
- Wraps app in Ant Design `ConfigProvider` with custom theme
- `QueryClientProvider` provides TanStack Query context
- `RouterProvider` renders routes from `createBrowserRouter`

## Shared Infrastructure

### `api/` — Axios Instance
```ts
const api = axios.create({ baseURL: '/api/v1' })
// Interceptor: attach JWT token from localStorage
// Interceptor: handle 401 → redirect to login
```
- One axios instance for all API calls
- Attaches `Authorization: Bearer <token>` header
- Automatically redirects on 401

### `lib/` — Runtime Validation
- `validateResponse.ts` — wraps Zod schemas for API response validation
- Uses `zod.response()` pattern: validates API responses at runtime
- Imported by feature API modules for type-safe data fetching

### `hooks/` — Shared Hooks
- `useAuth.ts` — Auth context (login/logout/current user)
- `usePagination.ts` — Pagination state management

### `stores/` — Zustand State Management
- `authStore.ts` — Authentication state (token, user, login/logout)
- `appStore.ts` — Global app state (sidebar collapsed, theme, etc.)
- `unitStore.ts` — Unit conversion state (metric/imperial toggle)

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
- 15 namespaces across zh/ and en/
- Namespace per feature: `'common'`, `'pipes'`, `'inventory'`, etc.
- Use `useTranslation('feature_name')` in components

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
- Uses `createBrowserRouter` (not flat route array)
- `ProtectedRoute` wrapper checks auth before rendering `MainLayout`
- `Outlet` pattern for nested layouts
- No lazy loading currently (all pages eagerly loaded)

### `shared/` — Shared Components & Utilities
- `components/` — 9 reusable UI components:
  - `ConfirmModal` — Confirmation dialog with customizable content
  - `EmptyState` — Empty state placeholder with icon and message
  - `ErrorBoundary` — React error boundary with fallback UI
  - `FileUploader` — File upload with drag-and-drop
  - `LoadingSpin` — Centered loading spinner
  - `PageContainer` — Standard page layout wrapper
  - `PageHeader` — Page title with breadcrumb and actions
  - `SearchBar` — Search input with debounce
  - `StatusTag` — Colored status tag badge
- `hooks/` — Shared hooks:
  - `useDebounce` — Debounce value changes

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
- Consistent branding colors and spacing
- Override Ant Design CSS via Less variables in `vite.config.ts`

### `zod-schemas/` — Zod Validation Schemas
```
zod-schemas/
├── core.ts        ← Common types (PaginatedResponse, ApiResponse wrapper)
├── orders.ts      ← Purchase/Sales order schemas
├── inventory.ts   ← Inventory, inbound, outbound schemas
├── quality.ts     ← Quality certificate schemas
├── reports.ts     ← Report parameter schemas
├── labels.ts      ← Label data schemas
```
- Each schema file exports Zod types for API request/response validation
- Used by `lib/validateResponse.ts` for runtime API response checking
- Complements TypeScript static types with runtime validation

### `utils/` — Utility Functions
- `formatters.ts` — Date, currency, decimal formatting
- `validators.ts` — Legacy form validation helpers (may reference zod-schemas/)
- Primary validation schemas live in `zod-schemas/`
- `constants.ts` — API endpoint paths, status enums

## How to Add a New Feature Page
1. Create feature files in `src/features/{feature}/` (see features/AGENTS.md)
2. Add route in `src/routes/index.tsx`
3. Add i18n namespace in `src/i18n/zh/{feature}.json` and `src/i18n/en/{feature}.json`
4. Import api instance from `src/api/` for data fetching
5. Create Zustand store in `src/stores/` if the feature needs client-side state
