# AGENTS.md — steel_pipe_db

## Project Identity

- **What**: API 5CT seamless steel pipe & screen pipe inventory management system (oil & gas)
- **Stack**: Rust (Axum 0.8+ / SQLx / SQLite WAL) + React 19 (Ant Design 5 / TanStack Query / Zustand / TypeScript / Vite 6)
- **Remote**: `xiaokonglong123-dot/steel_pipe_db` on GitHub
- **Current state**: All source files have been deleted (commit `3cd0d86`). Only design docs remain.

## Source of Truth — Design Documents

The repo is currently **design-first / spec-only**. All implementation source has been removed. The docs/ directory is the canonical reference:

| File | What it contains |
|------|-----------------|
| `docs/需求文档.md` | Full PRD: features, data model, API 5CT standards, roadmap |
| `docs/详细设计文档.md` | Architecture, module design, DB schema (19 tables), REST API, security |
| `docs/前端设计文档.md` | React component tree, routing, state management, i18n, theme |
| `docs/tasks/progress.md` | Master task breakdown: 12 backend + 12 frontend modules across 3 phases |
| `docs/tasks/phase*/` | Individual task files (~320 items total) |

## Project Structure

```
docs/
├── superpowers/specs/        # One optimization spec for rust-axum-vue3
├── tasks/phase1/             # P0: pipe mgmt, inventory, auth, tracing (8 files)
├── tasks/phase2/             # P1: quality, purchase, sales, data-io (8 files)
├── tasks/phase3/             # P2: contracts, reports, labels, i18n (7 files)
├── 需求文档.md               # PRD
├── 详细设计文档.md            # Detailed design (architecture, DB, API)
├── 前端设计文档.md             # Frontend design (component tree, routing)
└── tasks/progress.md         # Master progress tracker
.github/workflows/
├── ci.yml                    # Rust cargo check + Node type-check+build (broken - dirs removed)
└── python-test.yml           # Flask legacy tests (broken - dirs removed)
```

## Architecture Notes (from design docs)

- **Backend layering**: Handler → Service → Repository → Domain (Axum + SQLx)
- **DB strategy**: SQLite WAL mode, no foreign key constraints (app-level integrity), soft deletes via `deleted_at`, ISO 8601 text timestamps
- **State management**: Zustand (client: auth, app, unit prefs) + TanStack Query (server: all business data)
- **Auth**: JWT + Argon2 passwords, RBAC with 4 roles (admin / warehouse / qc / sales)
- **i18n**: react-i18next, per-module namespaces (zh + en), unit system toggle (metric/imperial)
- **Inbound/Outbound**: Header+Items pattern with approval workflow (auto-approve for purchase/sales, manual for production/return/transfer/scrapped)
- **Inventory tracking**: Per-pipe granularity via `inventory_logs`, ATP calculation for sales

## Critical Gotchas

- **CI is broken**: Both `.github/workflows/ci.yml` and `python-test.yml` reference `backend/` and `frontend/` directories that no longer exist. Any rebuild must update or remove these.
- **`.omo/` and `memory.md` are gitignored** — OpenCode working files are excluded from version control.
- **SSL cert issue on this machine**: `git push` fails with `error adding trust anchors from file: /etc/ssl/certs/ca-certificates.crt`. Workaround: `git config http.sslVerify false` before push, then restore.
- **Commit style**: Mixed Chinese/English commit messages, conventional-commits-ish but not strict.

## Commands

No runnable commands currently (source removed). When reimplementing from design docs:

```bash
# Backend (once backend/ exists)
cd backend && cargo run          # Start Axum dev server
cargo check                      # Type-check only (no full build)
cargo test                       # Run tests

# Frontend (once frontend/ exists)
cd frontend && npm run dev       # Vite dev server
npx tsc --noEmit                 # TypeScript type check
npm run build                    # Production build
```

## Key Links

- Remote: `https://github.com/xiaokonglong123-dot/steel_pipe_db`
- The PRD (`docs/需求文档.md`) is the best starting point for understanding the domain
- Task files in `docs/tasks/phase*/` are the implementation roadmap — follow them in order
