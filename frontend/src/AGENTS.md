# `frontend/src/` — App Structure & Shared Infrastructure

## Entry Points

### `main.tsx`
```tsx
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter } from 'react-router-dom'

// Init i18n
// Create QueryClient
// Render: <QueryClientProvider> → <BrowserRouter> → <App />
```
- Initializes i18next (detect language, load resources)
- Creates QueryClient with default staleTime
- Renders React app into `#root`

### `App.tsx`
```tsx
function App() {
  return (
    <ConfigProvider theme={theme}>
      <AppLayout>
        <AuthGuard>
          <AppRoutes />
        </AuthGuard>
      </AppLayout>
    </ConfigProvider>
  )
}
```
- Wraps app in Ant Design `ConfigProvider` with custom theme
- `AppLayout` — sidebar + header + content area
- `AuthGuard` — checks JWT token, redirects to login if expired
- `AppRoutes` — renders the matched route from `routes/`

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

### `components/` — Shared Components
- `AppLayout.tsx` — Sidebar + Header + Content shell
- `Sidebar.tsx` — Navigation menu
- `PrivateRoute.tsx` — Auth guard wrapper
- `Loading.tsx` — Loading spinner
- `ErrorBoundary.tsx` — Error fallback

### `hooks/` — Shared Hooks
- `useAuth.ts` — Auth context (login/logout/current user)
- `usePagination.ts` — Pagination state management

### `i18n/` — Translations
```
i18n/
├── index.ts        ← i18next init
├── zh/             ← Chinese translations
│   ├── common.json
│   ├── pipes.json
│   ├── inventory.json
│   └── ...
└── en/             ← English translations (same structure)
```
- Namespace per feature: `'common'`, `'pipes'`, `'inventory'`, etc.
- Use `useTranslation('feature_name')` in components

### `routes/` — Route Definitions
```tsx
const routes = [
  { path: '/', element: <Navigate to="/pipes" /> },
  { path: '/pipes', element: <PipeListPage /> },
  { path: '/pipes/create', element: <PipeCreatePage /> },
  { path: '/pipes/:id', element: <PipeDetailPage /> },
  // ... per feature
]
```
- Flat route structure (no nested routing)
- Each feature registers its routes here
- Lazy loading via `React.lazy()` on large pages

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

### `utils/` — Utility Functions
- `formatters.ts` — Date, currency, decimal formatting
- `validators.ts` — Zod schemas for form validation
- `constants.ts` — API endpoint paths, status enums

## How to Add a New Feature Page
1. Create feature files in `src/features/{feature}/` (see features/AGENTS.md)
2. Add route in `src/routes/index.tsx`
3. Add i18n namespace in `src/i18n/zh/{feature}.json` and `src/i18n/en/{feature}.json`
4. Import api instance from `src/api/` for data fetching
