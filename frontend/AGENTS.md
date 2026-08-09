# frontend — React 19

What you need to know: this is the frontend of the **ERP（通用企业资源计划系统）** monorepo, built with React 19 + Vite + Ant Design 5. TypeScript strict mode, no exceptions. The backend crate is `erp-server` (code-phase target); the backend data layer is **SQLite3** (`sqlite://data/erp.db?mode=rwc`, sqlx 0.8 sqlite feature).

## Tech Stack

- **React 19** — UI layer
- **Vite** — Build tool (vanilla-ts template)
- **TypeScript** — Strict. No `as any`, no `@ts-ignore`, no `@ts-expect-error`.
- **Ant Design 5** — UI components
- **TanStack Query 5** — Server state (2min staleTime, 5min gcTime)
- **react-router-dom v7** — Routing (createBrowserRouter)
- **Native `fetch` wrapper** — HTTP client (`src/api/client.ts`)
- **i18next / react-i18next** — i18n (zh-CN primary, en-US fallback)
- **dayjs** — Dates
- **zod** — Schema validation + runtime response checking

## Build & Dev

```bash
cd frontend
npm install          # Install everything
npm run dev          # Dev server → http://localhost:5173
npm run build        # Production build → dist/
npm run lint         # ESLint
npm run preview      # Preview the production build
```

## Project Layout

```
frontend/
├── public/
├── src/
│   ├── main.tsx         ← Boot: i18n init, QueryClient, render
│   ├── App.tsx          ← ConfigProvider + QueryClientProvider + RouterProvider
│   ├── api/             ← Native fetch wrapper + QueryClient config
│   ├── lib/             ← validateResponse.ts, runtime Zod validation
│   ├── stores/          ← Zustand stores (authStore, appStore)
│   ├── i18n/            ← Translation files (zh, en) — per-feature namespaces
│   ├── routes/          ← Route definitions
│   ├── shared/          ← Shared components & hooks
│   │   ├── components/  ← ActionButton, Can, DataTable, EmptyState, ErrorBoundary, FormField, ItemPicker, ListPageTemplate, PageLayout, RouteBoundary, SearchBar, StatusBadge, StatusTag
│   │   └── hooks/       ← useDebounce
│   ├── theme/           ← Ant Design theme config
│   ├── zod-schemas/     ← Zod schemas for API response validation
│   ├── utils/           ← Utility functions
│   └── features/        ← Feature modules (see features/AGENTS.md)
│       ├── auth/            ← Login, RBAC (users/roles/departments)
│       ├── inventory/       ← 商品/Item + SKU master data, 库存 (inbound/outbound/stock/locations/check)
│       ├── inventory_atp/   ← 库存预留 (ATP)
│       ├── suppliers/       ← 供应商
│       ├── customers/       ← 客户
│       ├── purchases/       ← 采购订单
│       ├── sales/           ← 销售订单
│       ├── sales_crm/       ← 客户信用/发货
│       ├── contracts/       ← 采购/销售合同
│       ├── manufacturing/   ← BOM/工单/质检 (Inspection)
│       ├── project/         ← 项目/WBS/预算
│       ├── assets/          ← 固定资产
│       ├── hr/              ← 员工/考勤/薪资/劳动合同
│       ├── finance/         ← 会计科目/日记账/发票/付款
│       ├── procurement/     ← 采购申请/采购收货/采购报价/供应商评分
│       ├── workflow/        ← 审批流定义/实例/任务
│       ├── notification/    ← 通知
│       ├── portal/          ← 门户账户
│       ├── reports/         ← 明细报表
│       ├── bi/              ← BI 分析看板
│       ├── search/          ← 全局商品搜索
│       ├── data-io/         ← 通用数据导入导出
│       ├── quality/         ← 质检证书 (certs)
│       ├── labels/          ← 标签打印
│       └── profile/
├── index.html
├── vite.config.ts       ← React plugin, proxy, vendor-ui manual chunk
├── tsconfig.json
├── eslint.config.js     ← ESLint 9 flat config
├── .prettierrc
└── package.json
```

## Key Dependencies

- `react`, `react-dom` (^19)
- `antd` (^5)
- `@tanstack/react-query` (^5)
- `react-router-dom` (^7)
- `i18next`, `react-i18next`
- `dayjs`
- `zod`

## Conventions

- Feature-based structure under `src/features/`. Each feature owns its API hooks, pages, types, etc.
- All API calls go through TanStack Query hooks — no raw `fetch` in components.
- i18n namespace per feature (auth, inventory, purchase, sales, contracts, suppliers, customers, manufacturing, project, assets, hr, finance, procurement, workflow, notification, portal, reports, bi, search, profile, data-io, plus common/system/validation).
- Ant Design theme lives in `src/theme/`.
- Vite dev proxy: `/api/*` → `http://localhost:3000`.
- TypeScript strict. `as any` and suppression comments are banned.
- Vendor chunk splitting in `vite.config.ts`: React, Ant Design, TanStack Query, Zustand, i18next, and dayjs are grouped into `vendor-ui` to avoid circular chunk warnings.
- Feature API hooks use local `queryKeys.ts` factories; do not add inline `queryKey: [...]` literals in feature API code.

## Key Files

- `vite.config.ts` — React plugin, proxy config, manualChunks splitting
- `tsconfig.json` — Strict mode, JSX react-jsx
- `eslint.config.js` — ESLint 9 flat config
- `.prettierrc` — `singleQuote: true, tabWidth: 2, bracketSpacing: false`
