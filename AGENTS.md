# Steel Pipe DB — Project Index

## Architecture Overview

Rust + React 19 dual-package monorepo. Inventory management system (steel pipe fabrication).

```
steel-pipe-db/
├── backend/          ← Rust Axum REST API (SQLite, JWT)
│   └── src/
│       ├── handlers/     ← HTTP layer: validate → call service → respond
│       ├── services/     ← Business logic, orchestration
│       ├── repositories/ ← SQL queries (SQLx)
│       ├── models/       ← DB row structs
│       ├── dto/          ← Request/response types
│       ├── domain/       ← Enums, domain types
│       └── middleware/   ← Auth, RBAC
├── frontend/         ← React 19 + Vite + Ant Design + TanStack Query
│   └── src/
│       └── features/    ← [pipes|inventory|purchases|reports|production|customers]
│                            ├── api/       ← TanStack Query hooks
│                            ├── hooks/     ← Feature-specific logic
│                            ├── pages/     ← Route page components
│                            └── types/     ← TypeScript interfaces
└── docs/             ← Design docs, ADR, task breakdown
```

## Build & Run

| Command | What |
|---------|------|
| `make backend` | `cd backend && cargo build` |
| `make frontend` | `cd frontend && npm install && npm run build` |
| `make dev` | Start both backend (cargo run) and frontend (dev server) |
| `make clean` | Clean all build artifacts |
| `make test` | Run all tests (backend + frontend) |
| `make run` | Full production build |
| `make build` | Build both packages |

Backend runs on `http://localhost:3000`, frontend dev on `http://localhost:5173`.

## Tech Stack

### Backend
- **Rust** — nightly toolchain (2024-02-08), 2021 edition
- **Axum** 0.7 — HTTP framework
- **SQLx** 0.8 — Async SQL with compile-time checking
- **SQLite** — Database (no external DB needed)
- **JWT** (jsonwebtoken) — Auth tokens
- **bcrypt** — Password hashing
- **serde** — JSON serialization
- **tokio** — Async runtime
- **tower-http** — CORS, logging middleware
- **Dependencies**: Backpack (validator), rust_decimal, bigdecimal, chrono

### Frontend
- **React 19** — UI framework
- **Vite** — Build tool
- **TypeScript** — Type safety
- **Ant Design 5** — UI component library
- **TanStack Query (React Query)** — Server state management
- **react-router-dom v7** — Routing
- **axios** — HTTP client
- **i18next / react-i18next** — Internationalization (zh-CN primary)
- **dayjs** — Date handling
- **zod** — Schema validation

## Core Conventions

### Backend Layer Pattern
```
Request → Handler (validate) → Service (business logic) → Repository (SQL) → DB
                                                                              ↓
Response ← Handler (format)   ← Service (orchestrate)   ← Repository (rows) ←
```

- **Handlers**: Extract params, call one service method, return `ApiResponse<T>` | `ErrorResponse`
- **Services**: Orchestrate business logic, cross-entity operations, transactions
- **Repositories**: Pure SQL, single-entity CRUD, no business logic
- **Models**: Row structs matching DB schema, no methods
- **DTOs**: Request validation structs (serde::Deserialize with `validator`), response structs

### Frontend Feature Pattern
```
features/{feature}/
├── api/      ← TanStack Query hooks (useQuery, useMutation)
├── hooks/    ← Feature-specific React hooks
├── pages/    ← Page components (1 file = 1 route)
└── types/    ← TypeScript interfaces
```

## Safety Rules
- JWT auth required on all mutating endpoints (middleware enforced)
- RBAC scopes: `admin`, `manager`, `user`, `operator`
- Never suppress type errors (`as any`, `// @ts-ignore`, `// @ts-expect-error`)
- SQLx compile-time checked queries (no raw SQL strings)
- All decimal values use `rust_decimal` / `BigDecimal` type

## Key Entry Points
- **Router**: `backend/src/bin/main.rs` → mounts all handlers at `/api/v1/{entity}`
- **App**: `frontend/src/App.tsx` → React Router setup with layouts
- **Build**: `Makefile` at root orchestrates both packages

## Subordinate AGENTS.md Files
- `backend/AGENTS.md` — Rust package details, build, dependencies
- `backend/src/AGENTS.md` — Module wiring, router registration
- `backend/src/handlers/AGENTS.md` — Handler patterns
- `backend/src/services/AGENTS.md` — Service layer conventions
- `backend/src/repositories/AGENTS.md` — Repository/CRUD patterns
- `frontend/AGENTS.md` — Frontend package details
- `frontend/src/AGENTS.md` — App structure, shared infrastructure
- `frontend/src/features/AGENTS.md` — Feature module template
- `docs/AGENTS.md` — Design docs index & architecture decisions
