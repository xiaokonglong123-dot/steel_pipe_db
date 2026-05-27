# Steel Pipe DB — Project Index (Dev Onboarding Edition)

> Yeah, this was the Chinese version. Now it's English too, but it covers different angles than AGENTS.md — more about conventions, architecture layers, and where to find things. Consider this the "dev onboarding" doc.

## Architecture at a Glance

Rust + React 19 monorepo for API 5CT steel pipe inventory management. Oil & gas stuff.

```
steel-pipe-db/
├── backend/          ← Rust Axum 0.8 REST API (SQLite, JWT/Argon2)
│   └── src/
│       ├── handlers/     ← 13 files, HTTP glue: validate → call service → respond
│       ├── services/     ← 12 files, business logic (unit structs, static methods — no DI magic)
│       ├── repositories/ ← 13 files, pure SQL queries via SQLx
│       ├── models/       ← 11 files, DB row structs with sqlx::FromRow
│       ├── dto/          ← 14 files, request/response types with serde + validator
│       ├── domain/       ← 4 files, enums and domain types
│       ├── middleware/   ← auth.rs + rbac.rs (includes success/request_id in error responses)
│       ├── router.rs     ← ~70 endpoints wired up
│       ├── config.rs     ← env-based config (DATABASE_URL, JWT_SECRET, etc.)
│       ├── error.rs      ← AppError enum with numeric codes; ApiErrorResponse includes success + request_id
│       └── response.rs   ← ApiResponse<T>, PaginatedResponse<T>, Meta struct, request_id (uuid v4)
├── frontend/         ← React 19 + Vite + Ant Design + TanStack Query
│   └── src/
│       ├── main.tsx       ← React DOM entry (imports './i18n' for i18n init)
│       ├── App.tsx        ← ConfigProvider + QueryClientProvider + RouterProvider
│       ├── features/      ← 13 feature modules (auth, pipes, inventory, suppliers, customers, search, profile, ...)
│       │                   ├── api/       ← TanStack Query hooks (useQuery, useMutation)
│       │                   ├── hooks/     ← feature-specific React hooks
│       │                   ├── pages/     ← route-level page components
│       │                   └── types/     ← TypeScript interfaces
│       ├── layouts/       ← MainLayout (sidebar + search + profile settings)
│       ├── stores/        ← Zustand authStore, appStore (global state), unitStore (unit conversion)
│       ├── routes/        ← createBrowserRouter + ProtectedRoute
│       ├── lib/           ← validateResponse.ts, runtime Zod response validation
│       ├── shared/        ← hooks (useDebounce), components/ (9 shared: ConfirmModal, etc.)
│       ├── zod-schemas/   ← 7 Zod schema files for runtime response validation
│       └── i18n/          ← react-i18next (zh-CN primary); namespaces: inventory, pipes, profile, etc.
└── docs/             ← design docs, task breakdowns
```

## Build & Run

**Quick note**: No Makefile here. Use the raw commands.

| Command | What it does |
|---------|-------------|
| `cd backend && cargo check` | Backend type-check (faster than a full build) |
| `cd backend && cargo test` | Run backend tests |
| `cd backend && cargo run` | Fire up the backend dev server |
| `cd frontend && npm run dev` | Fire up the frontend dev server |
| `cd frontend && npx tsc --noEmit` | Frontend type-check |
| `cd frontend && npm run build` | Production frontend build |
| Frontend chunk analysis | `cd frontend && npx vite build --analyze` (manualChunks in vite.config.ts) |
| Full CI pipeline | `cargo check` + `tsc --noEmit` + `npm run build` (runs in parallel) |

Backend: `http://localhost:3000`, Frontend: `http://localhost:5173`.

## Tech Stack

### Backend
- **Rust** — nightly toolchain 2024-02-08, edition 2021
- **Axum 0.8** — HTTP framework (macros + multipart features enabled)
- **SQLx 0.8** — async SQL driver (SQLite, runtime-tokio-rustls, chrono)
- **SQLite** — zero-config DB, no external service needed
- **JWT** (jsonwebtoken 9) — auth tokens
- **Argon2 0.5** — password hashing (NOT bcrypt, we have standards)
- **validator 0.19** — request validation with derive macros
- **tracing** + tracing-subscriber — structured logging (env-filter, json output)
- **tower-http 0.6** — CORS, trace, request-id middleware
- **calamine 0.26** — Excel import, **rust_xlsxwriter 0.80** — Excel export, **csv 1.3** — CSV
- **No `rust_decimal` / `bigdecimal`** — decimals are f64 for now
- **No `build.rs`** — that subordinate AGENTS.md was wrong about this

### Frontend
- **React 19** — UI framework (react-router-dom v7, createBrowserRouter)
- **Vite** — build tool
- **TypeScript strict** — noUnusedLocals, noUnusedParameters enforced
- **Ant Design 5** — UI component library (+ @ant-design/icons)
- **TanStack Query 5** — server state (staleTime: 2min, gcTime: 5min)
- **Zustand 5** — client state (auth, global, unit conversion)
- **react-router-dom v7** — routing
- **axios** — HTTP client (baseURL `/api/v1`, auto Bearer token)
- **i18next / react-i18next** — i18n (zh-CN primary, per-feature namespaces)
- **dayjs** — date handling
- **zod** — schema validation
- **zod runtime validation** — `src/lib/validateResponse.ts` wraps `zod.response()` for API response checking

## Core Conventions

### Backend Layered Pattern
```
Request → Handler (validate) → Service (business logic) → Repository (SQL) → DB
                                                                                   ↓
Response ← Handler (format)  ← Service (orchestrate)    ← Repository (results) ←
```

- **Handlers**: Extract params, call one service method, return `ApiResponse<T>` | error
- **Services**: Business logic orchestration, cross-entity ops, transaction management
- **Repositories**: Pure SQL, single-entity CRUD, zero business logic
- **Models**: Row structs matching DB schema, no methods
- **DTOs**: Request validation structs (serde::Deserialize + validator), response structs

### Frontend Feature Pattern
```
features/{feature}/
├── api/      ← TanStack Query hooks (useQuery, useMutation)
├── hooks/    ← feature-specific React hooks
├── pages/    ← page components (1 file = 1 route)
└── types/    ← TypeScript interfaces
```

- **Dead code cleanup**: 26 unused items removed from domain/dto/error/response/repo modules. `#![allow(dead_code)]` stays at the crate root to suppress legit false positives.
- **Backend DI**: DB pool is injected as `Extension<SqlitePool>`; auth uses `Extension<JwtSecret>` newtype, not bare `Extension<String>`.
- **Request IDs**: response bodies include `request_id`; the backend also emits/exposes `x-request-id` headers via `tower-http`.
- **Frontend query keys**: feature API hooks use per-feature `queryKeys.ts` factories instead of inline query-key array literals.

## Security Rules
- All mutation endpoints require JWT auth (enforced by middleware, don't skip it)
- RBAC roles: `admin`, `warehouse`, `qc`, `sales`
- Never suppress type errors (`as any`, `// @ts-ignore`, `// @ts-expect-error` are banned)
- SQLx compile-time checked queries (no raw SQL strings floating around)
- Decimals use f64 (no rust_decimal / BigDecimal)

## Key Entry Points
- **Routes**: `backend/src/router.rs` — mounts all handlers under `/api/v1/{entity}` (~70 endpoints)
- **App**: `frontend/src/App.tsx` — ConfigProvider + QueryClientProvider + RouterProvider
- **Build**: Direct cargo/npm commands (no Makefile, seriously)

## Subordinate AGENTS.md Files

These live in subdirectories and cover module-level details:

- `backend/AGENTS_zh.md` — Rust crate details, build, dependencies
- `backend/src/AGENTS_zh.md` — Module wiring, route registration
- `backend/src/handlers/AGENTS_zh.md` — Handler patterns
- `backend/src/services/AGENTS_zh.md` — Service layer conventions
- `backend/src/repositories/AGENTS_zh.md` — Repository/CRUD patterns
- `frontend/AGENTS_zh.md` — Frontend package details
- `frontend/src/AGENTS_zh.md` — App structure, shared infra
- `frontend/src/features/AGENTS_zh.md` — Feature module template
- `docs/AGENTS_zh.md` — Design doc index and architecture decisions
