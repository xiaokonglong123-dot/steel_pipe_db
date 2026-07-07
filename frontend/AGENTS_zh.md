# frontend — React 19

Quick orientation: React 19 + Vite + Ant Design 5 + TanStack Query 5. Strict TypeScript, no shortcuts.

## Tech Stack

- **React 19** — UI
- **Vite** — Build tool (vanilla-ts template)
- **TypeScript** — Strict mode. No `as any`, no `@ts-ignore`, no `@ts-expect-error`.
- **Ant Design 5** — UI components
- **TanStack Query 5** — Server state (staleTime: 2min, gcTime: 5min)
- **react-router-dom v7** — Routing (createBrowserRouter)
- **axios** — HTTP client
- **i18next / react-i18next** — i18n (zh-CN primary, en-US fallback)
- **dayjs** — Date handling
- **zod** — Validation + runtime response checking

## Build & Dev

```bash
cd frontend
npm install          # Install deps
npm run dev          # Dev server → http://localhost:5173
npm run build        # Production → dist/
npm run lint         # ESLint
npm run preview      # Preview build
```

## Project Layout

```
frontend/
├── public/
├── src/
│   ├── main.tsx         ← Entry: i18n init, QueryClient, render
│   ├── App.tsx          ← ConfigProvider + QueryClientProvider + RouterProvider
│   ├── api/             ← Shared axios instance + interceptors
│   ├── lib/             ← validateResponse.ts, runtime Zod validation
│   ├── stores/          ← Zustand (authStore, appStore, unitStore)
│   ├── i18n/            ← Translations (zh, en) — 15 namespaces
│   ├── routes/          ← Route definitions
│   ├── shared/          ← Shared components & hooks
│   │   ├── components/  ← ConfirmModal, EmptyState, ErrorBoundary, FileUploader, LoadingSpin, PageContainer, PageHeader, SearchBar, StatusTag
│   │   └── hooks/       ← useDebounce
│   ├── theme/           ← Ant Design theme config
│   ├── zod-schemas/     ← 7 Zod schemas for API response validation
│   ├── utils/           ← Utilities
│   └── features/        ← Feature modules
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
- `axios` (^1)
- `i18next`, `react-i18next`
- `dayjs`
- `zod`

## Conventions

- Feature-based structure under `src/features/`. Each feature owns its own API hooks, pages, types.
- Every API call goes through TanStack Query hooks — no raw `fetch` in components.
- i18n namespace per feature (15 total): common, pipes, inventory, purchase, sales, quality, contracts, suppliers, customers, reports, labels, profile, search, system, validation.
- Ant Design theme config in `src/theme/`.
- Vite dev proxy: `/api/*` → `http://localhost:3000`.
- TypeScript strict mode. `as any` and suppression comments are banned.
- Vendor chunk splitting: React, Ant Design, TanStack Query, Zustand, i18next, and dayjs are grouped into `vendor-ui` to avoid circular chunk warnings.
- Feature API hooks use local `queryKeys.ts` factories; do not add inline `queryKey: [...]` literals in feature API code.

## Key Files

- `vite.config.ts` — React plugin, proxy, manualChunks
- `tsconfig.json` — Strict, JSX react-jsx
- `eslint.config.js` — ESLint 9 flat config
- `.prettierrc` — `singleQuote: true, tabWidth: 2, bracketSpacing: false`
