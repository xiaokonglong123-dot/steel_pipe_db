# Phase 1 — Frontend: Pipe Management Module (P0 MVP)

> Based on: `docs/frontend-design.en.md` §2, §3, §4.2, §6, §8

## Tasks

### 1.1 Project Setup
- [ ] Init Vite + React 19 + TypeScript project (`npm create vite`)
- [ ] Install core deps: antd 5, @ant-design/icons, react-router-dom 7, @tanstack/react-query 5, zustand 5, axios, react-i18next, i18next, dayjs, zod
- [ ] Configure `vite.config.ts` (API proxy to `/api/v1`)
- [ ] Configure TypeScript strict mode + path alias `@/`
- [ ] Configure ESLint + Prettier

### 1.2 Infrastructure
- [ ] Create `src/api/client.ts`: Axios instance + request interceptor (inject JWT) + response interceptor (401 refresh token)
- [ ] Create `src/api/queryClient.ts`: TanStack Query Client config (staleTime: 2min, gcTime: 5min)
- [ ] Create `src/i18n/index.ts`: i18next init (zh/en resource file skeleton)
- [ ] Create i18n resource files: `zh/common.json`, `en/common.json`, `zh/pipes.json`, `en/pipes.json`
- [ ] Create `src/styles/theme.ts`: Ant Design 5 industrial blue theme config
- [ ] Create `src/styles/global.less`: global style overrides

### 1.3 Shared Components
- [ ] Implement `PageContainer`: white card container, unified padding
- [ ] Implement `PageHeader`: page title + breadcrumb + right action buttons
- [ ] Implement `ErrorBoundary`: React Error Boundary
- [ ] Implement `ConfirmModal`: confirmation dialog (delete/cancel operations)
- [ ] Implement `LoadingSpin`: global / area loading state
- [ ] Implement `EmptyState`: empty data placeholder

### 1.4 Pipe Shared Types & API
- [ ] Define `features/pipes/types.ts`: SeamlessPipe, ScreenPipe, PipeFilter, PaginatedResponse etc.
- [ ] Define `features/pipes/api/pipeApi.ts`:
  - `getSeamlessPipes(filters, page, pageSize)`
  - `getSeamlessPipe(id)`
  - `createSeamlessPipe(data)`
  - `updateSeamlessPipe(id, data)`
  - `deleteSeamlessPipe(id)`
  - `getScreenPipes(...)`, `getScreenPipe(...)`, `createScreenPipe(...)`, `updateScreenPipe(...)`, `deleteScreenPipe(...)`
  - `searchPipes(query)`
- [ ] Implement `hooks/useSeamlessPipes.ts` (React Query hooks: useSeamlessPipes, useSeamlessPipe, useCreateSeamlessPipe, useUpdateSeamlessPipe, useDeleteSeamlessPipe)
- [ ] Implement `hooks/useScreenPipes.ts` (same deal, for screen pipes)

### 1.5 Pipe Shared Components
- [ ] Implement `PipeFilterBar`: grade dropdown, OD/WT range inputs, status dropdown, location dropdown, search box, query/reset buttons
- [ ] Implement `PipeTable`: Ant Design Table wrapper (with pagination, sorting, action column)
- [ ] Implement `GradeTag`: grade label component (color-coded by grade group)
- [ ] Implement `PipeStatusBadge`: stock status badge (In Stock 🟢 / Outbound 🔵 / Scrapped 🔴)
- [ ] Implement `PipeDetailCard`: pipe detail info card

### 1.6 Seamless Pipe Pages
- [ ] Implement `SeamlessPipeListPage`:
  - Top action bar (+New, Import, Export buttons)
  - FilterBar + PipeTable combo
  - Row actions: view detail, edit, delete
  - Batch select + batch operations
- [ ] Implement `SeamlessPipeFormPage`:
  - Ant Design Form, sections: basic info, coupling info, production info
  - Cascading field selects (grade cascade, spec auto-fills reference weight)
  - Zod form validation
  - Submit + cancel buttons
- [ ] Implement `SeamlessPipeDetailPage`:
  - PipeDetailCard showing all fields
  - Linked inbound/outbound records Tab
  - Operation log Tab
  - Edit/delete buttons

### 1.7 Screen Pipe Pages
- [ ] Implement `ScreenPipeListPage` (same structure as seamless list, add screen-specific filters)
- [ ] Implement `ScreenPipeFormPage` (with screen-specific fields: screen type, slot width, filtration precision, base pipe params)
- [ ] Implement `ScreenPipeDetailPage` (show all screen pipe fields + linked records)

### 1.8 Unified Search Page
- [ ] Implement `UnifiedPipeSearchPage`:
  - Global search box (fuzzy search by pipe number / grade / heat number)
  - Results split by Tab (Seamless Pipes / Screen Pipes)
  - Click result to navigate to detail page

### 1.9 Constants & Utility Functions
- [ ] Implement `shared/utils/constants.ts`: grade list, end type list, screen type list
- [ ] Implement `shared/utils/format.ts`: date formatting, number formatting, enum value mapping
- [ ] Implement `shared/utils/pipe-number.ts`: pipe number generation/parsing utilities

> **Deps**: Infrastructure (Axios, QueryClient, i18n, theme)
