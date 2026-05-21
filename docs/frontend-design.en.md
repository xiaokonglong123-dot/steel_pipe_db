# Seamless Steel Pipe & Screen Pipe Management System — Frontend Design Document

> **Document Version**: v1.1
> **Created**: 2026-05-19
> **Based on Backend Design**: docs/详细设计文档.md
> **Tech Stack**: React 19 + Ant Design 5 + TypeScript + Vite

---

## Revision History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| v1.0 | 2026-05-19 | Initial version | - |
| v1.1 | 2026-05-19 | Axios 401 changed to refresh-then-logout; Zustand pure state (login moved to Hook); removed global loading; removed route loader; use system fallback fonts; added MSW mock design; staleTime adjusted to 2min | - |

---

## Table of Contents

1. [Tech Stack & Dependencies](#1-tech-stack--dependencies)
2. [Project Directory Structure](#2-project-directory-structure)
3. [Page Route Design](#3-page-route-design)
4. [Page List & Features](#4-page-list--features)
5. [Layout Structure](#5-layout-structure)
6. [Component Tree](#6-component-tree)
7. [State Management](#7-state-management)
8. [API Layer Design](#8-api-layer-design)
9. [Authentication & Authorization Flow](#9-authentication--authorization-flow)
10. [Internationalization](#10-internationalization)
11. [Ant Design 5 Theme Customization](#11-ant-design-5-theme-customization)
12. [Frontend Data Flow Overview](#12-frontend-data-flow-overview)
13. [Implementation Recommendations](#13-implementation-recommendations)

---

## 1. Tech Stack & Dependencies

### 1.1 Technology Selection

| Layer | Selection | Version | Notes |
|-------|-----------|---------|-------|
| **Build Tool** | Vite | 6.x | Extremely fast dev server startup, ESBuild compilation |
| **UI Framework** | React | 19.x | — |
| **Language** | TypeScript | 5.x | Strict mode |
| **Component Library** | Ant Design | 5.x | Enterprise-grade component library, mature tables/forms/menus |
| **Routing** | React Router | 7.x | Supports nested routes, loaders, actions |
| **Server State** | TanStack Query | 5.x | Caching, background refresh, optimistic updates |
| **Client State** | Zustand | 5.x | Lightweight, no boilerplate |
| **HTTP Client** | Axios | 1.x | Interceptors, request/response transformation |
| **Internationalization** | react-i18next | 15.x | Integrated with i18next ecosystem |
| **Tables** | Ant Design Table | — | Built into Ant Design 5 |
| **Charts** | @ant-design/charts | 2.x | Based on G2Plot, consistent with Ant Design styling |
| **Forms** | Ant Design Form | — | Built into Ant Design 5 |
| **Date Handling** | dayjs | 1.x | Built into Ant Design 5 |
| **Code Standards** | ESLint + Prettier | — | — |

### 1.2 package.json Core Dependencies

```json
{
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "react-router-dom": "^7.0.0",
    "antd": "^5.20.0",
    "@ant-design/icons": "^5.5.0",
    "@ant-design/charts": "^2.1.0",
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
    "msw": "^2.6.0",
    "@faker-js/faker": "^9.3.0"
  }
}
```

---

## 2. Project Directory Structure

Uses a **Feature-Sliced Design** variant, organizing code by feature module.

```
pipe-management-frontend/
├── index.html
├── vite.config.ts                  # Vite config (proxy, plugins)
├── tsconfig.json
├── tsconfig.node.json
├── .eslintrc.cjs
├── .prettierrc
├── public/
│   └── favicon.ico
├── src/
│   ├── main.tsx                    # Application entry point
│   ├── App.tsx                     # Root component (Provider assembly)
│   ├── vite-env.d.ts
│   │
│   ├── routes/
│   │   ├── index.tsx               # Route configuration summary
│   │   ├── ProtectedRoute.tsx      # Auth-guarded route wrapper
│   │   └── routes.ts               # Route definition constants
│   │
│   ├── layouts/
│   │   ├── MainLayout.tsx          # Main layout (sidebar + header + content)
│   │   ├── MainLayout.less
│   │   ├── Sidebar.tsx             # Side navigation
│   │   ├── Header.tsx              # Top bar (user info, language switcher, notifications)
│   │   └── components/
│   │       ├── Logo.tsx            # System logo
│   │       ├── UserDropdown.tsx    # User dropdown menu
│   │       └── LanguageSwitcher.tsx# Language switcher
│   │
│   ├── features/                   # Organized by business module
│   │   ├── auth/
│   │   │   ├── pages/
│   │   │   │   ├── LoginPage.tsx
│   │   │   │   └── LoginPage.less
│   │   │   ├── stores/
│   │   │   │   └── authStore.ts     # Zustand store
│   │   │   ├── api/
│   │   │   │   └── authApi.ts       # Login/logout/refresh API
│   │   │   ├── hooks/
│   │   │   │   └── useAuth.ts
│   │   │   └── types.ts
│   │   │
│   │   ├── pipes/                   # Pipe management
│   │   │   ├── pages/
│   │   │   │   ├── SeamlessPipeListPage.tsx
│   │   │   │   ├── SeamlessPipeDetailPage.tsx
│   │   │   │   ├── SeamlessPipeFormPage.tsx
│   │   │   │   ├── ScreenPipeListPage.tsx
│   │   │   │   ├── ScreenPipeDetailPage.tsx
│   │   │   │   ├── ScreenPipeFormPage.tsx
│   │   │   │   └── UnifiedPipeSearchPage.tsx
│   │   │   ├── components/
│   │   │   │   ├── PipeFilterBar.tsx
│   │   │   │   ├── PipeTable.tsx
│   │   │   │   ├── PipeForm.tsx
│   │   │   │   ├── PipeDetailCard.tsx
│   │   │   │   ├── GradeTag.tsx
│   │   │   │   └── PipeStatusBadge.tsx
│   │   │   ├── api/
│   │   │   │   └── pipeApi.ts
│   │   │   ├── hooks/
│   │   │   │   ├── useSeamlessPipes.ts
│   │   │   │   └── useScreenPipes.ts
│   │   │   └── types.ts
│   │   │
│   │   ├── inventory/               # Inventory management
│   │   │   ├── pages/
│   │   │   │   ├── InventoryPage.tsx
│   │   │   │   ├── InboundListPage.tsx
│   │   │   │   ├── InboundFormPage.tsx
│   │   │   │   ├── OutboundListPage.tsx
│   │   │   │   ├── OutboundFormPage.tsx
│   │   │   │   ├── InventoryLogPage.tsx
│   │   │   │   ├── InventoryCheckPage.tsx
│   │   │   │   └── LocationManagePage.tsx
│   │   │   ├── components/
│   │   │   │   ├── InboundForm.tsx
│   │   │   │   ├── OutboundForm.tsx
│   │   │   │   ├── StockSummaryCards.tsx
│   │   │   │   ├── InventoryTable.tsx
│   │   │   │   └── LocationTree.tsx
│   │   │   ├── api/
│   │   │   │   └── inventoryApi.ts
│   │   │   ├── hooks/
│   │   │   │   └── useInventory.ts
│   │   │   └── types.ts
│   │   │
│   │   ├── quality/                 # Quality management
│   │   │   ├── pages/
│   │   │   │   ├── QualityCertListPage.tsx
│   │   │   │   ├── QualityCertFormPage.tsx
│   │   │   │   ├── QualityTracePage.tsx
│   │   │   │   └── Api5ctRefPage.tsx
│   │   │   ├── components/
│   │   │   │   ├── CertFileUploader.tsx
│   │   │   │   ├── TraceTimeline.tsx
│   │   │   │   └── GradeCompareTable.tsx
│   │   │   ├── api/
│   │   │   │   └── qualityApi.ts
│   │   │   └── types.ts
│   │   │
│   │   ├── purchases/               # Purchase management (independent module)
│   │   │   ├── pages/
│   │   │   │   ├── PurchaseOrderListPage.tsx
│   │   │   │   ├── PurchaseOrderDetailPage.tsx
│   │   │   │   ├── PurchaseOrderFormPage.tsx
│   │   │   │   ├── SupplierListPage.tsx
│   │   │   │   └── SupplierFormPage.tsx
│   │   │   ├── components/
│   │   │   │   ├── OrderStatusTag.tsx
│   │   │   │   └── SupplierSelect.tsx
│   │   │   ├── api/
│   │   │   │   ├── purchaseApi.ts
│   │   │   │   └── supplierApi.ts
│   │   │   └── types.ts
│   │   │
│   │   ├── sales/                   # Sales management (independent module)
│   │   │   ├── pages/
│   │   │   │   ├── SalesOrderListPage.tsx
│   │   │   │   ├── SalesOrderDetailPage.tsx
│   │   │   │   ├── SalesOrderFormPage.tsx
│   │   │   │   ├── CustomerListPage.tsx
│   │   │   │   └── CustomerFormPage.tsx
│   │   │   ├── components/
│   │   │   │   ├── OrderStatusTag.tsx
│   │   │   │   ├── CustomerSelect.tsx
│   │   │   │   └── AtpBadge.tsx
│   │   │   ├── api/
│   │   │   │   ├── salesApi.ts
│   │   │   │   └── customerApi.ts
│   │   │   └── types.ts
│   │   │
│   │   ├── data-io/                # Data import/export
│   │   │   ├── pages/
│   │   │   │   ├── ImportPage.tsx
│   │   │   │   └── ExportPage.tsx
│   │   │   ├── components/
│   │   │   │   ├── FileUploader.tsx
│   │   │   │   └── ImportResultTable.tsx
│   │   │   └── types.ts
│   │   │
│   │   ├── reports/                # Reports & statistics
│   │   │   ├── pages/
│   │   │   │   ├── InventoryReportPage.tsx
│   │   │   │   ├── TrendReportPage.tsx
│   │   │   │   └── DashboardPage.tsx
│   │   │   ├── components/
│   │   │   │   ├── KpiCards.tsx
│   │   │   │   ├── StockPieChart.tsx
│   │   │   │   ├── TrendLineChart.tsx
│   │   │   │   └── ReportFilterBar.tsx
│   │   │   └── api/
│   │   │       └── reportApi.ts
│   │   │
│   │   └── system/                 # System management
│   │       ├── pages/
│   │       │   ├── UserListPage.tsx
│   │       │   ├── UserFormPage.tsx
│   │       │   ├── OperationLogPage.tsx
│   │       │   └── ProfilePage.tsx
│   │       ├── components/
│   │       │   ├── RoleTag.tsx
│   │       │   └── UserForm.tsx
│   │       ├── api/
│   │       │   └── userApi.ts
│   │       └── types.ts
│   │
│   ├── shared/                     # Shared layer
│   │   ├── components/
│   │   │   ├── PageHeader.tsx       # Page title component
│   │   │   ├── SearchBar.tsx        # Generic search bar
│   │   │   ├── ConfirmModal.tsx     # Confirmation dialog
│   │   │   ├── LoadingSpin.tsx      # Loading spinner
│   │   │   ├── EmptyState.tsx       # Empty state placeholder
│   │   │   ├── ErrorBoundary.tsx    # Error boundary
│   │   │   └── PageContainer.tsx    # Page container (breadcrumb + content)
│   │   ├── hooks/
│   │   │   ├── usePagination.ts     # Pagination logic reuse
│   │   │   ├── useUnitConvert.ts    # Unit conversion hook
│   │   │   └── usePermission.ts     # Permission check hook
│   │   ├── utils/
│   │   │   ├── format.ts            # Data formatting
│   │   │   ├── unit-convert.ts      # Imperial/metric conversion utilities
│   │   │   ├── pipe-number.ts       # Pipe number generation/parsing
│   │   │   └── constants.ts         # Constants (grade list, end types, etc.)
│   │   └── types/
│   │       ├── api.ts               # Unified API response types
│   │       └── common.ts            # Common type definitions
│   │
│   ├── api/                        # API infrastructure
│   │   ├── client.ts               # Axios instance + interceptors
│   │   ├── types.ts                # Common request/response types
│   │   └── queryClient.ts          # TanStack Query Client configuration
│   │
│   ├── stores/                     # Global Zustand stores
│   │   ├── authStore.ts            # Authentication state
│   │   ├── appStore.ts             # Application state (sidebar collapse, theme, etc.)
│   │   └── unitStore.ts            # Unit system preference
│   │
│   ├── mocks/                      # MSW Mock API (dev environment only)
│   │   ├── browser.ts              # Worker initialization
│   │   ├── handlers.ts             # Route handler definitions
│   │   └── data/                   # Mock data generation
│   │       ├── pipes.ts
│   │       └── auth.ts
│   │
│   ├── i18n/                       # Internationalization
│   │   ├── index.ts                # i18next initialization
│   │   ├── resources/
│   │   │   ├── zh/
│   │   │   │   ├── common.json
│   │   │   │   ├── pipes.json
│   │   │   │   ├── inventory.json
│   │   │   │   ├── quality.json
│   │   │   │   ├── purchase.json
│   │   │   │   ├── sales.json
│   │   │   │   ├── system.json
│   │   │   │   └── validation.json
│   │   │   └── en/
│   │   │       ├── common.json
│   │   │       ├── pipes.json
│   │   │       ├── inventory.json
│   │   │       ├── quality.json
│   │   │       ├── purchase.json
│   │   │       ├── sales.json
│   │   │       ├── system.json
│   │   │       └── validation.json
│   │   └── locale.ts               # dayjs locale configuration
│   │
│   └── styles/                     # Global styles
│       ├── global.less              # Global style overrides
│       ├── theme.ts                # Ant Design 5 theme tokens
│       └── variables.less          # Less variables
│
└── .env                            # Environment variables (API_BASE_URL)
```

---

## 3. Page Route Design

### 3.1 Route Structure

```
/login                                  # Login page (public)
/                                       # Main layout (requires authentication)
├── dashboard                           # Dashboard home
│
├── pipes                               # Pipe management
│   ├── seamless                        # Seamless pipe list
│   ├── seamless/new                    # New seamless pipe
│   ├── seamless/:id                    # Seamless pipe detail
│   ├── seamless/:id/edit              # Edit seamless pipe
│   ├── screen                          # Screen pipe list
│   ├── screen/new                      # New screen pipe
│   ├── screen/:id                      # Screen pipe detail
│   ├── screen/:id/edit                # Edit screen pipe
│   └── search                          # Unified pipe search
│
├── inventory                           # Inventory management
│   ├── stock                           # Real-time inventory
│   ├── inbound                         # Inbound records
│   ├── inbound/new                     # New inbound
│   ├── outbound                        # Outbound records
│   ├── outbound/new                    # New outbound
│   ├── logs                            # Inventory audit trail
│   ├── checks                          # Inventory check management
│   │   └── :id                         # Inventory check detail
│   └── locations                       # Location management
│
├── quality                             # Quality management
│   ├── certs                           # Quality certificates
│   ├── certs/new                       # New quality certificate
│   ├── certs/:id                       # Quality certificate detail
│   ├── trace                           # Quality traceability
│   └── api5ct-ref                      # API 5CT standard reference
│
├── purchases                           # Purchase management (independent module)
│   ├── orders                          # Purchase orders
│   │   ├── :id                         # Purchase order detail
│   │   └── new                         # New purchase order
│   └── suppliers                       # Supplier management
│
├── sales                               # Sales management (independent module)
│   ├── orders                          # Sales orders
│   │   ├── :id                         # Sales order detail
│   │   └── new                         # New sales order
│   ├── customers                       # Customer management
│   └── atp                             # Available-to-Promise (ATP) query
│
├── data-io                             # Data import/export
│   ├── import                          # Import
│   └── export                          # Export
│
├── reports                             # Reports & statistics
│   ├── inventory                       # Inventory reports
│   ├── trends                          # Trend analysis
│   └── dashboard                       # Dashboard (alias for dashboard)
│
└── system                              # System management
    ├── users                           # User management
    ├── users/:id                       # User edit
    ├── logs                            # Operation logs
    └── profile                         # Profile settings
```

### 3.2 Route Configuration Definition

```typescript
// src/routes/routes.ts
import { lazy } from 'react';

// Lazy-load all pages
const LoginPage = lazy(() => import('@/features/auth/pages/LoginPage'));
const DashboardPage = lazy(() => import('@/features/reports/pages/DashboardPage'));

// Pipe management
const SeamlessPipeList = lazy(() => import('@/features/pipes/pages/SeamlessPipeListPage'));
const SeamlessPipeForm = lazy(() => import('@/features/pipes/pages/SeamlessPipeFormPage'));
const ScreenPipeList = lazy(() => import('@/features/pipes/pages/ScreenPipeListPage'));
const ScreenPipeForm = lazy(() => import('@/features/pipes/pages/ScreenPipeFormPage'));

// ... other pages

export interface RouteConfig {
  path: string;
  element: React.LazyExoticComponent<React.ComponentType>;
  permissions?: Role[];    // Allowed roles
  titleKey: string;       // i18n key for page title
  hideInMenu?: boolean;   // Whether to hide in menu
  icon?: React.ComponentType;
}

export const menuRoutes: RouteConfig[] = [
  { path: 'dashboard', element: DashboardPage, titleKey: 'menu.dashboard', icon: DashboardOutlined, permissions: ['admin', 'warehouse', 'qc', 'sales'] },

  // Pipe management
  { path: 'pipes/seamless', element: SeamlessPipeList, titleKey: 'menu.pipes.seamless', icon: NodeIndexOutlined, permissions: ['admin', 'warehouse', 'qc'] },
  { path: 'pipes/seamless/new', element: SeamlessPipeForm, titleKey: 'menu.pipes.seamless_new', hideInMenu: true, permissions: ['admin', 'warehouse'] },
  { path: 'pipes/screen', element: ScreenPipeList, titleKey: 'menu.pipes.screen', permissions: ['admin', 'warehouse', 'qc'] },

  // Inventory management
  { path: 'inventory/stock', element: InventoryPage, titleKey: 'menu.inventory.stock', icon: DatabaseOutlined, permissions: ['admin', 'warehouse', 'sales'] },
{ path: 'inventory/inbound', element: InboundListPage, titleKey: 'menu.inventory.inbound', icon: ImportOutlined, permissions: ['admin', 'warehouse', 'sales'] },
{ path: 'inventory/inbound/:id/approve', element: InboundApprovalPanel, titleKey: 'menu.inventory.inbound_approve', hideInMenu: true, permissions: ['admin', 'warehouse'] },
{ path: 'inventory/outbound', element: OutboundListPage, titleKey: 'menu.inventory.outbound', icon: ExportOutlined, permissions: ['admin', 'warehouse', 'sales'] },
{ path: 'inventory/outbound/:id/approve', element: OutboundApprovalPanel, titleKey: 'menu.inventory.outbound_approve', hideInMenu: true, permissions: ['admin', 'warehouse'] },

  // ... other modules
];
```

### 3.3 Route Guard

```typescript
// src/routes/ProtectedRoute.tsx
// Get current user role from authStore
// Compare against route config permissions
// Show 403 page when unauthorized
// Redirect to /login when not authenticated
```

---

## 4. Page List & Features

### 4.1 Page Overview (approximately 35 pages)

| Module | Page | Description | Role Restriction |
|--------|------|-------------|-----------------|
| **Auth** | Login | Username/password login, language switch (zh/en) | Public |
| | Profile Settings | Change password, language preference, unit system | All |
| **Dashboard** | Home | KPI cards (total inventory / inbound-outbound stats), charts, quick links | All |
| **Pipe Management** | Seamless Pipe List | Table display + advanced filtering + batch operations | admin/wh/qc |
| | New Seamless Pipe | Form page, cascading grade/spec selection | admin/wh |
| | Seamless Pipe Detail | Display all fields + related inbound/outbound records | admin/wh/qc |
| | Screen Pipe List | Same as seamless pipe | admin/wh/qc |
| | New/Edit Screen Pipe | Includes screen-pipe-specific fields | admin/wh |
| | Unified Search | Cross-type search | All |
| **Inventory Management** | Real-time Inventory | Aggregated query, grouped by grade/spec | All |
| | Inbound Record List | Display inbound history | admin/wh |
| | New Inbound | Select pipe + fill inbound info | admin/wh |
| | Outbound Record List | Display outbound history | admin/wh |
| | New Outbound | Select pipe + fill outbound info | admin/wh |
| | Inventory Audit Trail | Full lifecycle movement records per pipe | admin/wh |
| | Inventory Check | Create/execute checks, view variance reports | admin/wh |
| | Location Management | Zone/shelf tree management | admin/wh |
| **Quality Management** | Quality Certificate List | Display quality records | admin/qc |
| | New Quality Certificate | Upload files, fill test results | admin/qc |
| | Quality Traceability | Query traceability chain by heat number/pipe number | admin/qc |
| | API 5CT Standard Reference | Query mechanical property references by grade | All |
| **Purchase Management** | Purchase Order List | PO CRUD + approval workflow | admin/sales |
| | Purchase Order Detail | Order details + related inbound records | admin/sales |
| | Supplier Management | Supplier information maintenance | admin/sales |
| **Sales Management** | Sales Order List | SO CRUD + approval workflow | admin/sales |
| | Sales Order Detail | Order details + ATP + outbound records | admin/sales |
| | Customer Management | Customer information maintenance | admin/sales |
| | ATP Query | View available-to-promise by spec | admin/sales/wh |
| **Data Import/Export** | Import Page | Upload file, configure mapping, view import results | admin/wh/qc |
| | Export Page | Select data range, generate Excel | admin/wh/qc/sales |
| **Reports & Statistics** | Inventory Report | Summary by type/grade + monthly changes | admin/wh |
| | Trend Analysis | Inbound/outbound trend charts | admin |
| **System Management** | User Management | User list + CRUD + role assignment | admin |
| | Operation Logs | Audit log query | admin |
| | Profile Settings | Preference settings | All |

### 4.2 Core Page Prototype Descriptions

#### Login Page

```
┌─────────────────────────────────────────────┐
│                                             │
│            [System Logo]                     │
│       Seamless Steel Pipe & Screen Pipe      │
│       Management System                      │
│                                             │
│  ┌─────────────────────────────────────┐    │
│  │  Username                            │    │
│  │  [________________________]          │    │
│  │  Password                            │    │
│  │  [________________________]          │    │
│  │                                     │    │
│  │  [     LOG IN      ] [ En / 中文 ]  │    │
│  └─────────────────────────────────────┘    │
│                                             │
│  © 2026 Pipe Management System              │
└─────────────────────────────────────────────┘
```

#### Main Layout

```
┌──────────┬──────────────────────────────────────────┐
│          │  ← Sidebar collapse  [ Search... ]  User ▼│
│  Sidebar ├──────────────────────────────────────────┤
│  Nav     │  Breadcrumb > Current page                │
│          │                                           │
│  Level 1 │  ┌─────────────────────────────────────┐  │
│  ───────│  │                                     │  │
│  📊 Dash │  │        Page Content Area            │  │
│  ───────│  │                                     │  │
│  🔩 Pipe │  │  (Ant Design Pro layout)            │  │
│  │ Seaml│  │                                     │  │
│  │ Screen│  │                                     │  │
│  │ Search│  └─────────────────────────────────────┘  │
│  ───────│                                           │
│  📦 Inv  │                                           │
│  ├ Stock │                                           │
│  ├ Inbnd │                                           │
│  ├ Outbd │                                           │
│  ├ Trail │                                           │
│  ├ Check │                                           │
│  └ Locat │                                           │
│  ───────│                                           │
│  ✅ Qual │                                           │
│  📋 Purch│                                           │
│  📊 Rpts │                                           │
│  ⚙ System│                                           │
└──────────┴──────────────────────────────────────────┘
```

#### List Page Common Pattern (Seamless Pipe List Example)

```
┌─────────────────────────────────────────────────────┐
│  Seamless Pipe Management    [ + New ] [ Import ] [ Export ] │
│                                                     │
│  ┌─ Filter Conditions ─────────────────────────────┐ │
│  │  Grade: [Dropdown]  OD: [__]~[__]  WT: [__]~[__]│ │
│  │  Status: [Dropdown]  Location: [Dropdown]  Search: [______] │ │
│  │  [Query] [Reset]                                │ │
│  └───────────────────────────────────────────────┘ │
│                                                     │
│  ┌─ Table ──────────────────────────────────────────┐│
│  │ ☐ │ Pipe No. │ Grade │ Spec(OD×WT) │ Status │ Loc││
│  │ ☐ │ CSG-... │ J55  │ 4.5×0.250  │ ✔ In Stock │A-..││
│  │ ☐ │ CSG-... │ N80  │ 5.5×0.304  │ ➡ Outbound │B-..││
│  │ ☐ │ ...     │ ...  │ ...        │ ...   │... ││
│  │                                               ││
│  │  Total 128   ◀ 1 2 3 4 5 ... 10 ▶  20 per page ││
│  └───────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

#### Form Page Common Pattern

```
┌─────────────────────────────────────────────────────┐
│  New Seamless Pipe    [Back]                         │
│                                                     │
│  ┌─ Basic Information ─────────────────────────────┐ │
│  │  Pipe Number *: [_____________________]  [Auto] │ │
│  │  Product Type *: ○ Casing  ○ Tubing             │ │
│  │  Grade *:     [J55 ▼]                           │ │
│  │  O.D. (in) *: [____]  W.T. (in) *: [____]      │ │
│  │  Length (ft):  [____]  Unit Weight(lb/ft): [____] │ │
│  │  End Type:    [SC ▼]                            │ │
│  └───────────────────────────────────────────────┘ │
│  ┌─ Coupling Information ──────────────────────────┐ │
│  │  Coupling Type: [__________]  O.D.: [____]  Len:│ │
│  └───────────────────────────────────────────────┘ │
│  ┌─ Production Information ────────────────────────┐ │
│  │  Heat Number: [______________]  Serial No: [_________] │ │
│  │  Manufacturer: [______________]  Prod. Date: [📅__]  │ │
│  │  QC Certificate No.: [______________]           │ │
│  └───────────────────────────────────────────────┘ │
│                                                     │
│  [  SUBMIT  ]  [  CANCEL  ]                         │
└─────────────────────────────────────────────────────┘
```

---

## 5. Layout Structure

### 5.1 Ant Design Pro Layout

Uses Ant Design 5's `ProLayout` component (or a combination of `Layout` components):

```
┌──────────────────────────────────────────────────────────┐
│  Layout (flex container)                                  │
│                                                           │
│  ┌── Sider ──────────────────┬── Content ───────────────┐│
│  │                           │                          ││
│  │  Logo + System Name       │  Header                  ││
│  │                           │  ┌─────────────────────┐ ││
│  │  Menu (Ant Design Menu)   │  │ Collapse  Breadcrumb│ ││
│  │  ├── Dashboard            │  │ Lang Switch  User   │ ││
│  │  ├── Pipe Management ▼    │  └─────────────────────┘ ││
│  │  │   ├── Seamless Pipe    │                          ││
│  │  │   ├── Screen Pipe      │  Page Content            ││
│  │  │   └── Unified Search   │  ┌─────────────────────┐ ││
│  │  ├── Inventory Mgmt ▼     │  │                     │ ││
│  │  │   ├── Real-time Stock  │  │  (Page Content)     │ ││
│  │  │   ├── Inbound Mgmt     │  │                     │ ││
│  │  │   ├── Outbound Mgmt    │  │                     │ ││
│  │  │   ├── Audit Trail      │  └─────────────────────┘ ││
│  │  │   ├── Inventory Check  │                          ││
│  │  │   └── Location Mgmt    │                          ││
│  │  ├── Quality Mgmt ▼       │                          ││
│  │  ├── Purchase Mgmt ▼      │                          ││
│  │  ├── Sales Mgmt ▼         │                          ││
│  │  ├── Data Import/Export ▼ │                          ││
│  │  ├── Reports ▼            │                          ││
│  │  └── System Mgmt ▼        │                          ││
│  │                           │                          ││
│  │  (Collapsed: icons only)  │                          ││
│  └───────────────────────────┴──────────────────────────┘│
└──────────────────────────────────────────────────────────┘
```

### 5.2 Responsive Breakpoints

| Breakpoint | Width | Sidebar Behavior |
|------------|-------|-----------------|
| `xxl` | ≥1600px | Expanded, 220px wide |
| `xl` | 1200~1599px | Expanded, 220px wide |
| `lg` | 992~1199px | Collapsed, icons only (80px) |
| `md` | 768~991px | Collapsed, overlay popup |
| `sm` | <768px | Collapsed, overlay popup |

### 5.3 Header Content

| Area | Component | Description |
|------|-----------|-------------|
| Left | Collapse button + breadcrumb navigation | Click to collapse/expand sidebar |
| Center | Global search | Quick search for pipe numbers (shortcut Ctrl+K) |
| Right | Language switcher | Chinese / English toggle |
| | Unit system switch | Metric / Imperial quick toggle |
| | User avatar dropdown | Profile settings, logout |

---

## 6. Component Tree

### 6.1 Component Hierarchy

```
<App>
  <QueryClientProvider>       // TanStack Query
    <BrowserRouter>
      <AntdConfigProvider>    // Ant Design 5 theme
        <I18nextProvider>     // Internationalization
        <Routes>
          ├── /login → <LoginPage>
          │
          └── / → <MainLayout>       // Wrapped by ProtectedRoute
                ├── <Sidebar>
                │   ├── <Logo />
                │   └── <Menu />      // Dynamically generated by role
                │
                ├── <Header>
                │   ├── <Breadcrumb />
                │   ├── <LanguageSwitcher />
                │   ├── <UnitSwitch />
                │   └── <UserDropdown />
                │
                └── <Content>
                    ├── <Outlet />    // Nested route pages
                    │
                    ├── <PageHeader />        // Page title + action buttons
                    ├── <PageContainer />     // Standard page container
                    │
                    ├── List page common template
                    │   ├── <FilterBar />
                    │   ├── <AntTable />      // With pagination
                    │   └── <ActionButtons />
                    │
                    └── Form page common template
                        ├── <AntForm />
                        └── <FormActions />
```

### 6.2 Shared Component Inventory

| Component | Location | Description |
|-----------|----------|-------------|
| `PageHeader` | `shared/components` | Page title + breadcrumb + right-side action buttons |
| `PageContainer` | `shared/components` | White card container, unified padding |
| `SearchBar` | `shared/components` | Generic fuzzy search input |
| `ConfirmModal` | `shared/components` | Confirmation dialog (delete/approve/cancel operations) |
| `LoadingSpin` | `shared/components` | Full-screen or region-level loading state |
| `EmptyState` | `shared/components` | Empty data placeholder |
| `ErrorBoundary` | `shared/components` | React Error Boundary |
| `StatusTag` | `shared/components` | Generic status tag (color mapped by status) |
| `FileUploader` | `shared/components` | File upload component (single/multi-file) |

---

## 7. State Management

### 7.1 Layered State Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Client State (Zustand)                     │
│                                                             │
│  authStore          appStore          unitStore             │
│  ┌─────────────┐  ┌─────────────┐  ┌────────────┐          │
│  │ user        │  │ siderCollapsed│  │ unitSystem │         │
│  │ token       │  │ theme       │  │ (metric /  │         │
│  │ role        │  │ currentLang │  │  imperial) │         │
│  │ permissions │  │            │  └────────────┘          │
│  └─────────────┘  └─────────────┘                           │
├─────────────────────────────────────────────────────────────┤
│                   Server State (TanStack Query)               │
│                                                             │
│  useSeamlessPipes(queryKey: ['seamless_pipes', filters])    │
│  useScreenPipes(queryKey: ['screen_pipes', filters])        │
│  useInventory(queryKey: ['inventory', filters])             │
│  useInboundRecords(queryKey: ['inbound', filters])          │
│  useOutboundRecords(queryKey: ['outbound', filters])        │
│  usePurchaseOrders(queryKey: ['purchase_orders', filters])  │
│  useSalesOrders(queryKey: ['sales_orders', filters])        │
│  ...                                                         │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 Zustand Store Definition

```typescript
// src/stores/authStore.ts — pure state, no API calls
interface AuthState {
  user: User | null;
  token: string | null;
  refreshTokenValue: string | null;
  isAuthenticated: boolean;

  // Pure state operations (no requests)
  setUser: (user: User) => void;
  setToken: (token: string, refreshToken: string) => void;
  logout: () => void;
}

// Note: login() is called by the React Hook (useLogin) which calls the API, then writes via setToken;
//       refreshToken is handled by the Axios interceptor which calls the API, then writes via setToken.
```

### 7.3 TanStack Query Configuration

```typescript
// src/api/queryClient.ts
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 120_000,         // 2min — data considered fresh within this window (reduces polling requests)
      gcTime: 5 * 60_000,         // 5min cache
      retry: 2,                    // Retry on failure 2 times
      refetchOnWindowFocus: false, // No auto-refresh (not suitable for admin panels)
    },
    mutations: {
      retry: 0,
    },
  },
});

// Cache invalidation strategy
// Pipe changes → invalidate ['seamless_pipes'], ['screen_pipes']
// Inbound operation → invalidate ['inbound'], ['inventory']
// Outbound operation → invalidate ['outbound'], ['inventory']
```

### 7.4 Typical Hook Example

```typescript
// src/features/pipes/hooks/useSeamlessPipes.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { pipeApi } from '@/features/pipes/api/pipeApi';
import type { SeamlessPipe, PipeFilter } from '@/features/pipes/types';

export function useSeamlessPipes(filters: PipeFilter, page: number, pageSize: number) {
  return useQuery({
    queryKey: ['seamless_pipes', filters, page, pageSize],
    queryFn: () => pipeApi.getSeamlessPipes(filters, page, pageSize),
    placeholderData: keepPreviousData,  // Keep old data during pagination
  });
}

export function useCreateSeamlessPipe() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: SeamlessPipeCreateDto) => pipeApi.createSeamlessPipe(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['seamless_pipes'] });
    },
  });
}
```

---

## 8. API Layer Design

### 8.1 Axios Instance & Interceptors

```typescript
// src/api/client.ts
import axios from 'axios';
import { authStore } from '@/stores/authStore';

const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api/v1',
  timeout: 30_000,
  headers: { 'Content-Type': 'application/json' },
});

// Request interceptor: auto-inject token
apiClient.interceptors.request.use((config) => {
  const token = authStore.getState().token;
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Response interceptor: unified error handling + token auto-refresh (pure Axios, no store dependency)
let isRefreshing = false;
let failedQueue: Array<{ resolve: Function; reject: Function }> = [];

const processQueue = (error: unknown, token: string | null = null) => {
  failedQueue.forEach(({ resolve, reject }) => {
    if (token) resolve(token);
    else reject(error);
  });
  failedQueue = [];
};

// Independent Axios instance for refresh (avoids interceptor loop)
const refreshClient = axios.create({ baseURL: import.meta.env.VITE_API_BASE_URL || '/api/v1' });

apiClient.interceptors.response.use(
  (response) => response.data,
  async (error) => {
    const originalRequest = error.config;

    if (error.response?.status !== 401 || originalRequest._retry) {
      return Promise.reject(error.response?.data?.error || error);
    }

    if (isRefreshing) {
      return new Promise((resolve, reject) => {
        failedQueue.push({ resolve, reject });
      }).then((token) => {
        originalRequest.headers.Authorization = `Bearer ${token}`;
        return apiClient(originalRequest);
      });
    }

    originalRequest._retry = true;
    isRefreshing = true;

    try {
      const store = authStore.getState();
      const { data } = await refreshClient.post('/auth/refresh', {
        refresh_token: store.refreshTokenValue,
      });
      const { access_token, refresh_token } = data;

      // Pure state write
      store.setToken(access_token, refresh_token);

      processQueue(null, access_token);
      originalRequest.headers.Authorization = `Bearer ${access_token}`;
      return apiClient(originalRequest);
    } catch (refreshError) {
      processQueue(refreshError, null);
      authStore.getState().logout();
      window.location.href = '/login';
      return Promise.reject(refreshError);
    } finally {
      isRefreshing = false;
    }
  }
);

export default apiClient;
```

### 8.2 Type-Safe API Functions

```typescript
// src/features/pipes/api/pipeApi.ts
import apiClient from '@/api/client';
import type {
  SeamlessPipe,
  ScreenPipe,
  PipeFilter,
  PaginatedResponse,
  SeamlessPipeCreateDto,
  ScreenPipeCreateDto,
} from '../types';

export const pipeApi = {
  // Seamless pipes
  getSeamlessPipes(filters: PipeFilter, page: number, pageSize: number) {
    return apiClient.get<PaginatedResponse<SeamlessPipe>>('/seamless-pipes', {
      params: { ...filters, page, page_size: pageSize },
    });
  },

  getSeamlessPipe(id: number) {
    return apiClient.get<SeamlessPipe>(`/seamless-pipes/${id}`);
  },

  createSeamlessPipe(data: SeamlessPipeCreateDto) {
    return apiClient.post<SeamlessPipe>('/seamless-pipes', data);
  },

  updateSeamlessPipe(id: number, data: SeamlessPipeCreateDto) {
    return apiClient.put<SeamlessPipe>(`/seamless-pipes/${id}`, data);
  },

  deleteSeamlessPipe(id: number) {
    return apiClient.delete(`/seamless-pipes/${id}`);
  },

  // Screen pipes — similar structure
  getScreenPipes(filters: PipeFilter, page: number, pageSize: number) {
    return apiClient.get<PaginatedResponse<ScreenPipe>>('/screen-pipes', {
      params: { ...filters, page, page_size: pageSize },
    });
  },

  // Unified search
  searchPipes(query: string) {
    return apiClient.get<{ seamless_pipes: SeamlessPipe[]; screen_pipes: ScreenPipe[] }>(
      '/pipes/search',
      { params: { q: query } }
    );
  },
};
```

### 8.3 Unified Type Definitions

```typescript
// src/shared/types/api.ts
export interface ApiResponse<T> {
  success: boolean;
  data: T;
  meta?: PaginationMeta;
  request_id: string;
}

export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

export interface PaginationMeta {
  page: number;
  page_size: number;
  total: number;
}

export interface PaginatedRequest {
  page?: number;
  page_size?: number;
  sort_by?: string;
  sort_order?: 'asc' | 'desc';
}
```

---

## 9. Authentication & Authorization Flow

### 9.1 Login Flow

```
User enters credentials
    │
    ▼
POST /api/v1/auth/login
    │
    ▼
Receive { access_token, refresh_token, expires_in, user }
    │
    ├── Login Hook calls authStore.setToken(access_token, refresh_token)
    ├── Persist to localStorage
    └── Set axios default header Authorization
    │
    ▼
Navigate to /dashboard
    │
    ▼
Subsequent requests automatically carry Bearer Token
```

### 9.2 Token Refresh Mechanism

```
API request → 401 response
    │
    ▼
Freeze subsequent requests (queue)
    │
    ▼
Request POST /api/v1/auth/refresh { refresh_token }
    │
    ├── Success → authStore.setToken(new_access, new_refresh)
    │             → Replay queued requests → continue
    │
    └── Failure → authStore.logout() → navigate to /login
```

### 9.3 Route-Level Access Control

```typescript
// ProtectedRoute core logic
function ProtectedRoute({ route }: { route: RouteConfig }) {
  const { isAuthenticated, user } = useAuthStore();

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  const requiredRoles = route.permissions;
  if (requiredRoles && !requiredRoles.includes(user!.role)) {
    return <ForbiddenPage />;  // 403 page
  }

  return <Outlet />;
}
```

### 9.4 Menu-Level Access Control

```typescript
// Filter menu by role in Sidebar
const menuItems = menuRoutes
  .filter(route => !route.hideInMenu)
  .filter(route => route.permissions?.includes(user.role))
  .map(route => ({
    key: route.path,
    icon: route.icon ? <route.icon /> : null,
    label: t(route.titleKey),
    children: route.children?.filter(child => child.permissions?.includes(user.role)),
  }));
```

---

## 10. Internationalization

### 10.1 i18next Initialization

```typescript
// src/i18n/index.ts
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import zhCommon from './resources/zh/common.json';
import enCommon from './resources/en/common.json';
// ... other namespaces

i18n.use(initReactI18next).init({
  resources: {
    zh: {
      common: zhCommon,
      pipes: zhPipes,
      inventory: zhInventory,
      quality: zhQuality,
      purchase: zhPurchase,
      sales: zhSales,
      system: zhSystem,
      validation: zhValidation,
    },
    en: {
      common: enCommon,
      pipes: enPipes,
      inventory: enInventory,
      // ...
    },
  },
  lng: localStorage.getItem('language') || 'zh',
  fallbackLng: 'zh',
  ns: ['common', 'pipes', 'inventory', 'quality', 'purchase', 'sales', 'system', 'validation'],
  defaultNS: 'common',
  interpolation: {
    escapeValue: false,  // React already handles XSS protection
  },
});
```

### 10.2 Translation File Examples

```json
// i18n/resources/zh/pipes.json
{
  "pipe_number": "管材编号",
  "grade": "钢级",
  "od": "外径",
  "wt": "壁厚",
  "length": "长度",
  "end_type": "端部类型",
  "heat_number": "炉批号",
  "status": {
    "in_stock": "在库",
    "outbound": "已出库",
    "scrapped": "报废"
  }
}

// i18n/resources/en/pipes.json
{
  "pipe_number": "Pipe Number",
  "grade": "Grade",
  "od": "O.D.",
  "wt": "W.T.",
  "length": "Length",
  "end_type": "End Type",
  "heat_number": "Heat Number",
  "status": {
    "in_stock": "In Stock",
    "outbound": "Outbound",
    "scrapped": "Scrapped"
  }
}
```

### 10.3 Hook Usage

```typescript
// Usage in components
import { useTranslation } from 'react-i18next';

function PipeTable() {
  const { t, i18n } = useTranslation('pipes');

  return (
    <Table columns={[
      { title: t('pipe_number'), dataIndex: 'pipe_number' },
      { title: t('grade'), dataIndex: 'grade' },
      { title: t('status.in_stock'), dataIndex: 'status' },
    ]} />
  );
}
```

---

## 11. Ant Design 5 Theme Customization

### 11.1 Industrial Theme Configuration

```typescript
// src/styles/theme.ts
import type { ThemeConfig } from 'antd';

const theme: ThemeConfig = {
  token: {
    // Brand color - industrial blue, representing professionalism and reliability
    colorPrimary: '#1B3A5C',         // Deep sea blue
    colorInfo: '#1B3A5C',
    colorSuccess: '#389E0D',         // Jungle green - for "in stock" status
    colorWarning: '#D48806',         // Golden yellow - for warnings
    colorError: '#CF1322',           // Red - for errors/scrapped

    // Neutral colors
    colorBgLayout: '#F0F2F5',        // Page background
    colorBgContainer: '#FFFFFF',     // Card background
    colorText: '#1A1A1A',            // Body text
    colorTextSecondary: '#595959',   // Secondary text

    // Fonts - use system fonts, no extra font packages
    // Chinese fallback: Microsoft YaHei / PingFang SC; English: SF Pro / Segoe UI
    fontFamily: `-apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif`,
    fontFamilyCode: `'SF Mono', 'Cascadia Code', 'Consolas', 'Courier New', monospace`,

    // Border radius
    borderRadius: 6,
    borderRadiusLG: 8,

    // Spacing
    margin: 16,
    padding: 16,
    paddingContentHorizontal: 24,
    paddingContentVertical: 16,

    // Tables
    tableHeaderBg: '#F5F7FA',
    tableRowHoverBg: '#E6F0FF',
  },
  components: {
    Menu: {
      // Sidebar menu
      itemBg: 'transparent',
      subMenuItemBg: 'transparent',
      itemColor: '#FFFFFF',
      itemHoverColor: '#FFFFFF',
      itemHoverBg: 'rgba(255,255,255,0.12)',
      itemSelectedColor: '#FFFFFF',
      itemSelectedBg: 'rgba(255,255,255,0.2)',
      iconSize: 18,
    },
    Table: {
      // Data tables - compact, information-dense
      headerBg: '#F5F7FA',
      headerColor: '#1A1A1A',
      headerBorderRadius: 6,
      rowHoverBg: '#E6F0FF',
      cellFontSize: 13,
    },
    Form: {
      labelColor: '#262626',
      verticalLabelMargin: 4,
    },
    Card: {
      paddingLG: 20,
    },
    Button: {
      primaryShadow: '0 2px 0 rgba(27,58,92,0.1)',
    },
  },
};

export default theme;
```

### 11.2 Sidebar Dark Theme

The sidebar uses a dark theme to emphasize navigation and create a clear visual separation from the content area:

```
Sider Background:  #0F1A2E (deep navy blue)
Menu Text:         #FFFFFF / rgba(255,255,255,0.65)
Active Indicator:  #3B82F6 (bright blue highlight)
Divider:           rgba(255,255,255,0.1)
```

### 11.3 Card & Page Container Specifications

| Element | Style |
|---------|-------|
| **Page Container** | White background, 12px padding, 6px border radius |
| **Filter Bar** | Gray background `#FAFAFA`, 16px padding, 16px bottom spacing |
| **Table** | Compact mode `size="small"`, optional zebra striping |
| **Form Card** | Grouped with Card wrapper, each group title 14px bold |
| **Statistic Cards** | Ant Design `Statistic` component, icon on left + value on right |

### 11.4 Status Color Mapping

| Status | Color | Ant Design Token |
|--------|-------|-----------------|
| In Stock | Green | `colorSuccess` |
| Outbound | Orange | `colorWarning` |
| Scrapped | Red | `colorError` |
| Draft | Gray | `default` |
| Pending | Gold | `gold` |
| Approved | Blue | `blue` |
| Completed | Green | `colorSuccess` |
| Cancelled | Gray | `#D9D9D9` |

---

## 12. Frontend Data Flow Overview

```
User Action (click/input/submit)
    │
    ▼
React Component
    │
    ├── Client state change → Zustand Store → subscribed components re-render
    │
    └── Server data request → TanStack Query Hook
            │
            ▼
        API Function (pipeApi.getSeamlessPipes)
            │
            ▼
        Axios Interceptor (inject token)
            │
            ▼
        HTTP Request (JSON)
            │
            ▼
        Backend API (/api/v1/seamless-pipes)
            │
            ▼
        HTTP Response (JSON)
            │
            ▼
        Axios Interceptor (unified error handling)
            │
            ▼
        TanStack Query (cache + state management)
            │
            ├── loading → component shows Skeleton/Spin
            ├── success → component shows data
            └── error → component shows error notification

Data mutation operation (create/update/delete)
    │
    ▼
TanStack Query Mutation
    │
    ▼
API request → success
    │
    ▼
queryClient.invalidateQueries  →  list page auto-refresh
```

---

## 13. Implementation Recommendations

### 13.1 Implementation Order

| Phase | Content | Effort Estimate |
|-------|---------|----------------|
| **Phase 1** | Project scaffolding + layout + routing + authentication | 2-3 days |
| **Phase 2** | Pipe management (list/form/detail) + search | 3-4 days |
| **Phase 3** | Inventory management (inbound/outbound/stock query/locations) | 3-4 days |
| **Phase 4** | Quality management (certificates/traceability/API 5CT reference) | 2-3 days |
| **Phase 5** | Purchase & sales management (orders/suppliers/customers) | 3-4 days |
| **Phase 6** | Data import/export + system management (users/logs) | 2-3 days |
| **Phase 7** | Reports & statistics + Dashboard | 2-3 days |
| **Phase 8** | Internationalization refinement + unit switching + label printing | 2 days |

### 13.2 Development Key Points

1. **Use Vite Proxy for CORS**: Proxy `/api` requests to the backend during development; **do not use React Router loaders** (all data dependencies are fetched within components via TanStack Query)
2. **Mock API**: Use MSW (Mock Service Worker) for independent frontend development (see 13.3)
3. **TypeScript Strict Mode**: All API responses are typed, with Zod runtime validation for critical data
4. **Ant Design Table Performance**: Use `virtualized` rendering or pagination for large datasets
5. **Unified Error Handling**: Axios interceptor (with 401 auto-refresh) + `ErrorBoundary` (catches render errors, shows fallback UI with retry) + Ant Design `message` / `notification`
6. **Label Printing**: Backend generates PDF (`printpdf` crate), frontend downloads PDF via `POST /api/v1/pipes/{id}/print-label` then triggers browser print

### 13.3 Mock API Design (MSW)

```typescript
// src/mocks/handlers.ts — example
import { http, HttpResponse } from 'msw';
import { faker } from '@faker-js/faker/locale/zh_CN';

// Simulate paginated response
function paginated<T>(items: T[], page: number, pageSize: number) {
  const start = (page - 1) * pageSize;
  return {
    success: true,
    data: items.slice(start, start + pageSize),
    meta: { page, page_size: pageSize, total: items.length },
    request_id: crypto.randomUUID(),
  };
}

export const handlers = [
  // Seamless pipe list
  http.get('/api/v1/seamless-pipes', ({ request }) => {
    const url = new URL(request.url);
    const page = Number(url.searchParams.get('page')) || 1;
    const pageSize = Number(url.searchParams.get('page_size')) || 20;
    const pipes = Array.from({ length: 53 }, (_, i) => ({
      id: i + 1,
      pipe_number: `J55 4.500in×11.60lb SC-H2405-${String(i + 1).padStart(6, '0')}`,
      grade: faker.helpers.arrayElement(['J55', 'N80', 'L80', 'P110']),
      od: Number(faker.number.float({ min: 4, max: 9, fractionDigits: 3 })),
      wt: Number(faker.number.float({ min: 0.2, max: 0.6, fractionDigits: 3 })),
      length: Number(faker.number.float({ min: 8, max: 13, fractionDigits: 2 })),
      status: faker.helpers.arrayElement(['in_stock', 'out_stock', 'qc_hold']),
      heat_number: `HT${faker.date.recent({ days: 90 }).toISOString().slice(0, 10).replace(/-/g, '')}-${String(Math.floor(i / 20) + 1).padStart(2, '0')}`,
    }));
    return HttpResponse.json(paginated(pipes, page, pageSize));
  }),

  // Authentication
  http.post('/api/v1/auth/login', async ({ request }) => {
    const body = await request.json() as any;
    if (body.username === 'admin' && body.password === 'admin123') {
      return HttpResponse.json({
        success: true,
        data: {
          access_token: `mock_access_${Date.now()}`,
          refresh_token: `mock_refresh_${Date.now()}`,
          expires_in: 3600,
          user: { id: 1, username: 'admin', role: 'admin', display_name: 'Administrator' },
        },
        request_id: crypto.randomUUID(),
      });
    }
    return HttpResponse.json(
      { success: false, error: { code: 'AUTH_INVALID_CREDENTIALS', message: 'Invalid username or password' } },
      { status: 401 }
    );
  }),

  // Token refresh (mock success)
  http.post('/api/v1/auth/refresh', () => {
    return HttpResponse.json({
      success: true,
      data: {
        access_token: `mock_refreshed_access_${Date.now()}`,
        refresh_token: `mock_refreshed_refresh_${Date.now()}`,
        expires_in: 3600,
      },
      request_id: crypto.randomUUID(),
    });
  }),

  // More handlers...
];

// src/mocks/browser.ts
import { setupWorker } from 'msw/browser';
export const worker = setupWorker(...handlers);

// src/main.ts — enable MSW in development
if (import.meta.env.DEV) {
  const { worker } = await import('./mocks/browser');
  await worker.start({ onUnhandledRequest: 'bypass' });
}
```

> **Mock Development Principles**:
> - MSW is only enabled under `import.meta.env.DEV`, does not affect production builds
> - Use `@faker-js/faker` to generate realistic Chinese data
> - Mock handlers strictly correspond to API types (share `types.ts`)
> - During implementation, gradually replace mock handlers with real APIs without modifying component code

---

## Appendix: Key Third-Party Library Quick Reference

| Purpose | Library | Notes |
|---------|---------|-------|
| Barcode | `jsbarcode` | Generate Code128 / EAN barcodes |
| QR Code | `qrcode.react` | Generate QR codes |
| Excel Export | `xlsx` (SheetJS) | Generate Excel files directly in the browser |
| Date Formatting | `dayjs` (built into Ant Design) | — |
| Drag & Drop Sort | `@dnd-kit/core` | For table drag-sorting if needed |
| Tree Select | Ant Design TreeSelect | Location selection |
| Virtual Scrolling | `rc-virtual-list` (built into Ant Design) | Table optimization for large datasets |
