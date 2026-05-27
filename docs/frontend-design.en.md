# Steel Pipe DB — Frontend Design

> **Doc Version**: v1.2 (updated from the implementation, not from some idealized design doc)
> **Date**: 2026-05-27
> **Stack**: React 19 + Ant Design 5 + TypeScript 5 + Vite 6
> **Status**: Living doc — what's actually running is what matters

---

## Revision History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| v1.0 | 2026-05-19 | Initial version | — |
| v1.1 | 2026-05-19 | Axios 401 → refresh-then-logout; Zustand pure state; removed global loading; removed route loader; system fonts; MSW mock design added; staleTime → 2min | — |
| v1.2 | 2026-05-27 | Rewrote with casual tone. Removed MSW/mock sections (not actually used). Fixed dir structure to match real code. Updated route list from actual `routes/index.tsx`. Removed phantom deps (charts, dnd, qrcode, jsbarcode, xlsx). Stripped `ProLayout` references. | — |

---

## TL;DR

React 19 + Ant Design 5 + TanStack Query + Zustand. Feature-based layout. 13 feature modules, ~35 pages. Chinese-first i18n, 15 namespaces. Axios auto-injects JWT, handles 401 → refresh → retry. No Redux. No route loaders. No fancy patterns — just stuff that works.

---

## Table of Contents

1. [Tech Stack & Dependencies](#1-tech-stack--dependencies)
2. [Project Directory Structure](#2-project-directory-structure)
3. [Route Design](#3-route-design)
4. [Layout Structure](#4-layout-structure)
5. [Component Tree](#5-component-tree)
6. [State Management](#6-state-management)
7. [API Layer](#7-api-layer)
8. [Auth Flow](#8-auth-flow)
9. [Internationalization](#9-internationalization)
10. [Ant Design Theme](#10-ant-design-theme)
11. [Data Flow](#11-data-flow)
12. [Things We Learned The Hard Way](#12-things-we-learned-the-hard-way)

---

## 1. Tech Stack & Dependencies

### 1.1 What We Actually Use

| Layer | Choice | Version | Notes |
|-------|--------|---------|-------|
| **Build** | Vite | 6.x | Fast. ESBuild. No Webpack pain. |
| **UI** | React | 19.x | Latest stable. No class components. |
| **Lang** | TypeScript | 5.x | Strict mode. No `as any`, no `@ts-ignore`. Enforced in CI. |
| **Components** | Ant Design | 5.x | Enterprise-grade. Tables, forms, menus are solid. CSS-in-JS (no less/sass needed for most stuff). |
| **Routing** | React Router | 7.x | `createBrowserRouter`. Nested layouts via `Outlet`. |
| **Server State** | TanStack Query | 5.x | 2min staleTime, 5min gcTime. No manual loading states. |
| **Client State** | Zustand | 5.x | authStore, appStore, unitStore. Tiny, no boilerplate. |
| **HTTP** | Axios | 1.x | Interceptors for token injection + 401 refresh. Nukes the need for manual header management. |
| **i18n** | react-i18next | 15.x | 15 namespaces, zh-CN primary. |
| **Dates** | dayjs | 1.x | Bundled with Ant Design 5. |
| **Validation** | zod | 3.x | Runtime response validation via `validateResponse.ts`. |

### 1.2 Dependencies (from actual `package.json`)

```json
{
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "react-router-dom": "^7.0.0",
    "antd": "^5.20.0",
    "@ant-design/icons": "^5.5.0",
    "@tanstack/react-query": "^5.60.0",
    "zustand": "^5.0.0",
    "axios": "^1.7.0",
    "react-i18next": "^15.0.0",
    "i18next": "^24.0.0",
    "dayjs": "^1.11.0",
    "zod": "^3.23.0"
  },
  "devDependencies": {
    "typescript": "^5.6.0",
    "vite": "^6.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "eslint": "^9.0.0",
    "prettier": "^3.4.0",
    "rollup-plugin-visualizer": "^5.13.1"
  }
}
```

**Not used**: no chart library (dashboard uses Ant Design Statistic + custom CSS), no MSW (we test against real API), no SheetJS/`xlsx` (Excel export is backend-generated), no `@dnd-kit`, no `jsbarcode`, no `qrcode.react`. If you need barcodes, the backend generates PDFs.

---

## 2. Project Directory Structure

### 2.1 Actual Layout

```
frontend/
├── index.html
├── vite.config.ts              # React plugin, proxy to :3000, manualChunks for vendor splitting
├── tsconfig.json               # Strict mode, path alias @/
├── .eslintrc.cjs
├── .prettierrc
├── src/
│   ├── main.tsx                # Entry: i18n side-effect import, StrictMode, render App
│   ├── App.tsx                 # ConfigProvider + QueryClientProvider + RouterProvider
│   ├── vite-env.d.ts
│   │
│   ├── routes/
│   │   ├── index.tsx           # createBrowserRouter config (~40 routes)
│   │   └── ProtectedRoute.tsx  # Auth guard (wraps MainLayout)
│   │
│   ├── layouts/
│   │   └── MainLayout.tsx      # Sider + Header + Outlet
│   │
│   ├── features/               # One directory per business module
│   │   ├── auth/               # LoginPage, UserManagementPage (admin)
│   │   ├── pipes/              # Seamless + Screen pipe CRUD (List/Detail/Form)
│   │   ├── inventory/          # Inbound, Outbound, StockQuery, Locations, Stocktake
│   │   ├── suppliers/          # Supplier list + form
│   │   ├── customers/          # Customer list + form
│   │   ├── purchases/          # PO list/detail/form
│   │   ├── sales/              # SO list/detail/form
│   │   ├── quality/            # Cert list/detail/form
│   │   ├── contracts/          # Contract list/detail/form
│   │   ├── reports/            # ReportListPage, DashboardPage
│   │   ├── labels/             # LabelPrintPage
│   │   ├── search/             # SearchPage (cross-type search)
│   │   └── profile/            # ProfileSettingsPage
│   │
│   ├── shared/                 # Reusable bits
│   │   ├── components/         # 9 components (see below)
│   │   └── hooks/              # useDebounce
│   │
│   ├── api/                    # Axios instance + interceptors
│   ├── lib/                    # validateResponse.ts (zod runtime validation)
│   ├── stores/                 # authStore, appStore, unitStore
│   ├── i18n/                   # 15 namespaces × 2 locales
│   ├── styles/                 # theme.ts (Ant Design 5 tokens)
│   ├── types/                  # Shared TypeScript types
│   ├── zod-schemas/            # Zod schemas for API response validation
│   └── utils/                  # formatters, validators, constants
│
└── dist/                       # Production build output
```

### 2.2 Shared Components (9 of them)

| Component | Path | What It Does |
|-----------|------|-------------|
| `PageHeader` | `shared/components` | Title + breadcrumb + action buttons |
| `PageContainer` | `shared/components` | White card wrapper, unified padding |
| `SearchBar` | `shared/components` | Debounced search input |
| `ConfirmModal` | `shared/components` | Confirm/cancel dialog (delete, approve, etc.) |
| `LoadingSpin` | `shared/components` | Centered spinner (fullscreen or inline) |
| `EmptyState` | `shared/components` | Empty data placeholder (no pipes found) |
| `ErrorBoundary` | `shared/components` | Catches render errors, shows fallback + retry |
| `StatusTag` | `shared/components` | Colored badge by status |
| `FileUploader` | `shared/components` | Drag-and-drop file upload |

### 2.3 Each Feature Module Pattern

```
features/{module}/
├── pages/         # Page components (list, detail, form)
├── api/           # TanStack Query hooks (useXxxList, useXxxMutation, etc.)
└── types/         # Module-specific TS types
```

No separate `components/` or `hooks/` or `stores/` per module — if something's shared, it goes in `shared/`. If it's truly module-internal, keep it in the same file or split into a component file within `pages/`.

---

## 3. Route Design

### 3.1 Route Structure (from `routes/index.tsx`)

```
/login                          ← public, no auth needed
/                               ← ProtectedRoute → MainLayout → Outlet
  /pipes/seamless               ← Default redirect target
  /pipes/seamless/new
  /pipes/seamless/:id
  /pipes/seamless/:id/edit
  /pipes/screen
  /pipes/screen/new
  /pipes/screen/:id
  /pipes/screen/:id/edit
  /inventory/inbound
  /inventory/inbound/new
  /inventory/inbound/:id/edit
  /inventory/outbound
  /inventory/outbound/new
  /inventory/outbound/:id/edit
  /inventory/stock
  /inventory/locations
  /inventory/check
  /suppliers
  /suppliers/new
  /suppliers/:id/edit
  /customers
  /customers/new
  /customers/:id/edit
  /purchases
  /purchases/new
  /purchases/:id
  /purchases/:id/edit
  /sales
  /sales/new
  /sales/:id
  /sales/:id/edit
  /quality/certs
  /quality/certs/new
  /quality/certs/:id
  /quality/certs/:id/edit
  /contracts
  /contracts/new
  /contracts/:id
  /contracts/:id/edit
  /reports
  /reports/dashboard
  /labels
  /system/users
  /search
  /profile/settings
```

### 3.2 Route Config Pattern

```tsx
// routes/index.tsx — straight createBrowserRouter, no magic
export const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    path: '/',
    element: (
      <ProtectedRoute>
        <MainLayout />
      </ProtectedRoute>
    ),
    children: [
      { index: true, element: <Navigate to="/pipes/seamless" replace /> },
      { path: 'pipes/seamless', element: <SeamlessPipeListPage /> },
      { path: 'pipes/seamless/new', element: <SeamlessPipeFormPage /> },
      // ... rest of routes
    ],
  },
]);
```

**No lazy loading yet.** All pages are eagerly imported. With ~35 pages and chunk splitting already configured in `vite.config.ts`, the initial bundle is manageable (~162 kB gzip for app code). If it grows, add `React.lazy()` per route.

**No route loaders.** We don't use React Router loaders/actions for data fetching — all data dependencies are handled inside components via TanStack Query hooks. This keeps routing simple and data concerns where they belong.

### 3.3 Auth Guard

`ProtectedRoute` checks `authStore.isAuthenticated`. If not authenticated → redirect to `/login`. No role check at the route level (that's handled at the UI level via menu filtering + API enforcement).

```tsx
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore(s => s.isAuthenticated);
  if (!isAuthenticated) return <Navigate to="/login" replace />;
  return children;
}
```

---

## 4. Layout Structure

### 4.1 MainLayout (Sider + Header + Content)

Standard Ant Design Pro layout pattern, but we don't use `ProLayout` — it's just `Layout`, `Layout.Sider`, `Layout.Header`, `Layout.Content` with an `Outlet`.

```
┌──────────┬──────────────────────────────────────────┐
│          │  Header                                  │
│  Sider   │  ┌─────────────────────────────────────┐ │
│  (dark)  │  │ Collapse  Search  Lang  User ▼      │ │
│          │  └─────────────────────────────────────┘ │
│  Menu    │  Content (Outlet)                        │
│  ────────│  ┌─────────────────────────────────────┐ │
│  📊 Dash │  │                                     │ │
│  🔩 Pipe │  │  Page content goes here             │ │
│  📦 Inv  │  │                                     │ │
│  ✅ Qual │  │                                     │ │
│  📋 Purch│  └─────────────────────────────────────┘ │
│  💰 Sales│                                          │
│  🤝 Contr│                                          │
│  📊 Rprts│                                          │
│  🏷 Labels│                                         │
│  ⚙ System│                                          │
└──────────┴──────────────────────────────────────────┘
```

### 4.2 Breakpoints

| Breakpoint | Width | Sider Behavior |
|------------|-------|----------------|
| `xxl` | ≥1600px | Expanded, ~220px |
| `xl` | 1200-1599px | Expanded, ~220px |
| `lg` | 992-1199px | Collapsed to icons |
| `<lg` | <992px | Overlay drawer |

### 4.3 Header Bits

| Area | What | Notes |
|------|------|-------|
| Left | Collapse button | Toggles sider collapsed state in appStore |
| Center | Global search | Navigates to `/search` |
| Right | Language toggle | zh-CN / en-US |
| | Unit switch | Metric / Imperial (stored in unitStore) |
| | User dropdown | Profile, logout |

---

## 5. Component Tree

```
<App>
  <ErrorBoundary>
    <ConfigProvider theme={theme}>     // Ant Design tokens
      <QueryClientProvider>            // TanStack Query context
        <RouterProvider router={router}>
          ├── /login → <LoginPage />
          │
          └── / → <ProtectedRoute>
                └── <MainLayout>
                      ├── <Layout.Sider>
                      │   └── <Menu items={filteredByRole} />
                      ├── <Layout.Header>
                      │   ├── <Breadcrumb />
                      │   ├── <LanguageSwitcher />
                      │   └── <UserDropdown />
                      └── <Layout.Content>
                            └── <Outlet />
                                  ├── SeamlessPipeListPage
                                  │   ├── PageHeader
                                  │   ├── FilterBar (inline)
                                  │   ├── Table
                                  │   └── Pagination
                                  ├── SeamlessPipeFormPage
                                  │   └── Form (Ant Design)
                                  ├── InboundListPage
                                  │   └── ...
                                  └── ...
```

---

## 6. State Management

### 6.1 What Goes Where

```
┌──────────────────────────────────────────────────────┐
│                  Client State (Zustand)                │
│                                                       │
│  authStore         appStore           unitStore       │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐  │
│  │ token       │  │ siderCollapsed│  │ unitSystem  │  │
│  │ user        │  │ theme        │  │ (metric /   │  │
│  │ isAuth      │  │ currentLang  │  │  imperial)  │  │
│  └─────────────┘  └──────────────┘  └─────────────┘  │
│                                                       │
├──────────────────────────────────────────────────────┤
│               Server State (TanStack Query)            │
│                                                       │
│  useSeamlessPipes(['seamless_pipes', filters])        │
│  useScreenPipes(['screen_pipes', filters])            │
│  useInboundRecords(['inbound', filters])              │
│  useSupplierList(['suppliers', filters])              │
│  ... (one hook per list/detail view)                  │
└──────────────────────────────────────────────────────┘
```

### 6.2 Zustand: Pure State, No Side Effects

Zustand stores only hold state and setter functions. No API calls. Login/logout is handled by a `useAuth` hook that calls the API and then writes to the store.

```ts
// stores/authStore.ts — pure state
interface AuthState {
  user: User | null;
  token: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  setUser: (user: User) => void;
  setToken: (token: string, refreshToken: string) => void;
  logout: () => void;
}
```

The login Hook calls `POST /auth/login`, then calls `authStore.setToken()`. The Axios interceptor handles token refresh — it calls `POST /auth/refresh`, then calls `authStore.setToken()`. The stores never touch the network.

### 6.3 TanStack Query Config

```ts
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 2 * 60 * 1000,    // 2min — data is "fresh" for 2 minutes
      gcTime: 5 * 60 * 1000,        // 5min — keep in cache after unmount
      retry: 1,                      // Retry once on failure
      refetchOnWindowFocus: false,   // Admin panels don't need this
    },
  },
});
```

Cache invalidation pattern: after a mutation (create/update/delete), invalidate the related query key:

```
Create pipe → invalidateQueries(['seamless_pipes'])
Create inbound → invalidateQueries(['inbound'], ['inventory'])
Create outbound → invalidateQueries(['outbound'], ['inventory'])
```

### 6.4 Typical Hook

```ts
// features/pipes/api/useSeamlessPipes.ts
export function useSeamlessPipes(filters: PipeFilter, page: number, pageSize: number) {
  return useQuery({
    queryKey: ['seamless_pipes', filters, page, pageSize],
    queryFn: () => pipeApi.getSeamlessPipes(filters, page, pageSize),
    placeholderData: keepPreviousData,  // Keep showing old data while next page loads
  });
}

export function useCreateSeamlessPipe() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateSeamlessPipeDto) => pipeApi.createSeamlessPipe(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['seamless_pipes'] });
    },
  });
}
```

---

## 7. API Layer

### 7.1 Axios Instance

```ts
// api/client.ts
const apiClient = axios.create({
  baseURL: '/api/v1',
  timeout: 30_000,
});

// Request interceptor: auto-inject Bearer token
apiClient.interceptors.request.use((config) => {
  const token = authStore.getState().token;
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});
```

### 7.2 401 Refresh Flow

This is the trickiest part and it works (mostly):

1. Request returns 401.
2. Check if this is the refresh request itself (to avoid loops).
3. Queue all concurrent requests while refresh is in flight.
4. Call `POST /auth/refresh` with the refresh token.
5. On success: update tokens in authStore, replay queued requests.
6. On failure: clear authStore, redirect to `/login`.

```ts
let isRefreshing = false;
let failedQueue: Array<{ resolve, reject }> = [];

apiClient.interceptors.response.use(
  (response) => response.data,
  async (error) => {
    const originalRequest = error.config;
    if (error.response?.status !== 401 || originalRequest._retry) {
      return Promise.reject(error.response?.data || error);
    }

    if (isRefreshing) {
      // Queue this request — it'll replay after refresh
      return new Promise((resolve, reject) => {
        failedQueue.push({ resolve, reject });
      }).then(token => {
        originalRequest.headers.Authorization = `Bearer ${token}`;
        return apiClient(originalRequest);
      });
    }

    originalRequest._retry = true;
    isRefreshing = true;

    try {
      const { data } = await axios.post('/api/v1/auth/refresh', {
        refresh_token: authStore.getState().refreshToken,
      });
      authStore.getState().setToken(data.access_token, data.refresh_token);
      failedQueue.forEach(p => p.resolve(data.access_token));
      originalRequest.headers.Authorization = `Bearer ${data.access_token}`;
      return apiClient(originalRequest);
    } catch {
      failedQueue.forEach(p => p.reject(error));
      authStore.getState().logout();
      window.location.href = '/login';
    } finally {
      isRefreshing = false;
      failedQueue = [];
    }
  }
);
```

### 7.3 API Function Pattern

```ts
// features/pipes/api/pipeApi.ts
export const pipeApi = {
  getSeamlessPipes(filters: PipeFilter, page: number, pageSize: number) {
    return apiClient.get<PaginatedResponse<SeamlessPipe>>('/seamless-pipes', {
      params: { ...filters, page, page_size: pageSize },
    });
  },
  getSeamlessPipe(id: number) {
    return apiClient.get<SeamlessPipe>(`/seamless-pipes/${id}`);
  },
  createSeamlessPipe(data: CreateSeamlessPipeDto) {
    return apiClient.post<SeamlessPipe>('/seamless-pipes', data);
  },
  updateSeamlessPipe(id: number, data: CreateSeamlessPipeDto) {
    return apiClient.put<SeamlessPipe>(`/seamless-pipes/${id}`, data);
  },
  deleteSeamlessPipe(id: number) {
    return apiClient.delete(`/seamless-pipes/${id}`);
  },
};
```

### 7.4 Response Shapes

```ts
// types/api.ts
interface ApiResponse<T> {
  success: boolean;
  data: T;
  request_id: string;          // uuid v4
}

interface PaginatedResponse<T> extends ApiResponse<T[]> {
  meta: PaginationMeta;
}

interface PaginationMeta {
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
}

interface ApiErrorResponse {
  success: false;
  code: number;                // e.g., 11001 (auth), 12001 (pipe not found)
  message: string;
  request_id: string;
  details: Record<string, unknown> | null;
}
```

### 7.5 Runtime Zod Validation

We use `lib/validateResponse.ts` to validate API responses at runtime with Zod. Catches the case where the backend changes the response shape and the frontend doesn't know:

```ts
// zod-schemas/core.ts
const PaginatedResponseSchema = <T extends z.ZodTypeAny>(itemSchema: T) =>
  z.object({
    success: z.literal(true),
    data: z.array(itemSchema),
    meta: z.object({
      page: z.number(),
      page_size: z.number(),
      total: z.number(),
      total_pages: z.number(),
    }),
    request_id: z.string(),
  });

// lib/validateResponse.ts
function validateResponse<T>(schema: z.ZodType<T>, data: unknown): T {
  const result = schema.safeParse(data);
  if (!result.success) {
    // Log the mismatch but don't crash — return the data anyway
    console.error('Response validation failed:', result.error.issues);
    return data as T;
  }
  return result.data;
}
```

---

## 8. Auth Flow

### 8.1 Login

```
User types credentials
    │
    ▼
POST /api/v1/auth/login { username, password }
    │
    ▼
Response: { access_token, refresh_token, expires_in, user }
    │
    ├── useAuth hook calls authStore.setToken()
    ├── Persisted to localStorage
    └── Navigate to /pipes/seamless
```

### 8.2 Token Refresh

```
API returns 401
    │
    ▼
Queue concurrent requests
    │
    ▼
POST /api/v1/auth/refresh { refresh_token }
    │
    ├── Success → update tokens → replay queue
    └── Failure → logout → redirect /login
```

### 8.3 RBAC (Role-Based Menu)

Roles: `admin`, `warehouse`, `qc`, `sales`.

- Routes don't check roles (the backend enforces access).
- The sidebar menu filters items by role so users don't see what they can't use.
- Admin sees everything. Warehouse sees pipes/inventory. QC sees quality. Sales sees sales/customers.

---

## 9. Internationalization

### 9.1 Structure

```
i18n/
├── index.ts          ← i18next init
├── locales/
│   ├── zh/           ← Chinese (primary)
│   │   ├── common.json
│   │   ├── pipes.json
│   │   ├── inventory.json
│   │   ├── inbound.json
│   │   ├── outbound.json
│   │   ├── stock.json
│   │   ├── screen_pipes.json
│   │   ├── inventory_check.json
│   │   ├── location.json
│   │   ├── purchase.json
│   │   ├── sales.json
│   │   ├── quality.json
│   │   ├── contracts.json
│   │   ├── suppliers.json
│   │   ├── customers.json
│   │   ├── reports.json
│   │   ├── labels.json
│   │   ├── profile.json
│   │   ├── search.json
│   │   ├── system.json
│   │   └── validation.json
│   └── en/           ← English (same 21 files)
└── locale.ts         ← dayjs locale config
```

21 namespaces, mirrored between zh and en. Chinese is the default and primary language. English translations exist for all UI text but may lag behind on new features.

### 9.2 Usage

```tsx
import { useTranslation } from 'react-i18next';

function PipeTable() {
  const { t } = useTranslation('pipes');

  return (
    <Table columns={[
      { title: t('pipe_number'), dataIndex: 'pipe_number' },
      { title: t('grade'), dataIndex: 'grade' },
    ]} />
  );
}
```

---

## 10. Ant Design Theme

### 10.1 Industrial Blue Theme

```ts
// styles/theme.ts
const theme: ThemeConfig = {
  token: {
    colorPrimary: '#1B3A5C',       // Deep sea blue — industrial, not playful
    colorInfo: '#1B3A5C',
    colorSuccess: '#389E0D',       // Jungle green for "in stock"
    colorWarning: '#D48806',       // Golden yellow
    colorError: '#CF1322',         // Red for errors/scrapped

    colorBgLayout: '#F0F2F5',     // Page background
    colorBgContainer: '#FFFFFF',  // Card background
    colorText: '#1A1A1A',         // Body text
    colorTextSecondary: '#595959', // Secondary text

    // System fonts — no extra downloads
    fontFamily: `-apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif`,
    fontFamilyCode: `'SF Mono', 'Cascadia Code', 'Consolas', 'Courier New', monospace`,

    borderRadius: 6,
    borderRadiusLG: 8,
    margin: 16,
    padding: 16,
  },
  components: {
    Menu: {
      // Dark sidebar menu
      itemBg: 'transparent',
      itemColor: '#FFFFFF',
      itemHoverBg: 'rgba(255,255,255,0.12)',
      itemSelectedBg: 'rgba(255,255,255,0.2)',
    },
    Table: {
      headerBg: '#F5F7FA',
      rowHoverBg: '#E6F0FF',
      cellFontSize: 13,
    },
  },
};
```

### 10.2 Status Color Mapping

| Status | Color |
|--------|-------|
| In Stock | Green (`colorSuccess`) |
| Outbound | Orange (`colorWarning`) |
| Scrapped | Red (`colorError`) |
| Draft | Gray (default) |
| Pending | Gold |
| Approved | Blue |
| Completed | Green |
| Cancelled | Gray |

---

## 11. Data Flow

```
User clicks / types / submits
    │
    ▼
React Component
    │
    ├── Client state change → Zustand → re-render subscribed components
    │
    └── Server data request → TanStack Query hook
            │
            ▼
        API function (pipeApi.getSeamlessPipes)
            │
            ▼
        Axios interceptor (injects token)
            │
            ▼
        HTTP request to /api/v1/...
            │
            ▼
        Backend response
            │
            ▼
        Axios interceptor (unified error handling + 401 refresh)
            │
            ▼
        TanStack Query (caching + state)
            │
            ├── loading → Skeleton / Spin
            ├── success → render data
            └── error → notification + fallback

Mutations (create/update/delete):
    TanStack Query mutation
            │
            ▼
        API request → success
            │
            ▼
        queryClient.invalidateQueries() → list auto-refreshes
```

---

## 12. Things We Learned The Hard Way

1. **No route loaders for data fetching.** React Router loaders look clean but they don't integrate with TanStack Query's caching. Keep data fetching in components via hooks.

2. **Zustand stays pure.** Don't call APIs in stores. API calls go in hooks, results go into stores via setters. Keeps the store testable and the data flow predictable.

3. **chunkSizeWarningLimit = 600.** Vite's default 500 kB warning fires constantly with Ant Design. Bump it. The actual gzip size is what matters.

4. **Ant Design 5 is CSS-in-JS.** No need for Less/SASS imports for component customization. Use `ConfigProvider` tokens. Only reach for less overrides when token customization doesn't cut it.

5. **TanStack Query's `placeholderData: keepPreviousData` is your friend.** Without it, pagination causes a loading flash. With it, users see the old data until the new page loads.

6. **`key` prop on form pages for edit vs new.** We use `<InboundFormPage key="new" />` vs `<InboundFormPage key="edit" />` so React unmounts and remounts the form component when switching between new and edit modes. Without this, Ant Design form fields don't reset properly.

7. **No MSW.** We planned to use MSW for mock API while developing the frontend independently. In practice, running both frontend and backend together was faster than maintaining mock handlers. MSW is great for testing but not worth it for daily development here.

8. **The Axios 401 refresh interceptor works, but it's fragile.** The queue logic has had bugs. If the refresh token itself is invalid, you get stuck in a loop. Make sure the refresh endpoint returns a proper 401 so the interceptor can distinguish "refresh failed" from "original request failed".

9. **Route paths use `:id` syntax (React Router v7).** This is standard. The backend uses `{id}` syntax (Axum 0.8). Don't confuse them.

10. **System fonts are fine.** We don't need to download custom fonts. PingFang SC / Microsoft YaHei for Chinese, SF Pro / Segoe UI for English. Zero extra network requests.
