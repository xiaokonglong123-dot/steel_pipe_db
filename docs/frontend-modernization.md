# Frontend Modernization

## Completed

### Phase 1: Remove Axios, use native fetch + TanStack Query ✅

- Replaced Axios instance with native fetch wrapper (`src/api/client.ts`)
- Updated all 15 API files to use fetch
- Removed axios dependency
- Created custom `ApiError` class for structured error handling

### Phase 2: React 19 Features ✅

- Created `useOptimisticMutation` hook for instant UI feedback
- Created `useTransitionMutation` hook for non-blocking updates
- Both hooks are ready for use in feature pages

### Phase 3: Component Improvements ✅

- Created `DataTable` — Ant Design Table wrapper with loading, empty, and pagination
- Created `PageLayout` — Page layout with header, breadcrumbs, and content
- Created `FormField` — Form field wrapper with label, help text, and validation
- Created `ActionButton` — Button with loading, confirm, and tooltip
- Created `StatusBadge` — Status badge with color coding and icons

### Phase 4: Bundle Optimization ✅

- Split `vendor-ui` into separate chunks:
  - `vendor-react` (297 kB) — React core
  - `vendor-antd` (1,104 kB) — Ant Design
  - `vendor-ui` (38 kB) — TanStack + Zustand
  - `vendor-utils` (117 kB) — Zod + dayjs + i18next
- Total bundle size reduced by ~50 kB

## Verification

- ✅ `npx tsc --noEmit` — 0 errors
- ✅ `npm run build` — Build successful
- ✅ `cargo check` — Backend check passed

## Next Steps

1. **Migrate existing pages** to use new shared components (DataTable, PageLayout, etc.)
2. **Add React 19 features** to more pages (useOptimistic, useTransition)
3. **Add lazy loading** for route-level code splitting
4. **Add error boundaries** to more feature modules
