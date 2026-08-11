# Ikari_Shinji — ERP

A general-purpose ERP (Enterprise Resource Planning) system.

- **Backend:** Rust + Axum + SQLx + SQLite (Decimal money, JWT/Argon2)
- **Frontend:** Vue 3 + Pinia + Element Plus + TanStack Vue Query
- **Package manager:** bun (frontend), cargo (backend)

See [AGENTS.md](./AGENTS.md) for the project index, architecture, and developer conventions.
Design docs live in [docs/](./docs/); pre-rewrite (React-stack) docs are archived in [docs/legacy/](./docs/legacy/).

## Quick Start

```bash
# Backend (Rust Axum on :3000)
cd backend
cp .env.example .env
cargo run

# Frontend (Vue 3 + Vite on :5173)
cd frontend
bun install
bun run dev
```

Open `http://localhost:5173` and log in with `admin` / `admin123`.

## Build & Verify

| What | How |
| ---- | --- |
| Backend type-check | `cd backend && cargo check --all-targets` |
| Backend tests | `cd backend && cargo test --all` |
| Frontend type-check | `cd frontend && bunx tsc --noEmit` |
| Frontend build | `cd frontend && bun run build` |

Database is SQLite3 (single file `backend/data/erp.db`). 121 backend tests, frontend tsc + build green.

## History

The pre-rewrite (React 19 + Ant Design) stack lives on the `legacy/steel-pipe-react` branch. The current `main` is the erp-v2 rewrite era.
