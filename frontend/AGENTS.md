# Frontend — React 19 Package

## Tech Stack
- **React 19** — UI library
- **Vite** — Build tool (vanilla-ts template)
- **TypeScript** — Strict mode
- **Ant Design 5** — UI components
- **TanStack Query (React Query 5)** — Server state
- **react-router-dom v7** — Routing
- **axios** — HTTP client
- **i18next / react-i18next** — i18n (zh-CN primary, en-US fallback)
- **dayjs** — Date handling
- **zod** — Schema validation

## Build & Dev
```bash
cd frontend
npm install        # Install dependencies (including vite, antd, etc.)
npm run dev        # Dev server on http://localhost:5173
npm run build      # Production build to dist/
npm run lint       # ESLint
npm run preview    # Preview production build
```

## Package Architecture

```
frontend/
├── public/              ← Static assets
├── src/
│   ├── main.tsx         ← React entry, i18n init, QueryClient setup
│   ├── App.tsx          ← RouterProvider setup, ConfigProvider + QueryClient
│   ├── api/             ← Shared: axios instance, interceptors
│   ├── lib/             ← validateResponse.ts, runtime zod response validation
│   ├── stores/          ← Zustand stores (authStore, appStore, unitStore)
│   ├── i18n/            ← Translation resources (zh, en) — 15 namespaces
│   ├── routes/          ← Route definitions (react-router)
│   ├── shared/          ← Shared: components, hooks
│   │   ├── components/  ← ConfirmModal, EmptyState, ErrorBoundary, FileUploader, LoadingSpin, PageContainer, PageHeader, SearchBar, StatusTag
│   │   └── hooks/       ← useDebounce
│   ├── theme/           ← Ant Design theme config
│   ├── zod-schemas/     ← 7 Zod schema files for API response validation
│   ├── utils/           ← Utility functions
│   └── features/        ← Feature modules (see features/AGENTS.md)
│       ├── auth/
│       ├── pipes/
│       ├── inventory/
│       ├── suppliers/
│       ├── customers/
│       ├── purchases/
│       ├── sales/
│       ├── quality/
│       ├── contracts/
│       ├── reports/
│       ├── labels/
│       ├── search/
│       └── profile/
├── index.html
├── vite.config.ts       ← React plugin, proxy, manualChunks vendor splitting
├── tsconfig.json        ← Strict TypeScript config
├── .eslintrc.cjs        ← ESLint config
├── .prettierrc          ← Prettier config (singleQuote, 2 space, noBracketSpacing)
└── package.json
```

## Key Dependencies (from package.json)
- `react`, `react-dom` (^19)
- `antd` (^5) — UI library
- `@tanstack/react-query` (^5) — Server state
- `react-router-dom` (^7) — Client routing
- `axios` (^1) — HTTP client
- `i18next`, `react-i18next` — i18n
- `dayjs` — Date utilities
- `zod` — Schema validation

## Conventions
- Feature-based organization under `src/features/`
- All API calls go through `@tanstack/react-query` hooks (no direct fetch in components)
- i18n namespace per feature (15 total across zh/ and en/): common, pipes, inventory, purchase, sales, quality, contracts, suppliers, customers, reports, labels, profile, search, system, validation
- Ant Design components + theme config in `src/theme/`
- Vite dev proxy: `/api/*` → `http://localhost:3000`
- TypeScript strict mode enabled
- No `as any`, `@ts-ignore`, or `@ts-expect-error` allowed
- Vendor chunk splitting in vite.config.ts: antd→vendor-antd, react ecosystem→vendor-react, utils→vendor-utils, app→index (~162 kB gzip)

## Key Files
- `vite.config.ts` — Vite config (React plugin, proxy, manualChunks vendor splitting)
- `tsconfig.json` — TypeScript config (strict, JSX react-jsx)
- `.eslintrc.cjs` — ESLint rules
- `.prettierrc` — `singleQuote: true, tabWidth: 2, bracketSpacing: false`
