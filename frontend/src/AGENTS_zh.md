# `frontend/src/` — App Structure & Shared Infrastructure

How everything's wired up: entry points, shared modules, and the recipe for adding new features. Frontend of the **ERP（通用企业资源计划系统）** monorepo — backend crate `erp-server`, backend data layer **SQLite3** (`sqlite://data/erp.db?mode=rwc`, sqlx 0.8 sqlite feature).

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

### `api/` — 原生 fetch 封装

```ts
import api from '@/api/client'  // native fetch wrapper, baseURL '/api/v1'
// Authorization: Bearer token 自动附加
// 401 → 清除认证状态并跳转 /login
```

- 全项目共用同一个 fetch 封装（`src/api/client.ts`）。
- 自动附加 `Authorization: Bearer <token>`。
- 401 时清除认证状态并跳转 `/login`。

### `lib/` — 运行时校验

- `validateResponse.ts` 使用 `schema.safeParse(data)` 对 API 响应做运行时校验。

### `hooks/` — Shared Hooks

- `useAuth.ts` — Login/logout/current user tracking.
- `usePagination.ts` — Pagination controls.

### `stores/` — Zustand State Management

- `authStore.ts` — Token, user info, login/logout actions.
- `appStore.ts` — Global UI state (sidebar collapse, theme).

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
- Use `useTranslation('feature_name')` in components.

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

- Uses `createBrowserRouter` (not a flat array).
- `ProtectedRoute` gates access behind auth check.
- `Outlet` for nested layouts.
- No lazy loading yet — everything's eagerly loaded.

### `shared/` — Shared Components & Utilities

- `components/` — 13 reusable UI components:
  - `ActionButton` — 带 loading 状态的标准操作按钮
  - `Can` — 基于权限的渲染守卫
  - `DataTable` — 通用 Ant Design 表格封装
  - `EmptyState` — 空列表占位
  - `ErrorBoundary` — 捕获渲染错误并回退
  - `FormField` — 表单字段封装
  - `ItemPicker` — 商品/SKU 选择器
  - `ListPageTemplate` — 标准列表页脚手架
  - `PageLayout` — 标准页面布局封装
  - `RouteBoundary` — 路由级错误边界
  - `SearchBar` — 防抖搜索输入
  - `StatusBadge` — 彩色状态徽章
  - `StatusTag` — 彩色状态标签
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
├── orders.ts      ← 采购订单/销售订单 schemas
├── inventory.ts   ← 商品/SKU, 入库, 出库
├── reports.ts     ← Report parameter schemas
├── search.ts      ← Search query schemas
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
